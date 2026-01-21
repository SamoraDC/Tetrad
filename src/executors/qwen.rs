//! Executor para Qwen CLI.

use async_trait::async_trait;
use std::time::Duration;
use tokio::process::Command;

use super::base::{CliExecutor, ExecutorResponse};
use crate::types::config::ExecutorConfig;
use crate::types::requests::EvaluationRequest;
use crate::types::responses::{ModelVote, Vote};
use crate::{TetradError, TetradResult};

/// Executor para Qwen CLI (Alibaba).
///
/// Especialização: Bugs lógicos e correção de código.
pub struct QwenExecutor {
    command_name: String,
    args: Vec<String>,
    timeout: Duration,
}

impl QwenExecutor {
    /// Cria um novo executor Qwen com valores padrão.
    pub fn new() -> Self {
        Self {
            command_name: "qwen".to_string(),
            // Prompt é passado como argumento posicional
            args: vec![],
            timeout: Duration::from_secs(30),
        }
    }

    /// Cria executor a partir da configuração do TOML.
    pub fn from_config(config: &ExecutorConfig) -> Self {
        Self {
            command_name: config.command.clone(),
            args: config.args.clone(),
            timeout: Duration::from_secs(config.timeout_secs),
        }
    }

    /// Define o timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for QwenExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CliExecutor for QwenExecutor {
    fn name(&self) -> &str {
        "Qwen"
    }

    fn command(&self) -> &str {
        &self.command_name
    }

    fn specialization(&self) -> &str {
        "logic"
    }

    async fn evaluate(&self, request: &EvaluationRequest) -> TetradResult<ModelVote> {
        let prompt = self.build_prompt(request);

        // Constrói o comando com argumentos do config
        let mut cmd = Command::new(&self.command_name);
        for arg in &self.args {
            cmd.arg(arg);
        }
        cmd.arg(&prompt);

        // Executa a CLI com timeout
        let result = tokio::time::timeout(self.timeout, cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let response = ExecutorResponse::parse_from_output(&stdout, self.name())?;
                    Ok(response.into_vote(self.name()))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(TetradError::ExecutorFailed(
                        self.name().to_string(),
                        stderr.to_string(),
                    ))
                }
            }
            Ok(Err(e)) => {
                // CLI não encontrada ou erro de execução
                if e.kind() == std::io::ErrorKind::NotFound {
                    // Retorna voto neutro se CLI não estiver disponível
                    Ok(ModelVote::new(self.name(), Vote::Warn, 50)
                        .with_reasoning("Qwen CLI não disponível"))
                } else {
                    Err(TetradError::ExecutorFailed(
                        self.name().to_string(),
                        e.to_string(),
                    ))
                }
            }
            Err(_) => Err(TetradError::ExecutorTimeout(self.name().to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_valid() {
        let output = r#"{"vote": "FAIL", "score": 30, "reasoning": "Critical bug", "issues": ["bug1", "bug2"], "suggestions": ["fix1"]}"#;

        let response = ExecutorResponse::parse_from_output(output, "Qwen");
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.vote, "FAIL");
        assert_eq!(response.score, 30);
        assert_eq!(response.issues.len(), 2);
    }

    #[test]
    fn test_specialization() {
        let executor = QwenExecutor::new();
        assert_eq!(executor.specialization(), "logic");
    }
}
