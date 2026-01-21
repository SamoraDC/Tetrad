//! Executor para Gemini CLI.

use async_trait::async_trait;
use std::time::Duration;
use tokio::process::Command;

use super::base::{CliExecutor, ExecutorResponse};
use crate::types::requests::EvaluationRequest;
use crate::types::responses::{ModelVote, Vote};
use crate::{TetradError, TetradResult};

/// Executor para Gemini CLI (Google).
///
/// Especialização: Arquitetura e design de código.
pub struct GeminiExecutor {
    timeout: Duration,
}

impl GeminiExecutor {
    /// Cria um novo executor Gemini.
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }

    /// Define o timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for GeminiExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CliExecutor for GeminiExecutor {
    fn name(&self) -> &str {
        "Gemini"
    }

    fn command(&self) -> &str {
        "gemini"
    }

    fn specialization(&self) -> &str {
        "architecture"
    }

    async fn evaluate(&self, request: &EvaluationRequest) -> TetradResult<ModelVote> {
        let prompt = self.build_prompt(request);

        // Executa a CLI com timeout
        let result = tokio::time::timeout(
            self.timeout,
            Command::new(self.command())
                .arg("--output-format")
                .arg("json")
                .arg(&prompt)
                .output(),
        )
        .await;

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
                        .with_reasoning("Gemini CLI não disponível"))
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
        let output = r#"{"vote": "WARN", "score": 70, "reasoning": "Some issues", "issues": ["issue1"], "suggestions": []}"#;

        let response = ExecutorResponse::parse_from_output(output, "Gemini");
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.vote, "WARN");
        assert_eq!(response.score, 70);
        assert_eq!(response.issues.len(), 1);
    }

    #[test]
    fn test_specialization() {
        let executor = GeminiExecutor::new();
        assert_eq!(executor.specialization(), "architecture");
    }
}
