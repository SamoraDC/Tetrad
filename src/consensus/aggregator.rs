//! Agregador de votos do Tetrad.
//!
//! Responsável por:
//! - Agregar votos dos executores
//! - Extrair issues comuns (consenso em problemas)
//! - Consolidar feedback em mensagem coerente
//! - Calcular score agregado

use std::collections::HashMap;

use crate::types::responses::{Decision, EvaluationResult, Finding, ModelVote, Severity, Vote};

use super::rules::ConsensusRule;

/// Agregador de votos.
pub struct VoteAggregator;

impl VoteAggregator {
    /// Agrega votos e retorna o resultado da avaliação.
    pub fn aggregate(
        votes: HashMap<String, ModelVote>,
        rule: &dyn ConsensusRule,
        min_score: u8,
        request_id: &str,
    ) -> EvaluationResult {
        let decision = rule.evaluate(&votes, min_score);
        let consensus_achieved = rule.is_consensus_achieved(&votes, min_score);
        let score = Self::calculate_score(&votes);
        let findings = Self::extract_findings(&votes);
        let feedback = Self::consolidate_feedback(&votes, &decision);

        EvaluationResult {
            request_id: request_id.to_string(),
            decision,
            score,
            votes,
            findings,
            feedback,
            consensus_achieved,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Calcula o score agregado (média dos scores).
    pub fn calculate_score(votes: &HashMap<String, ModelVote>) -> u8 {
        if votes.is_empty() {
            return 0;
        }

        let total: u32 = votes.values().map(|v| v.score as u32).sum();
        (total / votes.len() as u32) as u8
    }

    /// Calcula o score mínimo entre os votos.
    pub fn calculate_min_score(votes: &HashMap<String, ModelVote>) -> u8 {
        votes.values().map(|v| v.score).min().unwrap_or(0)
    }

    /// Extrai findings dos votos, consolidando issues comuns.
    pub fn extract_findings(votes: &HashMap<String, ModelVote>) -> Vec<Finding> {
        let mut findings: Vec<Finding> = Vec::new();
        let mut issue_counts: HashMap<String, (Vec<String>, Severity)> = HashMap::new();

        // Conta quantos executores reportaram cada issue
        for (executor, vote) in votes {
            for issue in &vote.issues {
                let key = Self::normalize_issue(issue);
                let entry = issue_counts
                    .entry(key.clone())
                    .or_insert_with(|| (Vec::new(), Self::infer_severity(issue)));
                entry.0.push(executor.clone());
            }
        }

        // Cria findings para issues reportados por múltiplos executores (consenso)
        for (issue, (executors, severity)) in &issue_counts {
            let consensus_strength = if executors.len() >= 3 {
                "forte"
            } else if executors.len() >= 2 {
                "moderado"
            } else {
                "fraco"
            };

            // Busca sugestão correspondente
            let suggestion = Self::find_suggestion_for_issue(votes, issue);

            // Infere categoria do issue
            let category = Self::infer_category(issue);

            findings.push(Finding {
                issue: issue.clone(),
                severity: *severity,
                category,
                lines: None,
                suggestion,
                source: executors.join(", "),
                consensus_strength: consensus_strength.to_string(),
            });
        }

        // Ordena por severidade (Critical > Error > Warning > Info)
        findings.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Critical => 0,
                Severity::Error => 1,
                Severity::Warning => 2,
                Severity::Info => 3,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        findings
    }

    /// Consolida feedback de todos os executores.
    pub fn consolidate_feedback(votes: &HashMap<String, ModelVote>, decision: &Decision) -> String {
        let mut feedback = String::new();

        // Cabeçalho baseado na decisão
        let header = match decision {
            Decision::Pass => "## Avaliação Aprovada",
            Decision::Revise => "## Revisão Necessária",
            Decision::Block => "## Avaliação Bloqueada",
        };
        feedback.push_str(header);
        feedback.push_str("\n\n");

        // Resumo dos votos
        let pass_count = votes.values().filter(|v| v.vote == Vote::Pass).count();
        let warn_count = votes.values().filter(|v| v.vote == Vote::Warn).count();
        let fail_count = votes.values().filter(|v| v.vote == Vote::Fail).count();

        feedback.push_str(&format!(
            "**Votos:** {} PASS | {} WARN | {} FAIL\n\n",
            pass_count, warn_count, fail_count
        ));

        // Feedback individual de cada executor
        feedback.push_str("### Feedback dos Avaliadores\n\n");

        for (executor, vote) in votes {
            let icon = match vote.vote {
                Vote::Pass => "✓",
                Vote::Warn => "⚠",
                Vote::Fail => "✗",
            };

            feedback.push_str(&format!(
                "**{} {}** (score: {})\n",
                icon, executor, vote.score
            ));

            if !vote.reasoning.is_empty() {
                feedback.push_str(&format!("> {}\n", vote.reasoning));
            }

            if !vote.issues.is_empty() {
                feedback.push_str("\nIssues:\n");
                for issue in &vote.issues {
                    feedback.push_str(&format!("- {}\n", issue));
                }
            }

            if !vote.suggestions.is_empty() {
                feedback.push_str("\nSugestões:\n");
                for suggestion in &vote.suggestions {
                    feedback.push_str(&format!("- {}\n", suggestion));
                }
            }

            feedback.push('\n');
        }

        // Ações recomendadas
        feedback.push_str("### Ações Recomendadas\n\n");
        match decision {
            Decision::Pass => {
                feedback.push_str("O código foi aprovado por todos os avaliadores. ");
                feedback.push_str("Você pode prosseguir com a implementação.\n");
            }
            Decision::Revise => {
                feedback.push_str("O código precisa de ajustes antes de ser aprovado. ");
                feedback.push_str("Revise os issues acima e submeta novamente.\n");
            }
            Decision::Block => {
                feedback.push_str("O código foi bloqueado devido a problemas críticos. ");
                feedback.push_str("Corrija TODOS os issues marcados como Critical ou Error antes de prosseguir.\n");
            }
        }

        feedback
    }

    /// Normaliza um issue para comparação (lowercase, trim).
    fn normalize_issue(issue: &str) -> String {
        issue.to_lowercase().trim().to_string()
    }

    /// Infere a severidade de um issue baseado em keywords.
    fn infer_severity(issue: &str) -> Severity {
        let issue_lower = issue.to_lowercase();

        if issue_lower.contains("critical")
            || issue_lower.contains("security")
            || issue_lower.contains("vulnerability")
            || issue_lower.contains("injection")
        {
            Severity::Critical
        } else if issue_lower.contains("error")
            || issue_lower.contains("bug")
            || issue_lower.contains("fail")
            || issue_lower.contains("crash")
        {
            Severity::Error
        } else if issue_lower.contains("warning")
            || issue_lower.contains("warn")
            || issue_lower.contains("should")
            || issue_lower.contains("consider")
        {
            Severity::Warning
        } else {
            Severity::Info
        }
    }

    /// Infere a categoria de um issue baseado em keywords.
    fn infer_category(issue: &str) -> String {
        let issue_lower = issue.to_lowercase();

        if issue_lower.contains("security")
            || issue_lower.contains("injection")
            || issue_lower.contains("vulnerability")
            || issue_lower.contains("password")
            || issue_lower.contains("credential")
        {
            "security".to_string()
        } else if issue_lower.contains("performance")
            || issue_lower.contains("slow")
            || issue_lower.contains("memory")
            || issue_lower.contains("allocation")
        {
            "performance".to_string()
        } else if issue_lower.contains("logic")
            || issue_lower.contains("bug")
            || issue_lower.contains("incorrect")
            || issue_lower.contains("wrong")
        {
            "logic".to_string()
        } else if issue_lower.contains("style")
            || issue_lower.contains("convention")
            || issue_lower.contains("naming")
            || issue_lower.contains("format")
        {
            "style".to_string()
        } else if issue_lower.contains("architecture")
            || issue_lower.contains("design")
            || issue_lower.contains("pattern")
            || issue_lower.contains("structure")
        {
            "architecture".to_string()
        } else {
            "general".to_string()
        }
    }

    /// Busca uma sugestão correspondente a um issue.
    fn find_suggestion_for_issue(
        votes: &HashMap<String, ModelVote>,
        issue: &str,
    ) -> Option<String> {
        let issue_normalized = Self::normalize_issue(issue);

        for vote in votes.values() {
            for (i, vote_issue) in vote.issues.iter().enumerate() {
                if Self::normalize_issue(vote_issue) == issue_normalized {
                    if let Some(suggestion) = vote.suggestions.get(i) {
                        return Some(suggestion.clone());
                    }
                }
            }

            // Se não encontrou por índice, tenta a primeira sugestão disponível
            if !vote.suggestions.is_empty() {
                // Usa chars() para slice seguro em UTF-8 (evita panic em caracteres não-ASCII)
                let issue_prefix: String = issue_normalized.chars().take(20).collect();
                for suggestion in &vote.suggestions {
                    if suggestion.to_lowercase().contains(&issue_prefix) {
                        return Some(suggestion.clone());
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::rules::StrongRule;

    fn create_vote(name: &str, vote: Vote, score: u8) -> (String, ModelVote) {
        (name.to_string(), ModelVote::new(name, vote, score))
    }

    fn create_vote_with_issues(
        name: &str,
        vote: Vote,
        score: u8,
        issues: Vec<&str>,
        suggestions: Vec<&str>,
    ) -> (String, ModelVote) {
        let mut mv = ModelVote::new(name, vote, score);
        mv.issues = issues.into_iter().map(String::from).collect();
        mv.suggestions = suggestions.into_iter().map(String::from).collect();
        (name.to_string(), mv)
    }

    #[test]
    fn test_calculate_score() {
        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Pass, 80),
            create_vote("Gemini", Vote::Pass, 90),
            create_vote("Qwen", Vote::Pass, 85),
        ]
        .into_iter()
        .collect();

        assert_eq!(VoteAggregator::calculate_score(&votes), 85);
    }

    #[test]
    fn test_calculate_min_score() {
        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Pass, 80),
            create_vote("Gemini", Vote::Pass, 90),
            create_vote("Qwen", Vote::Warn, 60),
        ]
        .into_iter()
        .collect();

        assert_eq!(VoteAggregator::calculate_min_score(&votes), 60);
    }

    #[test]
    fn test_extract_findings_common_issues() {
        let votes: HashMap<String, ModelVote> = vec![
            create_vote_with_issues(
                "Codex",
                Vote::Warn,
                70,
                vec!["SQL injection vulnerability"],
                vec!["Use parameterized queries"],
            ),
            create_vote_with_issues(
                "Gemini",
                Vote::Warn,
                65,
                vec!["sql injection vulnerability"],
                vec!["Sanitize inputs"],
            ),
            create_vote_with_issues("Qwen", Vote::Pass, 85, vec![], vec![]),
        ]
        .into_iter()
        .collect();

        let findings = VoteAggregator::extract_findings(&votes);
        assert!(!findings.is_empty());

        // Deve haver um finding para SQL injection
        let sql_finding = findings.iter().find(|f| f.issue.contains("sql injection"));
        assert!(sql_finding.is_some());
    }

    #[test]
    fn test_aggregate_pass() {
        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Pass, 85),
            create_vote("Gemini", Vote::Pass, 90),
            create_vote("Qwen", Vote::Pass, 88),
        ]
        .into_iter()
        .collect();

        let rule = StrongRule;
        let result = VoteAggregator::aggregate(votes, &rule, 70, "test-123");

        assert_eq!(result.decision, Decision::Pass);
        assert!(result.consensus_achieved);
        assert_eq!(result.score, 87); // (85+90+88)/3
    }

    #[test]
    fn test_consolidate_feedback_pass() {
        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Pass, 85),
            create_vote("Gemini", Vote::Pass, 90),
        ]
        .into_iter()
        .collect();

        let feedback = VoteAggregator::consolidate_feedback(&votes, &Decision::Pass);

        assert!(feedback.contains("Avaliação Aprovada"));
        assert!(feedback.contains("2 PASS"));
    }

    #[test]
    fn test_consolidate_feedback_block() {
        let votes: HashMap<String, ModelVote> = vec![
            create_vote("Codex", Vote::Fail, 30),
            create_vote("Gemini", Vote::Fail, 25),
        ]
        .into_iter()
        .collect();

        let feedback = VoteAggregator::consolidate_feedback(&votes, &Decision::Block);

        assert!(feedback.contains("Avaliação Bloqueada"));
        assert!(feedback.contains("2 FAIL"));
    }

    #[test]
    fn test_infer_severity() {
        assert_eq!(
            VoteAggregator::infer_severity("SQL injection vulnerability"),
            Severity::Critical
        );
        assert_eq!(
            VoteAggregator::infer_severity("Error in logic"),
            Severity::Error
        );
        assert_eq!(
            VoteAggregator::infer_severity("Warning: consider refactoring"),
            Severity::Warning
        );
        assert_eq!(
            VoteAggregator::infer_severity("Minor style issue"),
            Severity::Info
        );
    }
}
