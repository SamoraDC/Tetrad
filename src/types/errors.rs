//! Error types for Tetrad.

use thiserror::Error;

/// Default result type for Tetrad.
pub type TetradResult<T> = Result<T, TetradError>;

/// Possible errors in Tetrad.
#[derive(Error, Debug)]
pub enum TetradError {
    #[cfg(feature = "sqlite")]
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Executor '{0}' not found or not available")]
    ExecutorNotFound(String),

    #[error("Executor '{0}' failed: {1}")]
    ExecutorFailed(String, String),

    #[error("Timeout executing '{0}'")]
    ExecutorTimeout(String),

    #[error("Consensus not reached: {0}")]
    ConsensusNotReached(String),

    #[error("ReasoningBank error: {0}")]
    ReasoningBank(String),

    #[error("MCP server error: {0}")]
    McpServer(String),

    #[error("Configuration not found at: {0}")]
    ConfigNotFound(String),

    #[error("{0}")]
    Other(String),

    #[cfg(feature = "cli")]
    #[error("Interactive input error: {0}")]
    Dialoguer(String),
}

#[cfg(feature = "cli")]
impl From<dialoguer::Error> for TetradError {
    fn from(e: dialoguer::Error) -> Self {
        TetradError::Dialoguer(e.to_string())
    }
}

impl TetradError {
    /// Creates a generic error.
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }

    /// Creates a configuration error.
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }
}
