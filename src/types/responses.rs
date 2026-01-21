//! Tipos de resposta do Tetrad.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resultado de uma avaliação.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// ID da requisição.
    pub request_id: String,

    /// Decisão final.
    pub decision: Decision,

    /// Score agregado (0-100).
    pub score: u8,

    /// Se consenso foi alcançado.
    pub consensus_achieved: bool,

    /// Votos de cada executor.
    pub votes: HashMap<String, ModelVote>,

    /// Findings/issues encontrados.
    pub findings: Vec<Finding>,

    /// Feedback consolidado.
    pub feedback: String,

    /// Timestamp da avaliação.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl EvaluationResult {
    /// Cria um resultado de sucesso.
    pub fn success(request_id: impl Into<String>, score: u8, feedback: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            decision: Decision::Pass,
            score,
            consensus_achieved: true,
            votes: HashMap::new(),
            findings: Vec::new(),
            feedback: feedback.into(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Cria um resultado de falha.
    pub fn failure(request_id: impl Into<String>, score: u8, feedback: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            decision: Decision::Block,
            score,
            consensus_achieved: false,
            votes: HashMap::new(),
            findings: Vec::new(),
            feedback: feedback.into(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Decisão final da avaliação.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// Aprovado - pode prosseguir.
    Pass,
    /// Necessita revisão - há issues menores.
    Revise,
    /// Bloqueado - há issues críticos.
    Block,
}

impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decision::Pass => write!(f, "PASS"),
            Decision::Revise => write!(f, "REVISE"),
            Decision::Block => write!(f, "BLOCK"),
        }
    }
}

/// Voto de um modelo/executor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVote {
    /// Nome do executor.
    pub executor: String,

    /// Voto (PASS, WARN, FAIL).
    pub vote: Vote,

    /// Score (0-100).
    pub score: u8,

    /// Justificativa.
    pub reasoning: String,

    /// Issues encontrados.
    pub issues: Vec<String>,

    /// Sugestões de melhoria.
    pub suggestions: Vec<String>,
}

impl ModelVote {
    /// Cria um novo voto.
    pub fn new(executor: impl Into<String>, vote: Vote, score: u8) -> Self {
        Self {
            executor: executor.into(),
            vote,
            score,
            reasoning: String::new(),
            issues: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Adiciona reasoning.
    pub fn with_reasoning(mut self, reasoning: impl Into<String>) -> Self {
        self.reasoning = reasoning.into();
        self
    }

    /// Adiciona issues.
    pub fn with_issues(mut self, issues: Vec<String>) -> Self {
        self.issues = issues;
        self
    }

    /// Adiciona sugestões.
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }
}

/// Voto individual.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Vote {
    /// Aprovado.
    Pass,
    /// Aviso - issues menores.
    Warn,
    /// Reprovado - issues críticos.
    Fail,
}

impl std::fmt::Display for Vote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Vote::Pass => write!(f, "PASS"),
            Vote::Warn => write!(f, "WARN"),
            Vote::Fail => write!(f, "FAIL"),
        }
    }
}

/// Um finding/issue encontrado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Severidade.
    pub severity: Severity,

    /// Categoria do issue.
    #[serde(default)]
    pub category: String,

    /// Descrição do issue.
    pub issue: String,

    /// Linha(s) afetada(s).
    #[serde(default)]
    pub lines: Option<Vec<u32>>,

    /// Sugestão de correção.
    #[serde(default)]
    pub suggestion: Option<String>,

    /// Fonte do finding (executores que reportaram).
    #[serde(default)]
    pub source: String,

    /// Força do consenso (forte, moderado, fraco).
    #[serde(default)]
    pub consensus_strength: String,
}

impl Finding {
    /// Cria um novo finding.
    pub fn new(severity: Severity, category: impl Into<String>, issue: impl Into<String>) -> Self {
        Self {
            severity,
            category: category.into(),
            issue: issue.into(),
            lines: None,
            suggestion: None,
            source: String::new(),
            consensus_strength: String::new(),
        }
    }

    /// Adiciona linhas afetadas.
    pub fn with_lines(mut self, lines: Vec<u32>) -> Self {
        self.lines = Some(lines);
        self
    }

    /// Adiciona sugestão.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Adiciona fonte.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Adiciona força do consenso.
    pub fn with_consensus_strength(mut self, strength: impl Into<String>) -> Self {
        self.consensus_strength = strength.into();
        self
    }
}

/// Severidade de um finding.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Informação.
    Info,
    /// Aviso.
    Warning,
    /// Erro.
    Error,
    /// Crítico.
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}
