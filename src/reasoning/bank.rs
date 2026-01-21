//! ReasoningBank - Sistema de aprendizado contínuo.
//!
//! Implementa o ciclo RETRIEVE → JUDGE → DISTILL → CONSOLIDATE
//! para aprender com cada avaliação e melhorar ao longo do tempo.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::types::config::ReasoningConfig;
use crate::types::responses::EvaluationResult;
use crate::TetradResult;

use super::patterns::PatternMatcher;

/// ReasoningBank - Sistema de aprendizado contínuo.
pub struct ReasoningBank {
    pub(crate) conn: Connection,
    config: ReasoningConfig,
}

/// Tipo de pattern.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    /// Código que sempre falha.
    AntiPattern,
    /// Código que sempre passa.
    GoodPattern,
    /// Resultados mistos.
    Ambiguous,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternType::AntiPattern => write!(f, "anti_pattern"),
            PatternType::GoodPattern => write!(f, "good_pattern"),
            PatternType::Ambiguous => write!(f, "ambiguous"),
        }
    }
}

impl PatternType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "anti_pattern" | "antipattern" => PatternType::AntiPattern,
            "good_pattern" | "goodpattern" => PatternType::GoodPattern,
            _ => PatternType::Ambiguous,
        }
    }
}

/// Um pattern aprendido pelo ReasoningBank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: i64,
    pub pattern_type: PatternType,
    pub code_signature: String,
    pub language: String,
    pub issue_category: String,
    pub description: String,
    pub solution: Option<String>,
    pub success_count: i32,
    pub failure_count: i32,
    pub confidence: f64,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Tipo de match ao buscar patterns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// Match exato por assinatura.
    Exact,
    /// Match por keyword.
    Keyword,
}

/// Um pattern encontrado em uma busca.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern: Pattern,
    pub match_type: MatchType,
    pub relevance: f64,
}

/// Resultado de um julgamento.
#[derive(Debug, Clone)]
pub struct JudgmentResult {
    pub was_successful: bool,
    pub patterns_updated: usize,
    pub new_patterns_created: usize,
}

/// Conhecimento destilado do banco.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistilledKnowledge {
    pub top_antipatterns: Vec<Pattern>,
    pub top_good_patterns: Vec<Pattern>,
    pub problematic_categories: HashMap<String, usize>,
    pub language_stats: HashMap<String, LanguageStats>,
    pub avg_loops_to_consensus: f64,
    pub total_patterns: usize,
    pub total_trajectories: usize,
}

/// Estatísticas por linguagem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub total_evaluations: usize,
    pub success_rate: f64,
    pub avg_score: f64,
}

/// Resultado de uma consolidação.
#[derive(Debug, Clone)]
pub struct ConsolidationResult {
    pub patterns_merged: usize,
    pub patterns_pruned: usize,
    pub patterns_reinforced: usize,
}

impl ReasoningBank {
    /// Cria ou abre o banco de patterns.
    pub fn new(db_path: &Path) -> TetradResult<Self> {
        let conn = Connection::open(db_path)?;

        // Cria as tabelas se não existirem
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern_type TEXT NOT NULL,
                code_signature TEXT NOT NULL,
                language TEXT NOT NULL,
                issue_category TEXT NOT NULL,
                description TEXT NOT NULL,
                solution TEXT,
                success_count INTEGER DEFAULT 0,
                failure_count INTEGER DEFAULT 0,
                confidence REAL DEFAULT 0.5,
                last_seen TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(code_signature, issue_category)
            );

            CREATE TABLE IF NOT EXISTS trajectories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern_id INTEGER REFERENCES patterns(id),
                request_id TEXT NOT NULL,
                code_hash TEXT NOT NULL,
                initial_score INTEGER,
                final_score INTEGER,
                loops_to_consensus INTEGER,
                was_successful BOOLEAN,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_patterns_signature ON patterns(code_signature);
            CREATE INDEX IF NOT EXISTS idx_patterns_category ON patterns(issue_category);
            CREATE INDEX IF NOT EXISTS idx_patterns_type ON patterns(pattern_type);
            CREATE INDEX IF NOT EXISTS idx_trajectories_pattern ON trajectories(pattern_id);
        "#,
        )?;

        Ok(Self {
            conn,
            config: ReasoningConfig::default(),
        })
    }

    /// Cria banco com configuração específica.
    pub fn with_config(db_path: &Path, config: ReasoningConfig) -> TetradResult<Self> {
        let mut bank = Self::new(db_path)?;
        bank.config = config;
        Ok(bank)
    }

    /// Cria banco com configuração por referência.
    pub fn new_with_config(db_path: &Path, config: &ReasoningConfig) -> TetradResult<Self> {
        let mut bank = Self::new(db_path)?;
        bank.config = config.clone();
        Ok(bank)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FASE 1: RETRIEVE - Busca patterns similares
    // ═══════════════════════════════════════════════════════════════════════

    /// Busca patterns conhecidos que podem afetar a avaliação.
    pub fn retrieve(&self, code: &str, language: &str) -> Vec<PatternMatch> {
        let signature = PatternMatcher::compute_signature(code);
        let keywords = PatternMatcher::extract_keywords(code);

        let mut matches = Vec::new();

        // Busca por assinatura exata
        if let Ok(exact) = self.find_by_signature(&signature) {
            matches.extend(exact.into_iter().map(|p| PatternMatch {
                pattern: p,
                match_type: MatchType::Exact,
                relevance: 1.0,
            }));
        }

        // Busca por keywords
        for keyword in &keywords {
            if let Ok(keyword_matches) = self.find_by_keyword(keyword, language) {
                matches.extend(keyword_matches.into_iter().map(|p| PatternMatch {
                    relevance: 0.7,
                    pattern: p,
                    match_type: MatchType::Keyword,
                }));
            }
        }

        // Remove duplicatas por ID
        let mut seen_ids = std::collections::HashSet::new();
        matches.retain(|m| seen_ids.insert(m.pattern.id));

        // Ordena por relevância * confidence
        matches.sort_by(|a, b| {
            let score_a = a.relevance * a.pattern.confidence;
            let score_b = b.relevance * b.pattern.confidence;
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Retorna top N matches
        matches.truncate(self.config.max_patterns_per_query);
        matches
    }

    fn find_by_signature(&self, signature: &str) -> TetradResult<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern_type, code_signature, language, issue_category,
                    description, solution, success_count, failure_count, confidence,
                    last_seen, created_at
             FROM patterns WHERE code_signature = ?",
        )?;

        let patterns = stmt
            .query_map(params![signature], |row| {
                Ok(Pattern {
                    id: row.get(0)?,
                    pattern_type: PatternType::from_str(&row.get::<_, String>(1)?),
                    code_signature: row.get(2)?,
                    language: row.get(3)?,
                    issue_category: row.get(4)?,
                    description: row.get(5)?,
                    solution: row.get(6)?,
                    success_count: row.get(7)?,
                    failure_count: row.get(8)?,
                    confidence: row.get(9)?,
                    last_seen: row
                        .get::<_, String>(10)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    created_at: row
                        .get::<_, String>(11)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(patterns)
    }

    fn find_by_keyword(&self, keyword: &str, language: &str) -> TetradResult<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern_type, code_signature, language, issue_category,
                    description, solution, success_count, failure_count, confidence,
                    last_seen, created_at
             FROM patterns
             WHERE (language = ? OR language = 'any')
               AND (issue_category LIKE ? OR description LIKE ?)
             ORDER BY confidence DESC
             LIMIT 10",
        )?;

        let keyword_pattern = format!("%{}%", keyword);

        let patterns = stmt
            .query_map(
                params![language, &keyword_pattern, &keyword_pattern],
                |row| {
                    Ok(Pattern {
                        id: row.get(0)?,
                        pattern_type: PatternType::from_str(&row.get::<_, String>(1)?),
                        code_signature: row.get(2)?,
                        language: row.get(3)?,
                        issue_category: row.get(4)?,
                        description: row.get(5)?,
                        solution: row.get(6)?,
                        success_count: row.get(7)?,
                        failure_count: row.get(8)?,
                        confidence: row.get(9)?,
                        last_seen: row
                            .get::<_, String>(10)?
                            .parse()
                            .unwrap_or_else(|_| Utc::now()),
                        created_at: row
                            .get::<_, String>(11)?
                            .parse()
                            .unwrap_or_else(|_| Utc::now()),
                    })
                },
            )?
            .filter_map(|r| r.ok())
            .collect();

        Ok(patterns)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FASE 2: JUDGE - Avalia sucesso/falha da avaliação
    // ═══════════════════════════════════════════════════════════════════════

    /// Julga o resultado de uma avaliação e atualiza patterns.
    ///
    /// # Parâmetros
    /// - `request_id`: ID único da requisição
    /// - `code`: Código fonte avaliado
    /// - `language`: Linguagem do código
    /// - `result`: Resultado da avaliação
    /// - `loops_to_consensus`: Número de loops até consenso
    /// - `max_loops`: Número máximo de loops permitidos pela configuração
    pub fn judge(
        &mut self,
        request_id: &str,
        code: &str,
        language: &str,
        result: &EvaluationResult,
        loops_to_consensus: u32,
        max_loops: u8,
    ) -> TetradResult<JudgmentResult> {
        let signature = PatternMatcher::compute_signature(code);
        // Sucesso = consenso alcançado dentro do limite de loops permitido
        let was_successful = result.consensus_achieved && loops_to_consensus <= max_loops as u32;

        let initial_score = result.votes.values().map(|v| v.score).min().unwrap_or(0);

        // Registra trajetória
        self.save_trajectory(
            request_id,
            &signature,
            initial_score,
            result.score,
            loops_to_consensus,
            was_successful,
        )?;

        let mut patterns_updated = 0;
        let mut new_patterns_created = 0;

        // Para cada finding, atualiza ou cria pattern
        for finding in &result.findings {
            let created = self.update_or_create_pattern(
                &signature,
                language,
                &finding.issue,
                finding.suggestion.as_deref(),
                &finding.category,
                was_successful,
            )?;

            if created {
                new_patterns_created += 1;
            } else {
                patterns_updated += 1;
            }
        }

        // Se não houve findings e foi sucesso, registra como GoodPattern
        if result.findings.is_empty() && was_successful {
            self.register_good_pattern(&signature, language)?;
            new_patterns_created += 1;
        }

        Ok(JudgmentResult {
            was_successful,
            patterns_updated,
            new_patterns_created,
        })
    }

    fn save_trajectory(
        &self,
        request_id: &str,
        code_hash: &str,
        initial_score: u8,
        final_score: u8,
        loops_to_consensus: u32,
        was_successful: bool,
    ) -> TetradResult<()> {
        self.conn.execute(
            "INSERT INTO trajectories (pattern_id, request_id, code_hash, initial_score,
                                       final_score, loops_to_consensus, was_successful, timestamp)
             VALUES (NULL, ?, ?, ?, ?, ?, ?, ?)",
            params![
                request_id,
                code_hash,
                initial_score as i32,
                final_score as i32,
                loops_to_consensus as i32,
                was_successful,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    fn update_or_create_pattern(
        &mut self,
        signature: &str,
        language: &str,
        issue: &str,
        solution: Option<&str>,
        category: &str,
        was_successful: bool,
    ) -> TetradResult<bool> {
        let now = Utc::now().to_rfc3339();

        // Tenta atualizar existente
        let updated = self.conn.execute(
            "UPDATE patterns
             SET success_count = success_count + ?,
                 failure_count = failure_count + ?,
                 last_seen = ?,
                 confidence = CAST(success_count + ? AS REAL) / (success_count + failure_count + 1)
             WHERE code_signature = ? AND issue_category = ?",
            params![
                if was_successful { 1 } else { 0 },
                if was_successful { 0 } else { 1 },
                &now,
                if was_successful { 1 } else { 0 },
                signature,
                category
            ],
        )?;

        if updated == 0 {
            // Cria novo pattern
            let pattern_type = if was_successful {
                PatternType::Ambiguous
            } else {
                PatternType::AntiPattern
            };

            self.conn.execute(
                "INSERT INTO patterns (pattern_type, code_signature, language, issue_category,
                                       description, solution, success_count, failure_count,
                                       confidence, last_seen, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0.5, ?, ?)",
                params![
                    pattern_type.to_string(),
                    signature,
                    language,
                    category,
                    issue,
                    solution,
                    if was_successful { 1 } else { 0 },
                    if was_successful { 0 } else { 1 },
                    &now,
                    &now
                ],
            )?;
            return Ok(true);
        }

        Ok(false)
    }

    fn register_good_pattern(&mut self, signature: &str, language: &str) -> TetradResult<()> {
        let now = Utc::now().to_rfc3339();

        // Tenta atualizar existente
        let updated = self.conn.execute(
            "UPDATE patterns
             SET success_count = success_count + 1,
                 pattern_type = 'good_pattern',
                 last_seen = ?,
                 confidence = CAST(success_count + 1 AS REAL) / (success_count + failure_count + 1)
             WHERE code_signature = ? AND issue_category = 'success'",
            params![&now, signature],
        )?;

        if updated == 0 {
            self.conn.execute(
                "INSERT INTO patterns (pattern_type, code_signature, language, issue_category,
                                       description, solution, success_count, failure_count,
                                       confidence, last_seen, created_at)
                 VALUES ('good_pattern', ?, ?, 'success', 'Código aprovado sem issues', NULL, 1, 0, 1.0, ?, ?)",
                params![signature, language, &now, &now],
            )?;
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FASE 3: DISTILL - Extrai learnings dos patterns
    // ═══════════════════════════════════════════════════════════════════════

    /// Destila conhecimento dos patterns para gerar insights.
    pub fn distill(&self) -> DistilledKnowledge {
        let top_antipatterns = self
            .get_top_patterns(PatternType::AntiPattern, 10)
            .unwrap_or_default();
        let top_good_patterns = self
            .get_top_patterns(PatternType::GoodPattern, 10)
            .unwrap_or_default();
        let problematic_categories = self.get_problematic_categories().unwrap_or_default();
        let language_stats = self.get_language_stats().unwrap_or_default();
        let avg_loops = self.get_average_loops_to_consensus().unwrap_or(0.0);

        DistilledKnowledge {
            top_antipatterns,
            top_good_patterns,
            problematic_categories,
            language_stats,
            avg_loops_to_consensus: avg_loops,
            total_patterns: self.count_patterns().unwrap_or(0),
            total_trajectories: self.count_trajectories().unwrap_or(0),
        }
    }

    fn get_top_patterns(
        &self,
        pattern_type: PatternType,
        limit: usize,
    ) -> TetradResult<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern_type, code_signature, language, issue_category,
                    description, solution, success_count, failure_count, confidence,
                    last_seen, created_at
             FROM patterns
             WHERE pattern_type = ?
             ORDER BY (success_count + failure_count) DESC, confidence DESC
             LIMIT ?",
        )?;

        let patterns = stmt
            .query_map(params![pattern_type.to_string(), limit as i32], |row| {
                Ok(Pattern {
                    id: row.get(0)?,
                    pattern_type: PatternType::from_str(&row.get::<_, String>(1)?),
                    code_signature: row.get(2)?,
                    language: row.get(3)?,
                    issue_category: row.get(4)?,
                    description: row.get(5)?,
                    solution: row.get(6)?,
                    success_count: row.get(7)?,
                    failure_count: row.get(8)?,
                    confidence: row.get(9)?,
                    last_seen: row
                        .get::<_, String>(10)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    created_at: row
                        .get::<_, String>(11)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(patterns)
    }

    fn get_problematic_categories(&self) -> TetradResult<HashMap<String, usize>> {
        let mut stmt = self.conn.prepare(
            "SELECT issue_category, COUNT(*) as count
             FROM patterns
             WHERE pattern_type = 'anti_pattern'
             GROUP BY issue_category
             ORDER BY count DESC",
        )?;

        let categories: HashMap<String, usize> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(categories)
    }

    fn get_language_stats(&self) -> TetradResult<HashMap<String, LanguageStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT language,
                    COUNT(*) as total,
                    AVG(CASE WHEN pattern_type = 'good_pattern' THEN 1.0 ELSE 0.0 END) as success_rate,
                    AVG(confidence * 100) as avg_score
             FROM patterns
             GROUP BY language",
        )?;

        let stats: HashMap<String, LanguageStats> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    LanguageStats {
                        total_evaluations: row.get::<_, usize>(1)?,
                        success_rate: row.get::<_, f64>(2)?,
                        avg_score: row.get::<_, f64>(3)?,
                    },
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(stats)
    }

    fn get_average_loops_to_consensus(&self) -> TetradResult<f64> {
        let avg: f64 = self
            .conn
            .query_row(
                "SELECT AVG(loops_to_consensus) FROM trajectories WHERE was_successful = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        Ok(avg)
    }

    fn count_patterns(&self) -> TetradResult<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM patterns", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Conta o número total de trajetórias.
    pub fn count_trajectories(&self) -> TetradResult<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM trajectories", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FASE 4: CONSOLIDATE - Previne esquecimento de patterns importantes
    // ═══════════════════════════════════════════════════════════════════════

    /// Consolida conhecimento, prevenindo esquecimento de patterns importantes.
    pub fn consolidate(&mut self) -> TetradResult<ConsolidationResult> {
        let merged = self.merge_similar_patterns()?;
        let pruned = self.prune_low_quality_patterns()?;
        let reinforced = self.reinforce_high_value_patterns()?;
        self.recalculate_all_confidences()?;

        Ok(ConsolidationResult {
            patterns_merged: merged,
            patterns_pruned: pruned,
            patterns_reinforced: reinforced,
        })
    }

    fn merge_similar_patterns(&mut self) -> TetradResult<usize> {
        // Encontra patterns com mesma categoria e assinatura similar
        let mut merged = 0;

        // Por enquanto, merge apenas duplicatas exatas
        let duplicates: Vec<(i64, i64)> = self
            .conn
            .prepare(
                "SELECT p1.id, p2.id
                 FROM patterns p1
                 JOIN patterns p2 ON p1.code_signature = p2.code_signature
                                  AND p1.issue_category = p2.issue_category
                                  AND p1.id < p2.id",
            )?
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        for (keep_id, remove_id) in duplicates {
            // Soma os counts do pattern removido ao mantido
            self.conn.execute(
                "UPDATE patterns
                 SET success_count = success_count + (SELECT success_count FROM patterns WHERE id = ?),
                     failure_count = failure_count + (SELECT failure_count FROM patterns WHERE id = ?)
                 WHERE id = ?",
                params![remove_id, remove_id, keep_id],
            )?;

            // Remove o duplicado
            self.conn
                .execute("DELETE FROM patterns WHERE id = ?", params![remove_id])?;
            merged += 1;
        }

        Ok(merged)
    }

    fn prune_low_quality_patterns(&mut self) -> TetradResult<usize> {
        // Remove patterns com baixa confiança e pouco uso (< 3 ocorrências)
        // Nota: created_at está em formato RFC3339 (ex: 2024-01-15T10:30:00+00:00),
        // então usamos strftime para gerar comparação compatível
        let pruned = self.conn.execute(
            "DELETE FROM patterns
             WHERE confidence < 0.3
               AND (success_count + failure_count) < 3
               AND created_at < strftime('%Y-%m-%dT%H:%M:%S+00:00', datetime('now', '-30 days'))",
            [],
        )?;

        Ok(pruned)
    }

    fn reinforce_high_value_patterns(&mut self) -> TetradResult<usize> {
        // Aumenta ligeiramente a confiança de patterns muito usados
        let reinforced = self.conn.execute(
            "UPDATE patterns
             SET confidence = MIN(confidence * 1.05, 1.0)
             WHERE (success_count + failure_count) > 10
               AND confidence > 0.7",
            [],
        )?;

        Ok(reinforced)
    }

    fn recalculate_all_confidences(&mut self) -> TetradResult<()> {
        self.conn.execute(
            "UPDATE patterns
             SET confidence = CASE
                 WHEN (success_count + failure_count) = 0 THEN 0.5
                 ELSE CAST(success_count AS REAL) / (success_count + failure_count)
             END,
             pattern_type = CASE
                 WHEN CAST(success_count AS REAL) / (success_count + failure_count + 0.001) > 0.8 THEN 'good_pattern'
                 WHEN CAST(failure_count AS REAL) / (success_count + failure_count + 0.001) > 0.8 THEN 'anti_pattern'
                 ELSE 'ambiguous'
             END",
            [],
        )?;

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Métodos auxiliares públicos
    // ═══════════════════════════════════════════════════════════════════════

    /// Retorna todos os patterns.
    pub fn get_all_patterns(&self) -> TetradResult<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern_type, code_signature, language, issue_category,
                    description, solution, success_count, failure_count, confidence,
                    last_seen, created_at
             FROM patterns
             ORDER BY (success_count + failure_count) DESC",
        )?;

        let patterns = stmt
            .query_map([], |row| {
                Ok(Pattern {
                    id: row.get(0)?,
                    pattern_type: PatternType::from_str(&row.get::<_, String>(1)?),
                    code_signature: row.get(2)?,
                    language: row.get(3)?,
                    issue_category: row.get(4)?,
                    description: row.get(5)?,
                    solution: row.get(6)?,
                    success_count: row.get(7)?,
                    failure_count: row.get(8)?,
                    confidence: row.get(9)?,
                    last_seen: row
                        .get::<_, String>(10)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    created_at: row
                        .get::<_, String>(11)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(patterns)
    }

    /// Verifica se um pattern existe.
    pub fn pattern_exists(&self, signature: &str, category: &str) -> TetradResult<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM patterns WHERE code_signature = ? AND issue_category = ?",
            params![signature, category],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::responses::{Decision, Finding};
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn create_test_bank() -> (ReasoningBank, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let bank = ReasoningBank::new(&db_path).unwrap();
        (bank, dir)
    }

    fn create_test_result(
        decision: Decision,
        score: u8,
        findings: Vec<Finding>,
    ) -> EvaluationResult {
        EvaluationResult {
            request_id: "test-123".to_string(),
            decision,
            score,
            consensus_achieved: decision == Decision::Pass,
            votes: HashMap::new(),
            findings,
            feedback: String::new(),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_create_bank() {
        let (bank, _dir) = create_test_bank();
        assert_eq!(bank.count_patterns().unwrap(), 0);
        assert_eq!(bank.count_trajectories().unwrap(), 0);
    }

    #[test]
    fn test_retrieve_empty() {
        let (bank, _dir) = create_test_bank();
        let matches = bank.retrieve("fn main() {}", "rust");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_judge_creates_pattern() {
        let (mut bank, _dir) = create_test_bank();

        let finding = Finding::new(
            crate::types::responses::Severity::Warning,
            "security",
            "SQL injection vulnerability",
        );

        let result = create_test_result(Decision::Revise, 60, vec![finding]);

        let judgment = bank
            .judge("test-123", "SELECT * FROM users", "sql", &result, 3, 3)
            .unwrap();

        assert!(!judgment.was_successful);
        assert_eq!(judgment.new_patterns_created, 1);
    }

    #[test]
    fn test_retrieve_after_judge() {
        let (mut bank, _dir) = create_test_bank();

        let finding = Finding::new(
            crate::types::responses::Severity::Warning,
            "security",
            "SQL injection",
        );

        let result = create_test_result(Decision::Revise, 60, vec![finding]);

        bank.judge("test-123", "SELECT * FROM users", "sql", &result, 3, 3)
            .unwrap();

        let matches = bank.retrieve("SELECT * FROM users", "sql");
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_good_pattern_creation() {
        let (mut bank, _dir) = create_test_bank();

        let result = create_test_result(Decision::Pass, 95, vec![]);

        let judgment = bank
            .judge(
                "test-123",
                "fn main() { println!(\"Hello\"); }",
                "rust",
                &result,
                1,
                3,
            )
            .unwrap();

        assert!(judgment.was_successful);
        assert_eq!(judgment.new_patterns_created, 1);
    }

    #[test]
    fn test_distill() {
        let (mut bank, _dir) = create_test_bank();

        // Adiciona alguns patterns
        let finding = Finding::new(
            crate::types::responses::Severity::Error,
            "logic",
            "Null pointer",
        );

        let result = create_test_result(Decision::Block, 30, vec![finding]);
        bank.judge("test-1", "data.unwrap()", "rust", &result, 5, 3)
            .unwrap();

        let result2 = create_test_result(Decision::Pass, 95, vec![]);
        bank.judge("test-2", "fn safe() {}", "rust", &result2, 1, 3)
            .unwrap();

        let knowledge = bank.distill();
        assert!(knowledge.total_patterns > 0);
        assert!(knowledge.total_trajectories > 0);
    }

    #[test]
    fn test_consolidate() {
        let (mut bank, _dir) = create_test_bank();

        // Adiciona alguns patterns
        let result = create_test_result(Decision::Pass, 90, vec![]);
        for i in 0..5 {
            bank.judge(
                &format!("test-{}", i),
                "fn good() {}",
                "rust",
                &result,
                1,
                3,
            )
            .unwrap();
        }

        let consolidation = bank.consolidate().unwrap();
        // Verifica que a consolidação retornou um resultado válido
        // (patterns_merged é usize, então sempre >= 0)
        let _ = consolidation.patterns_merged;
    }
}
