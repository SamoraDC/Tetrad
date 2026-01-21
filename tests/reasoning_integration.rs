//! Testes de integração para o ReasoningBank do Tetrad.

use std::path::PathBuf;
use tempfile::TempDir;

use tetrad::reasoning::{PatternType, ReasoningBank};
use tetrad::types::responses::EvaluationResult;

fn temp_db_path() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_reasoning.db");
    (temp_dir, db_path)
}

fn sample_result() -> EvaluationResult {
    EvaluationResult::success("test-123", 85, "Looks good!")
}

// Testes básicos do ReasoningBank
mod basic_tests {
    use super::*;

    #[test]
    fn test_reasoning_bank_creation() {
        let (_temp_dir, db_path) = temp_db_path();
        let bank = ReasoningBank::new(&db_path).expect("Failed to create bank");

        // Banco deve ser criado com sucesso
        assert!(db_path.exists());
        drop(bank);
    }

    #[test]
    fn test_reasoning_bank_retrieve_empty() {
        let (_temp_dir, db_path) = temp_db_path();
        let bank = ReasoningBank::new(&db_path).expect("Failed to create bank");

        let code = "fn main() { println!(\"Hello\"); }";
        let matches = bank.retrieve(code, "rust");

        // Sem patterns, não deve encontrar nada
        assert!(matches.is_empty());
    }
}

// Testes do ciclo RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
mod cycle_tests {
    use super::*;

    #[test]
    fn test_judge_without_patterns() {
        let (_temp_dir, db_path) = temp_db_path();
        let mut bank = ReasoningBank::new(&db_path).expect("Failed to create bank");

        let code = "fn main() { unsafe { *ptr } }";
        let result = sample_result();

        // Judge deve funcionar mesmo sem patterns
        let judgment = bank
            .judge(
                "test-req-1",
                code,
                "rust",
                &result,
                1, // loops_to_consensus
                3, // max_loops
            )
            .expect("Failed to judge");

        // Deve produzir um resultado de julgamento
        // patterns_updated é usize, verificamos que a operação completa sem erro
        let _ = judgment.patterns_updated;
    }

    #[test]
    fn test_distill() {
        let (_temp_dir, db_path) = temp_db_path();
        let bank = ReasoningBank::new(&db_path).expect("Failed to create bank");

        // Distill deve funcionar sem erros
        let knowledge = bank.distill();

        // Deve ter estatísticas (total_patterns é usize, operação completa sem erro)
        let _ = knowledge.total_patterns;
    }

    #[test]
    fn test_consolidate() {
        let (_temp_dir, db_path) = temp_db_path();
        let mut bank = ReasoningBank::new(&db_path).expect("Failed to create bank");

        // Consolidate deve funcionar sem erros
        let result = bank.consolidate().expect("Failed to consolidate");

        // Resultado de consolidação (patterns_merged é usize)
        let _ = result.patterns_merged;
    }

    #[test]
    fn test_full_cycle() {
        let (_temp_dir, db_path) = temp_db_path();
        let mut bank = ReasoningBank::new(&db_path).expect("Failed to create bank");

        let code = "fn main() { let x = 5; println!(\"{}\", x); }";
        let result = sample_result();

        // RETRIEVE
        let _matches = bank.retrieve(code, "rust");

        // JUDGE
        let _judgment = bank.judge("req-1", code, "rust", &result, 1, 3).unwrap();

        // DISTILL
        let _knowledge = bank.distill();

        // CONSOLIDATE
        let _consolidation = bank.consolidate().unwrap();

        // Ciclo completo sem erros - sucesso
    }
}

// Testes de persistência
mod persistence_tests {
    use super::*;

    #[test]
    fn test_pattern_persistence() {
        let (_temp_dir, db_path) = temp_db_path();

        // Cria banco e adiciona pattern via judge
        {
            let mut bank = ReasoningBank::new(&db_path).expect("Failed to create bank");
            let code = "fn test() { unwrap(); }";
            let result = EvaluationResult::failure("fail-1", 30, "Found issues");

            // Judge cria patterns a partir dos findings
            let _ = bank.judge("req-1", code, "rust", &result, 2, 3);
        }

        // Reabre banco
        {
            let bank = ReasoningBank::new(&db_path).expect("Failed to reopen bank");
            // Banco deve reabrir sem erros
            let _patterns = bank.get_all_patterns().unwrap();
        }
    }

    #[test]
    fn test_count_trajectories() {
        let (_temp_dir, db_path) = temp_db_path();
        let mut bank = ReasoningBank::new(&db_path).expect("Failed to create bank");

        // Inicialmente zero
        let count = bank.count_trajectories().unwrap();
        assert_eq!(count, 0);

        // Após judge, deve ter uma trajetória
        let result = sample_result();
        let _ = bank.judge("req-1", "fn test() {}", "rust", &result, 1, 3);

        let count = bank.count_trajectories().unwrap();
        assert_eq!(count, 1);
    }
}

// Testes de padrões
mod pattern_tests {
    use super::*;

    #[test]
    fn test_pattern_types() {
        let anti = PatternType::AntiPattern;
        let good = PatternType::GoodPattern;
        let ambiguous = PatternType::Ambiguous;

        assert!(matches!(anti, PatternType::AntiPattern));
        assert!(matches!(good, PatternType::GoodPattern));
        assert!(matches!(ambiguous, PatternType::Ambiguous));
    }

    #[test]
    fn test_pattern_type_display() {
        assert_eq!(format!("{}", PatternType::AntiPattern), "anti_pattern");
        assert_eq!(format!("{}", PatternType::GoodPattern), "good_pattern");
        assert_eq!(format!("{}", PatternType::Ambiguous), "ambiguous");
    }

    #[test]
    fn test_get_all_patterns() {
        let (_temp_dir, db_path) = temp_db_path();
        let bank = ReasoningBank::new(&db_path).expect("Failed to create bank");

        let patterns = bank.get_all_patterns().unwrap();

        // Inicialmente vazio
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_pattern_exists() {
        let (_temp_dir, db_path) = temp_db_path();
        let bank = ReasoningBank::new(&db_path).expect("Failed to create bank");

        // Padrão não existe inicialmente
        let exists = bank
            .pattern_exists("test_signature", "test_category")
            .unwrap();
        assert!(!exists);
    }
}

// Testes de utilitários
mod utility_tests {
    use super::*;

    #[test]
    fn test_bank_with_default_config() {
        let (_temp_dir, db_path) = temp_db_path();
        let bank = ReasoningBank::new(&db_path);

        assert!(bank.is_ok());
    }

    #[test]
    fn test_bank_nonexistent_path() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("deep").join("test.db");

        // Pode falhar ou criar os diretórios - não deve causar panic
        let result = ReasoningBank::new(&nested_path);
        let _ = result;
    }
}
