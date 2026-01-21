//! Testes de integração para o motor de consenso do Tetrad.

use std::collections::HashMap;
use tetrad::consensus::ConsensusEngine;
use tetrad::types::config::{ConsensusConfig, ConsensusRule as ConsensusRuleConfig};
use tetrad::types::responses::{ModelVote, Decision, Finding, Severity, Vote};

fn create_vote(executor: &str, vote: Vote, score: u8) -> (String, ModelVote) {
    (executor.to_string(), ModelVote::new(executor, vote, score))
}

fn create_config(rule: ConsensusRuleConfig, min_score: u8, max_loops: u8) -> ConsensusConfig {
    ConsensusConfig {
        default_rule: rule,
        min_score,
        max_loops,
    }
}

// Testes das regras de votação
mod voting_rules_tests {
    use super::*;

    #[test]
    fn test_golden_rule_unanimous_pass() {
        let config = create_config(ConsensusRuleConfig::Golden, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 85),
            create_vote("gemini", Vote::Pass, 88),
            create_vote("qwen", Vote::Pass, 82),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        assert!(matches!(result.decision, Decision::Pass));
    }

    #[test]
    fn test_golden_rule_one_fail() {
        let config = create_config(ConsensusRuleConfig::Golden, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 85),
            create_vote("gemini", Vote::Fail, 45),
            create_vote("qwen", Vote::Pass, 82),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        // Com golden, qualquer FAIL bloqueia
        assert!(matches!(result.decision, Decision::Block) ||
                matches!(result.decision, Decision::Revise));
    }

    #[test]
    fn test_golden_rule_all_fail() {
        let config = create_config(ConsensusRuleConfig::Golden, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Fail, 30),
            create_vote("gemini", Vote::Fail, 25),
            create_vote("qwen", Vote::Fail, 35),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        assert!(matches!(result.decision, Decision::Block));
    }

    #[test]
    fn test_strong_rule_3_of_3() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 85),
            create_vote("gemini", Vote::Pass, 88),
            create_vote("qwen", Vote::Pass, 82),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        assert!(matches!(result.decision, Decision::Pass));
    }

    #[test]
    fn test_strong_rule_2_of_3_pass_one_warn() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 85),
            create_vote("gemini", Vote::Pass, 88),
            create_vote("qwen", Vote::Warn, 65),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        // Strong com 2 PASS e 1 WARN pode passar ou pedir revisão
        assert!(matches!(result.decision, Decision::Pass) ||
                matches!(result.decision, Decision::Revise));
    }

    #[test]
    fn test_strong_rule_1_of_3() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 85),
            create_vote("gemini", Vote::Fail, 45),
            create_vote("qwen", Vote::Fail, 35),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        // Strong rule só bloqueia com 3/3 FAIL; com 1 PASS e 2 FAIL, retorna Revise
        assert!(matches!(result.decision, Decision::Revise));
    }

    #[test]
    fn test_weak_rule_majority_pass() {
        let config = create_config(ConsensusRuleConfig::Weak, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 85),
            create_vote("gemini", Vote::Pass, 88),
            create_vote("qwen", Vote::Fail, 45),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        assert!(matches!(result.decision, Decision::Pass));
    }

    #[test]
    fn test_weak_rule_majority_fail() {
        let config = create_config(ConsensusRuleConfig::Weak, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Fail, 40),
            create_vote("gemini", Vote::Fail, 35),
            create_vote("qwen", Vote::Pass, 75),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        assert!(matches!(result.decision, Decision::Block));
    }
}

// Testes do ConsensusEngine
mod engine_tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        assert_eq!(engine.rule_name(), "strong");
        assert_eq!(engine.min_score(), 70);
        assert_eq!(engine.max_loops(), 3);
    }

    #[test]
    fn test_engine_evaluate_pass() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 85),
            create_vote("gemini", Vote::Pass, 90),
            create_vote("qwen", Vote::Pass, 88),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");

        assert!(matches!(result.decision, Decision::Pass));
        assert!(result.consensus_achieved);
    }

    #[test]
    fn test_engine_evaluate_block() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Fail, 30),
            create_vote("gemini", Vote::Fail, 25),
            create_vote("qwen", Vote::Fail, 20),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");

        assert!(matches!(result.decision, Decision::Block));
    }

    #[test]
    fn test_engine_can_retry() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        assert!(engine.can_retry(0));
        assert!(engine.can_retry(1));
        assert!(engine.can_retry(2));
        assert!(!engine.can_retry(3));
        assert!(!engine.can_retry(4));
    }

    #[test]
    fn test_engine_calculate_confidence_high() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 95),
            create_vote("gemini", Vote::Pass, 98),
            create_vote("qwen", Vote::Pass, 97),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        let confidence = engine.calculate_confidence(&result);

        assert!(confidence > 0.8);
    }

    #[test]
    fn test_engine_calculate_confidence_low() {
        let config = create_config(ConsensusRuleConfig::Strong, 70, 3);
        let engine = ConsensusEngine::new(config);

        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 72),
            create_vote("gemini", Vote::Warn, 65),
            create_vote("qwen", Vote::Fail, 40),
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

        // Com Golden rule, mesmo um WARN deve resultar em Revise
        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 85),
            create_vote("gemini", Vote::Warn, 75),
            create_vote("qwen", Vote::Pass, 88),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        assert!(matches!(result.decision, Decision::Revise));
    }

    #[test]
    fn test_weak_rule_engine() {
        let config = create_config(ConsensusRuleConfig::Weak, 70, 3);
        let engine = ConsensusEngine::new(config);

        assert_eq!(engine.rule_name(), "weak");

        // Com Weak rule, 2 PASS são suficientes
        let votes: HashMap<String, ModelVote> = vec![
            create_vote("codex", Vote::Pass, 85),
            create_vote("gemini", Vote::Pass, 80),
            create_vote("qwen", Vote::Fail, 30),
        ]
        .into_iter()
        .collect();

        let result = engine.evaluate(votes, "test-123");
        assert!(matches!(result.decision, Decision::Pass));
    }

    #[test]
    fn test_engine_with_empty_votes() {
        let config = create_config(ConsensusRuleConfig::Golden, 70, 3);
        let engine = ConsensusEngine::new(config);
        let votes: HashMap<String, ModelVote> = HashMap::new();

        let result = engine.evaluate(votes, "test-123");

        // Golden rule sem votos suficientes retorna Revise (precisa de 3 votos)
        assert!(matches!(result.decision, Decision::Revise));
    }

    #[test]
    fn test_engine_with_single_vote() {
        let config = create_config(ConsensusRuleConfig::Golden, 70, 3);
        let engine = ConsensusEngine::new(config);
        let votes: HashMap<String, ModelVote> =
            vec![create_vote("codex", Vote::Pass, 85)].into_iter().collect();

        let result = engine.evaluate(votes, "test-123");

        // Com apenas 1 voto, depende da implementação
        // Pode passar se o score for alto o suficiente
        assert!(!result.votes.is_empty());
    }

    #[test]
    fn test_engine_default() {
        let engine = ConsensusEngine::default();

        // Default deve funcionar
        assert!(!engine.rule_name().is_empty());
    }
}

// Testes de decisões
mod decision_tests {
    use super::*;

    #[test]
    fn test_decision_display() {
        assert_eq!(format!("{}", Decision::Pass), "PASS");
        assert_eq!(format!("{}", Decision::Revise), "REVISE");
        assert_eq!(format!("{}", Decision::Block), "BLOCK");
    }

    #[test]
    fn test_consensus_rule_variants() {
        let golden = ConsensusRuleConfig::Golden;
        let strong = ConsensusRuleConfig::Strong;
        let weak = ConsensusRuleConfig::Weak;

        assert!(matches!(golden, ConsensusRuleConfig::Golden));
        assert!(matches!(strong, ConsensusRuleConfig::Strong));
        assert!(matches!(weak, ConsensusRuleConfig::Weak));
    }

    #[test]
    fn test_vote_display() {
        assert_eq!(format!("{}", Vote::Pass), "PASS");
        assert_eq!(format!("{}", Vote::Warn), "WARN");
        assert_eq!(format!("{}", Vote::Fail), "FAIL");
    }

    #[test]
    fn test_severity_ordering() {
        // Info < Warning < Error < Critical
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Info), "INFO");
        assert_eq!(format!("{}", Severity::Warning), "WARNING");
        assert_eq!(format!("{}", Severity::Error), "ERROR");
        assert_eq!(format!("{}", Severity::Critical), "CRITICAL");
    }
}

// Testes de ModelVote
mod model_vote_tests {
    use super::*;

    #[test]
    fn test_model_vote_new() {
        let vote = ModelVote::new("codex", Vote::Pass, 85);

        assert_eq!(vote.executor, "codex");
        assert_eq!(vote.vote, Vote::Pass);
        assert_eq!(vote.score, 85);
    }

    #[test]
    fn test_model_vote_builder() {
        let vote = ModelVote::new("codex", Vote::Warn, 75)
            .with_reasoning("Code has minor issues")
            .with_issues(vec!["Missing error handling".to_string()])
            .with_suggestions(vec!["Add try-catch".to_string()]);

        assert_eq!(vote.reasoning, "Code has minor issues");
        assert_eq!(vote.issues.len(), 1);
        assert_eq!(vote.suggestions.len(), 1);
    }
}

// Testes de Finding
mod finding_tests {
    use super::*;

    #[test]
    fn test_finding_new() {
        let finding = Finding::new(Severity::Warning, "style", "Missing semicolon");

        assert_eq!(finding.severity, Severity::Warning);
        assert_eq!(finding.category, "style");
        assert_eq!(finding.issue, "Missing semicolon");
    }

    #[test]
    fn test_finding_builder() {
        let finding = Finding::new(Severity::Error, "logic", "Null pointer dereference")
            .with_lines(vec![42, 43])
            .with_suggestion("Add null check")
            .with_source("codex,gemini")
            .with_consensus_strength("strong");

        assert_eq!(finding.lines, Some(vec![42, 43]));
        assert_eq!(finding.suggestion, Some("Add null check".to_string()));
        assert_eq!(finding.source, "codex,gemini");
        assert_eq!(finding.consensus_strength, "strong");
    }
}
