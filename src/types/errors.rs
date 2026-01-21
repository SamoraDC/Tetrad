//! Tipos de erro do Tetrad.

use thiserror::Error;

/// Tipo de resultado padrão do Tetrad.
pub type TetradResult<T> = Result<T, TetradError>;

/// Erros possíveis no Tetrad.
#[derive(Error, Debug)]
pub enum TetradError {
    #[error("Erro de configuração: {0}")]
    Config(String),

    #[error("Erro de IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro ao parsear TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Erro ao serializar TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Erro de JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Executor '{0}' não encontrado ou não disponível")]
    ExecutorNotFound(String),

    #[error("Executor '{0}' falhou: {1}")]
    ExecutorFailed(String, String),

    #[error("Timeout ao executar '{0}'")]
    ExecutorTimeout(String),

    #[error("Consenso não alcançado: {0}")]
    ConsensusNotReached(String),

    #[error("Erro no ReasoningBank: {0}")]
    ReasoningBank(String),

    #[error("Erro no servidor MCP: {0}")]
    McpServer(String),

    #[error("Configuração não encontrada em: {0}")]
    ConfigNotFound(String),

    #[error("{0}")]
    Other(String),
}

impl TetradError {
    /// Cria um erro genérico.
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }

    /// Cria um erro de configuração.
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }
}
