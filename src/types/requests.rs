//! Tipos de requisição do Tetrad.

use serde::{Deserialize, Serialize};

/// Requisição de avaliação de código.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRequest {
    /// ID único da requisição.
    pub request_id: String,

    /// Código a ser avaliado.
    pub code: String,

    /// Linguagem do código.
    pub language: String,

    /// Tipo de avaliação.
    pub evaluation_type: EvaluationType,

    /// Contexto adicional opcional.
    pub context: Option<String>,

    /// Arquivo de origem (se aplicável).
    pub file_path: Option<String>,
}

impl EvaluationRequest {
    /// Cria uma nova requisição de avaliação.
    pub fn new(code: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            code: code.into(),
            language: language.into(),
            evaluation_type: EvaluationType::Code,
            context: None,
            file_path: None,
        }
    }

    /// Define o tipo de avaliação.
    pub fn with_type(mut self, eval_type: EvaluationType) -> Self {
        self.evaluation_type = eval_type;
        self
    }

    /// Define o contexto.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Define o caminho do arquivo.
    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }
}

/// Tipo de avaliação.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationType {
    /// Avaliação de plano/design.
    Plan,
    /// Avaliação de código.
    Code,
    /// Avaliação de testes.
    Tests,
    /// Verificação final.
    FinalCheck,
}

impl std::fmt::Display for EvaluationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationType::Plan => write!(f, "plan"),
            EvaluationType::Code => write!(f, "code"),
            EvaluationType::Tests => write!(f, "tests"),
            EvaluationType::FinalCheck => write!(f, "final_check"),
        }
    }
}
