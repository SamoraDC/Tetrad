//! Export/Import de patterns do ReasoningBank.
//!
//! Permite compartilhar conhecimento entre diferentes instalações do Tetrad.

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::TetradResult;

use super::bank::{DistilledKnowledge, Pattern, ReasoningBank};

/// Estrutura de exportação do ReasoningBank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningBankExport {
    /// Versão do formato de exportação.
    pub version: String,
    /// Data/hora da exportação.
    pub exported_at: DateTime<Utc>,
    /// Conhecimento destilado.
    pub knowledge: DistilledKnowledge,
    /// Patterns exportados.
    pub patterns: Vec<Pattern>,
}

/// Resultado de uma importação.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// Patterns importados (novos).
    pub imported: usize,
    /// Patterns ignorados (já existentes).
    pub skipped: usize,
    /// Patterns mesclados (atualizados).
    pub merged: usize,
}

impl ReasoningBank {
    /// Exporta ReasoningBank para arquivo JSON.
    pub fn export(&self, path: &Path) -> TetradResult<()> {
        let knowledge = self.distill();
        let patterns = self.get_all_patterns()?;

        let export = ReasoningBankExport {
            version: "2.0".to_string(),
            exported_at: Utc::now(),
            knowledge,
            patterns,
        };

        let json = serde_json::to_string_pretty(&export)?;
        std::fs::write(path, json)?;

        tracing::info!(
            path = %path.display(),
            patterns = export.patterns.len(),
            "ReasoningBank exported"
        );

        Ok(())
    }

    /// Importa patterns de arquivo JSON.
    pub fn import(&mut self, path: &Path) -> TetradResult<ImportResult> {
        let json = std::fs::read_to_string(path)?;
        let export: ReasoningBankExport = serde_json::from_str(&json)?;

        let mut imported = 0;
        let mut skipped = 0;
        let mut merged = 0;

        for pattern in export.patterns {
            if self.pattern_exists(&pattern.code_signature, &pattern.issue_category)? {
                // Pattern já existe - tenta mesclar
                if self.merge_imported_pattern(&pattern)? {
                    merged += 1;
                } else {
                    skipped += 1;
                }
            } else {
                // Pattern novo - importa
                self.insert_pattern(&pattern)?;
                imported += 1;
            }
        }

        tracing::info!(
            path = %path.display(),
            imported,
            skipped,
            merged,
            "ReasoningBank imported"
        );

        Ok(ImportResult {
            imported,
            skipped,
            merged,
        })
    }

    /// Insere um pattern no banco.
    fn insert_pattern(&mut self, pattern: &Pattern) -> TetradResult<()> {
        self.conn.execute(
            "INSERT INTO patterns (pattern_type, code_signature, language, issue_category,
                                   description, solution, success_count, failure_count,
                                   confidence, last_seen, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                pattern.pattern_type.to_string(),
                pattern.code_signature,
                pattern.language,
                pattern.issue_category,
                pattern.description,
                pattern.solution,
                pattern.success_count,
                pattern.failure_count,
                pattern.confidence,
                pattern.last_seen.to_rfc3339(),
                pattern.created_at.to_rfc3339()
            ],
        )?;

        Ok(())
    }

    /// Mescla um pattern importado com um existente.
    fn merge_imported_pattern(&mut self, pattern: &Pattern) -> TetradResult<bool> {
        // Só mescla se o pattern importado for mais recente ou tiver mais dados
        let existing: Option<(i32, i32, String)> = self
            .conn
            .query_row(
                "SELECT success_count, failure_count, last_seen
                 FROM patterns
                 WHERE code_signature = ? AND issue_category = ?",
                rusqlite::params![pattern.code_signature, pattern.issue_category],
                |row: &rusqlite::Row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        if let Some((existing_success, existing_failure, existing_last_seen)) = existing {
            let existing_total = existing_success + existing_failure;
            let imported_total = pattern.success_count + pattern.failure_count;

            // Mescla se o importado tiver mais dados ou for mais recente
            let should_merge = imported_total > existing_total
                || pattern.last_seen.to_rfc3339() > existing_last_seen;

            if should_merge {
                self.conn.execute(
                    "UPDATE patterns
                     SET success_count = success_count + ?,
                         failure_count = failure_count + ?,
                         last_seen = MAX(last_seen, ?),
                         confidence = CAST(success_count + ? AS REAL) / (success_count + failure_count + ? + ?)
                     WHERE code_signature = ? AND issue_category = ?",
                    rusqlite::params![
                        pattern.success_count,
                        pattern.failure_count,
                        pattern.last_seen.to_rfc3339(),
                        pattern.success_count,
                        pattern.success_count,
                        pattern.failure_count,
                        pattern.code_signature,
                        pattern.issue_category
                    ],
                )?;

                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// Formata conhecimento destilado para exibição.
pub fn format_knowledge(knowledge: &DistilledKnowledge) -> String {
    let mut output = String::new();

    output.push_str("# ReasoningBank Knowledge\n\n");

    output.push_str(&format!(
        "**Total Patterns:** {}\n",
        knowledge.total_patterns
    ));
    output.push_str(&format!(
        "**Total Trajectories:** {}\n",
        knowledge.total_trajectories
    ));
    output.push_str(&format!(
        "**Avg Loops to Consensus:** {:.2}\n\n",
        knowledge.avg_loops_to_consensus
    ));

    // Top Anti-patterns
    if !knowledge.top_antipatterns.is_empty() {
        output.push_str("## Top Anti-patterns\n\n");
        for (i, pattern) in knowledge.top_antipatterns.iter().enumerate() {
            output.push_str(&format!(
                "{}. **{}** ({})\n   - Failures: {}\n   - Confidence: {:.0}%\n",
                i + 1,
                pattern.issue_category,
                pattern.language,
                pattern.failure_count,
                pattern.confidence * 100.0
            ));
            if let Some(solution) = &pattern.solution {
                output.push_str(&format!("   - Solution: {}\n", solution));
            }
            output.push('\n');
        }
    }

    // Top Good Patterns
    if !knowledge.top_good_patterns.is_empty() {
        output.push_str("## Top Good Patterns\n\n");
        for (i, pattern) in knowledge.top_good_patterns.iter().enumerate() {
            output.push_str(&format!(
                "{}. **{}** ({})\n   - Successes: {}\n   - Confidence: {:.0}%\n\n",
                i + 1,
                pattern.issue_category,
                pattern.language,
                pattern.success_count,
                pattern.confidence * 100.0
            ));
        }
    }

    // Problematic Categories
    if !knowledge.problematic_categories.is_empty() {
        output.push_str("## Problematic Categories\n\n");
        for (category, count) in &knowledge.problematic_categories {
            output.push_str(&format!("- **{}**: {} patterns\n", category, count));
        }
        output.push('\n');
    }

    // Language Stats
    if !knowledge.language_stats.is_empty() {
        output.push_str("## Language Statistics\n\n");
        for (language, stats) in &knowledge.language_stats {
            output.push_str(&format!(
                "### {}\n- Evaluations: {}\n- Success Rate: {:.0}%\n- Avg Score: {:.1}\n\n",
                language,
                stats.total_evaluations,
                stats.success_rate * 100.0,
                stats.avg_score
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_bank() -> (ReasoningBank, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let bank = ReasoningBank::new(&db_path).unwrap();
        (bank, dir)
    }

    #[test]
    fn test_export_empty_bank() {
        let (bank, dir) = create_test_bank();
        let export_path = dir.path().join("export.json");

        bank.export(&export_path).unwrap();

        assert!(export_path.exists());

        let content = std::fs::read_to_string(&export_path).unwrap();
        let export: ReasoningBankExport = serde_json::from_str(&content).unwrap();

        assert_eq!(export.version, "2.0");
        assert!(export.patterns.is_empty());
    }

    #[test]
    fn test_export_import_roundtrip() {
        use crate::types::responses::{Decision, Finding, Severity};

        let (mut bank1, dir1) = create_test_bank();

        // Adiciona alguns patterns ao banco 1
        let finding = Finding::new(Severity::Warning, "security", "SQL injection");
        let result = crate::types::responses::EvaluationResult {
            request_id: "test".to_string(),
            decision: Decision::Revise,
            score: 60,
            consensus_achieved: false,
            votes: std::collections::HashMap::new(),
            findings: vec![finding],
            feedback: String::new(),
            timestamp: Utc::now(),
        };

        bank1
            .judge("test-1", "SELECT * FROM users", "sql", &result, 3, 3)
            .unwrap();

        // Exporta
        let export_path = dir1.path().join("export.json");
        bank1.export(&export_path).unwrap();

        // Cria novo banco e importa
        let (mut bank2, _dir2) = create_test_bank();
        let import_result = bank2.import(&export_path).unwrap();

        assert_eq!(import_result.imported, 1);
        assert_eq!(import_result.skipped, 0);

        // Verifica que o pattern foi importado
        let patterns = bank2.get_all_patterns().unwrap();
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn test_import_skip_existing() {
        use crate::types::responses::{Decision, Finding, Severity};

        let (mut bank, dir) = create_test_bank();

        // Adiciona um pattern
        let finding = Finding::new(Severity::Warning, "security", "Test issue");
        let result = crate::types::responses::EvaluationResult {
            request_id: "test".to_string(),
            decision: Decision::Revise,
            score: 60,
            consensus_achieved: false,
            votes: std::collections::HashMap::new(),
            findings: vec![finding],
            feedback: String::new(),
            timestamp: Utc::now(),
        };

        bank.judge("test-1", "test code", "rust", &result, 3, 3)
            .unwrap();

        // Exporta
        let export_path = dir.path().join("export.json");
        bank.export(&export_path).unwrap();

        // Tenta importar de volta (deve fazer merge ou skip)
        let import_result = bank.import(&export_path).unwrap();

        // Como o pattern já existe, deve ser merged ou skipped
        assert_eq!(import_result.imported, 0);
        assert!(import_result.skipped > 0 || import_result.merged > 0);
    }

    #[test]
    fn test_format_knowledge() {
        let knowledge = DistilledKnowledge {
            top_antipatterns: vec![],
            top_good_patterns: vec![],
            problematic_categories: std::collections::HashMap::new(),
            language_stats: std::collections::HashMap::new(),
            avg_loops_to_consensus: 2.5,
            total_patterns: 10,
            total_trajectories: 50,
        };

        let formatted = format_knowledge(&knowledge);

        assert!(formatted.contains("**Total Patterns:** 10"));
        assert!(formatted.contains("**Total Trajectories:** 50"));
        assert!(formatted.contains("2.50"));
    }
}
