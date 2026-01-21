//! Motor de consenso do Tetrad.
//!
//! Orquestra o processo de avaliação, aplicando as regras
//! de consenso e gerenciando loops de refinamento.

use std::collections::HashMap;

use crate::types::config::ConsensusConfig;
use crate::types::responses::{Decision, EvaluationResult, ModelVote};

use super::aggregator::VoteAggregator;
use super::rules::{create_rule, ConsensusRule};

/// Motor de consenso.
///
/// Responsável por:
/// - Aplicar regras de consenso aos votos
/// - Calcular resultados agregados
/// - Determinar se consenso foi alcançado
pub struct ConsensusEngine {
    config: ConsensusConfig,
    rule: Box<dyn ConsensusRule>,
}

impl ConsensusEngine {
    /// Cria um novo motor de consenso.
    pub fn new(config: ConsensusConfig) -> Self {
        let rule = create_rule(&config.default_rule);
        Self { config, rule }
    }

    /// Avalia os votos e retorna o resultado.
    pub fn evaluate(
        &self,
        votes: HashMap<String, ModelVote>,
        request_id: &str,
    ) -> EvaluationResult {
        VoteAggregator::aggregate(votes, self.rule.as_ref(), self.config.min_score, request_id)
    }

    /// Verifica se o consenso foi alcançado.
    pub fn is_consensus_achieved(&self, result: &EvaluationResult) -> bool {
        result.consensus_achieved
    }

    /// Verifica se mais loops de refinamento são permitidos.
    pub fn can_retry(&self, current_loop: u8) -> bool {
        current_loop < self.config.max_loops
    }

    /// Retorna a decisão baseada nos votos.
    pub fn get_decision(&self, votes: &HashMap<String, ModelVote>) -> Decision {
        self.rule.evaluate(votes, self.config.min_score)
    }

    /// Retorna o score mínimo configurado.
    pub fn min_score(&self) -> u8 {
        self.config.min_score
    }

    /// Retorna o número máximo de loops permitidos.
    pub fn max_loops(&self) -> u8 {
        self.config.max_loops
    }

    /// Retorna o nome da regra de consenso atual.
    pub fn rule_name(&self) -> &str {
        self.rule.name()
    }

    /// Atualiza a regra de consenso.
    pub fn set_rule(&mut self, rule: Box<dyn ConsensusRule>) {
        self.rule = rule;
    }

    /// Verifica se deve bloquear imediatamente (issues críticos).
    pub fn should_block_immediately(&self, result: &EvaluationResult) -> bool {
        use crate::types::responses::Severity;

        // Bloqueia se houver findings críticos
        result
            .findings
            .iter()
            .any(|f| matches!(f.severity, Severity::Critical))
    }

    /// Calcula a confiança do consenso (0.0 - 1.0).
    ///
    /// Baseado em:
    /// - Unanimidade dos votos
    /// - Score médio vs min_score
    /// - Consistência dos reasonings
    pub fn calculate_confidence(&self, result: &EvaluationResult) -> f64 {
        if result.votes.is_empty() {
            return 0.0;
        }

        let mut confidence = 0.0;

        // Fator 1: Unanimidade (até 0.4)
        let pass_count = result
            .votes
            .values()
            .filter(|v| v.vote == crate::types::responses::Vote::Pass)
            .count();
        let unanimity = pass_count as f64 / result.votes.len() as f64;
        confidence += unanimity * 0.4;

        // Fator 2: Score vs min_score (até 0.3)
        let score_factor = if result.score >= self.config.min_score {
            (result.score - self.config.min_score) as f64 / (100 - self.config.min_score) as f64
        } else {
            0.0
        };
        confidence += score_factor * 0.3;

        // Fator 3: Consenso alcançado (até 0.3)
        if result.consensus_achieved {
            confidence += 0.3;
        }

        confidence.min(1.0)
    }
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new(ConsensusConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::config::ConsensusRule as ConsensusRuleConfig;
    use crate::types::responses::Vote;

    fn create_vote(name: &str, vote: Vote, score: u8) -> (String, ModelVote) {
        (name.to_string(), ModelVote::new(name, vote, score))
    }

    fn create_config(rule: ConsensusRuleConfig, min_score: u8, max_loops: u8) -> ConsensusConfig {
        ConsensusConfig {
            default_rule: rule,
            min_score,
            max_loops,
        }
    }

    #[test]
    fn test_new_engine() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        assert_eq!(engine.rule_name(), "strong");
        assert_eq!(engine.min_score(), 70);
        assert_eq!(engine.max_loops(), 3);
    }

    #[test]
    fn test_evaluate_pass() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Pass, 85),
            create_vote("Gemini", Vote::Pass, 90),
            create_vote("Qwen", Vote::Pass, 88),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");

        assert_eq!(result.decision, Decision::Pass);
        assert!(result.consensus_achieved);
    }

    #[test]
    fn test_evaluate_block() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Fail, 30),
            create_vote("Gemini", Vote::Fail, 25),
            create_vote("Qwen", Vote::Fail, 20),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");

        assert_eq!(result.decision, Decision::Block);
    }

    #[test]
    fn test_can_retry() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        assert!(engine.can_retry(0));
        assert!(engine.can_retry(1));
        assert!(engine.can_retry(2));
        assert!(!engine.can_retry(3));
        assert!(!engine.can_retry(4));
    }

    #[test]
    fn test_calculate_confidence_high() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Pass, 95),
            create_vote("Gemini", Vote::Pass, 98),
            create_vote("Qwen", Vote::Pass, 97),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        let confidence = engine.calculate_confidence(&result);

        assert!(confidence > 0.8);
    }

    #[test]
    fn test_calculate_confidence_low() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Pass, 72),
            create_vote("Gemini", Vote::Warn, 65),
            create_vote("Qwen", Vote::Fail, 40),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        let confidence = engine.calculate_confidence(&result);

        assert!(confidence < 0.5);
    }

    #[test]
    fn test_golden_rule_engine() {
        let config = create_config(ConsensusRuleConfig::Golden, 80, 3);
        let engine = ConsensusEngine::new(config);

        assert_eq!(engine.rule_name(), "golden");

        // Com Golden rule, mesmo um WARN bloqueia
        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Pass, 85),
            create_vote("Gemini", Vote::Warn, 75),
            create_vote("Qwen", Vote::Pass, 88),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        assert_eq!(result.decision, Decision::Revise);
    }

    #[test]
    fn test_weak_rule_engine() {
        let config = create_config(ConsensusRuleConfig::Weak, 70, 3);
        let engine = ConsensusEngine::new(config);

        assert_eq!(engine.rule_name(), "weak");

        // Com Weak rule, 2 PASS são suficientes
        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Pass, 85),
            create_vote("Gemini", Vote::Pass, 80),
            create_vote("Qwen", Vote::Fail, 30),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        assert_eq!(result.decision, Decision::Pass);
    }
}
