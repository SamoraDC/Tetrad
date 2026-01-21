//! Regras de consenso do Tetrad.
//!
//! Define as três regras de consenso disponíveis:
//! - Golden: Unanimidade (todos devem votar PASS)
//! - Strong: Consenso forte (3/3 CLIs concordam)
//! - Weak: Consenso fraco (2+ CLIs concordam)

use std::collections::HashMap;

use crate::types::config::ConsensusRule as ConsensusRuleConfig;
use crate::types::responses::{Decision, ModelVote, Vote};

/// Trait para regras de consenso.
pub trait ConsensusRule: Send + Sync {
    /// Nome da regra.
    fn name(&self) -> &str;

    /// Avalia os votos e retorna a decisão.
    fn evaluate(&self, votes: &HashMap<String, ModelVote>, min_score: u8) -> Decision;

    /// Número mínimo de votos necessários para consenso.
    fn min_required(&self) -> usize;

    /// Verifica se o consenso foi alcançado.
    fn is_consensus_achieved(&self, votes: &HashMap<String, ModelVote>, min_score: u8) -> bool;
}

/// Regra de Ouro: Unanimidade necessária.
///
/// Todos os avaliadores devem votar PASS com score >= min_score.
/// É a regra mais restritiva, ideal para código crítico.
#[derive(Debug, Clone, Default)]
pub struct GoldenRule;

impl ConsensusRule for GoldenRule {
    fn name(&self) -> &str {
        "golden"
    }

    fn evaluate(&self, votes: &HashMap<String, ModelVote>, min_score: u8) -> Decision {
        // Verifica mínimo de votos necessários
        if votes.len() < self.min_required() {
            return Decision::Revise; // Sem votos suficientes, precisa esperar
        }

        let all_pass = votes
            .values()
            .all(|v| v.vote == Vote::Pass && v.score >= min_score);

        let any_fail = votes.values().any(|v| v.vote == Vote::Fail);

        if all_pass {
            Decision::Pass
        } else if any_fail {
            Decision::Block
        } else {
            Decision::Revise
        }
    }

    fn min_required(&self) -> usize {
        3 // Todos os 3 CLIs
    }

    fn is_consensus_achieved(&self, votes: &HashMap<String, ModelVote>, min_score: u8) -> bool {
        if votes.len() < self.min_required() {
            return false;
        }
        matches!(self.evaluate(votes, min_score), Decision::Pass)
    }
}

/// Consenso Forte: 3/3 CLIs devem concordar.
///
/// Todos os avaliadores devem concordar na decisão (PASS ou FAIL).
/// É a regra padrão, balanceando rigor e praticidade.
#[derive(Debug, Clone, Default)]
pub struct StrongRule;

impl ConsensusRule for StrongRule {
    fn name(&self) -> &str {
        "strong"
    }

    fn evaluate(&self, votes: &HashMap<String, ModelVote>, min_score: u8) -> Decision {
        // Verifica mínimo de votos necessários (3/3)
        if votes.len() < self.min_required() {
            return Decision::Revise; // Sem votos suficientes, precisa esperar
        }

        let pass_count = votes.values().filter(|v| v.vote == Vote::Pass).count();
        let fail_count = votes.values().filter(|v| v.vote == Vote::Fail).count();

        let avg_score = self.calculate_average_score(votes);

        // Strong Rule: 3/3 devem concordar
        // Todos passam (3/3 PASS)
        if pass_count == self.min_required() && avg_score >= min_score {
            return Decision::Pass;
        }

        // Todos falham (3/3 FAIL)
        if fail_count == self.min_required() {
            return Decision::Block;
        }

        // Qualquer discordância ou score baixo = revisão
        Decision::Revise
    }

    fn min_required(&self) -> usize {
        3
    }

    fn is_consensus_achieved(&self, votes: &HashMap<String, ModelVote>, min_score: u8) -> bool {
        if votes.len() < self.min_required() {
            return false;
        }

        let decision = self.evaluate(votes, min_score);
        matches!(decision, Decision::Pass | Decision::Block)
    }
}

impl StrongRule {
    fn calculate_average_score(&self, votes: &HashMap<String, ModelVote>) -> u8 {
        if votes.is_empty() {
            return 0;
        }
        let total: u32 = votes.values().map(|v| v.score as u32).sum();
        (total / votes.len() as u32) as u8
    }
}

/// Consenso Fraco: 2+ CLIs concordam.
///
/// Maioria simples decide. É a regra mais permissiva,
/// útil para protótipos e experimentos.
#[derive(Debug, Clone, Default)]
pub struct WeakRule;

impl ConsensusRule for WeakRule {
    fn name(&self) -> &str {
        "weak"
    }

    fn evaluate(&self, votes: &HashMap<String, ModelVote>, min_score: u8) -> Decision {
        if votes.is_empty() {
            return Decision::Block;
        }

        let pass_votes: Vec<_> = votes.values().filter(|v| v.vote == Vote::Pass).collect();
        let fail_count = votes.values().filter(|v| v.vote == Vote::Fail).count();

        // Maioria passa (2+ de 3) - usa média apenas dos votos PASS
        if pass_votes.len() >= 2 {
            let avg_pass_score = self.calculate_average_score_of(&pass_votes);
            if avg_pass_score >= min_score {
                return Decision::Pass;
            }
        }

        // Maioria falha (2+ de 3)
        if fail_count >= 2 {
            return Decision::Block;
        }

        // Empate ou sem maioria clara
        Decision::Revise
    }

    fn min_required(&self) -> usize {
        2 // Apenas 2 necessários para decisão
    }

    fn is_consensus_achieved(&self, votes: &HashMap<String, ModelVote>, min_score: u8) -> bool {
        if votes.len() < self.min_required() {
            return false;
        }

        let decision = self.evaluate(votes, min_score);
        matches!(decision, Decision::Pass | Decision::Block)
    }
}

impl WeakRule {
    fn calculate_average_score_of(&self, votes: &[&ModelVote]) -> u8 {
        if votes.is_empty() {
            return 0;
        }
        let total: u32 = votes.iter().map(|v| v.score as u32).sum();
        (total / votes.len() as u32) as u8
    }
}

/// Cria uma regra de consenso a partir da configuração.
pub fn create_rule(config: &ConsensusRuleConfig) -> Box<dyn ConsensusRule> {
    match config {
        ConsensusRuleConfig::Golden => Box::new(GoldenRule),
        ConsensusRuleConfig::Strong => Box::new(StrongRule),
        ConsensusRuleConfig::Weak => Box::new(WeakRule),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_vote(name: &str, vote: Vote, score: u8) -> (String, ModelVote) {
        (name.to_string(), ModelVote::new(name, vote, score))
    }

    fn create_votes(votes: Vec<(&str, Vote, u8)>) -> HashMap<String, ModelVote> {
        votes
            .into_iter()
            .map(|(n, v, s)| create_vote(n, v, s))
            .collect()
    }

    // Testes para GoldenRule
    #[test]
    fn test_golden_rule_all_pass() {
        let rule = GoldenRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Pass, 85),
            ("Gemini", Vote::Pass, 90),
            ("Qwen", Vote::Pass, 88),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Pass);
        assert!(rule.is_consensus_achieved(&votes, 70));
    }

    #[test]
    fn test_golden_rule_one_fail() {
        let rule = GoldenRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Pass, 85),
            ("Gemini", Vote::Fail, 40),
            ("Qwen", Vote::Pass, 88),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Block);
        assert!(!rule.is_consensus_achieved(&votes, 70));
    }

    #[test]
    fn test_golden_rule_low_score() {
        let rule = GoldenRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Pass, 60),
            ("Gemini", Vote::Pass, 65),
            ("Qwen", Vote::Pass, 68),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Revise);
    }

    // Testes para StrongRule
    #[test]
    fn test_strong_rule_all_pass() {
        let rule = StrongRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Pass, 85),
            ("Gemini", Vote::Pass, 90),
            ("Qwen", Vote::Pass, 88),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Pass);
        assert!(rule.is_consensus_achieved(&votes, 70));
    }

    #[test]
    fn test_strong_rule_not_unanimous_revise() {
        // Strong Rule exige 3/3 - 2 PASS + 1 WARN = Revise
        let rule = StrongRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Pass, 85),
            ("Gemini", Vote::Pass, 90),
            ("Qwen", Vote::Warn, 65),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Revise);
    }

    #[test]
    fn test_strong_rule_not_unanimous_fail() {
        // Strong Rule exige 3/3 - 2 FAIL + 1 PASS = Revise (não Block)
        let rule = StrongRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Fail, 30),
            ("Gemini", Vote::Fail, 25),
            ("Qwen", Vote::Pass, 85),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Revise);
    }

    #[test]
    fn test_strong_rule_all_fail() {
        // Strong Rule: 3/3 FAIL = Block
        let rule = StrongRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Fail, 30),
            ("Gemini", Vote::Fail, 25),
            ("Qwen", Vote::Fail, 20),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Block);
    }

    // Testes para WeakRule
    #[test]
    fn test_weak_rule_two_pass() {
        let rule = WeakRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Pass, 85),
            ("Gemini", Vote::Pass, 90),
            ("Qwen", Vote::Fail, 30),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Pass);
        assert!(rule.is_consensus_achieved(&votes, 70));
    }

    #[test]
    fn test_weak_rule_two_fail() {
        let rule = WeakRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Fail, 30),
            ("Gemini", Vote::Fail, 25),
            ("Qwen", Vote::Pass, 85),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Block);
    }

    #[test]
    fn test_weak_rule_no_majority() {
        let rule = WeakRule;
        let votes = create_votes(vec![
            ("Codex", Vote::Pass, 85),
            ("Gemini", Vote::Warn, 60),
            ("Qwen", Vote::Fail, 30),
        ]);

        assert_eq!(rule.evaluate(&votes, 70), Decision::Revise);
    }

    // Testes para create_rule
    #[test]
    fn test_create_rule() {
        let golden = create_rule(&ConsensusRuleConfig::Golden);
        assert_eq!(golden.name(), "golden");

        let strong = create_rule(&ConsensusRuleConfig::Strong);
        assert_eq!(strong.name(), "strong");

        let weak = create_rule(&ConsensusRuleConfig::Weak);
        assert_eq!(weak.name(), "weak");
    }
}
