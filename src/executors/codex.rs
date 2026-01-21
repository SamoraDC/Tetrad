//! Executor para Codex CLI.

use async_trait::async_trait;
use std::time::Duration;
use tokio::process::Command;

use super::base::{CliExecutor, ExecutorResponse};
use crate::types::config::ExecutorConfig;
use crate::types::requests::EvaluationRequest;
use crate::types::responses::{ModelVote, Vote};
use crate::{TetradError, TetradResult};

/// Executor para Codex CLI (OpenAI).
///
/// Especialização: Sintaxe e convenções de código.
/// Usa o modo `codex exec --json` para execução não-interativa.
pub struct CodexExecutor {
    command_name: String,
    args: Vec<String>,
    timeout: Duration,
}

impl CodexExecutor {
    /// Cria um novo executor Codex com valores padrão.
    pub fn new() -> Self {
        Self {
            command_name: "codex".to_string(),
            // Usa exec --json para modo não-interativo
            args: vec!["exec".to_string(), "--json".to_string()],
            timeout: Duration::from_secs(60),
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

    /// Parseia eventos JSON Lines (NDJSON) do codex exec --json.
    /// Extrai a mensagem do agente do evento item.completed com type: "agent_message".
    fn parse_codex_events(output: &str) -> Option<String> {
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                // Procura por item.completed com type: "agent_message"
                if event.get("type").and_then(|t| t.as_str()) == Some("item.completed") {
                    if let Some(item) = event.get("item") {
                        if item.get("type").and_then(|t| t.as_str()) == Some("agent_message") {
                            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                return Some(text.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Analisa texto de resposta e extrai informações estruturadas.
    fn analyze_text_response(text: &str) -> ExecutorResponse {
        let lower = text.to_lowercase();

        // Determina o voto baseado em palavras-chave
        let vote = if lower.contains("erro crítico")
            || lower.contains("bug grave")
            || lower.contains("vulnerabilidade")
            || lower.contains("falha de segurança")
        {
            "FAIL"
        } else if lower.contains("problema")
            || lower.contains("issue")
            || lower.contains("considere")
            || lower.contains("sugestão")
            || lower.contains("atenção")
            || lower.contains("melhoria")
            || lower.contains("overflow")
        {
            "WARN"
        } else {
            "PASS"
        };

        // Score baseado no voto e conteúdo
        let score = if vote == "PASS" {
            if lower.contains("perfeito") || lower.contains("excelente") {
                95
            } else if lower.contains("bom") || lower.contains("correto") {
                85
            } else {
                80
            }
        } else if vote == "WARN" {
            if lower.contains("menor") || lower.contains("minor") {
                70
            } else {
                60
            }
        } else {
            35
        };

        // Extrai issues do texto (linhas que começam com - ou *)
        let issues: Vec<String> = text
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("- ") || trimmed.starts_with("* ")
            })
            .map(|line| {
                line.trim()
                    .trim_start_matches("- ")
                    .trim_start_matches("* ")
                    .to_string()
            })
            .take(5)
            .collect();

        ExecutorResponse {
            vote: vote.to_string(),
            score,
            reasoning: text.chars().take(500).collect(),
            issues,
            suggestions: vec![],
        }
    }
}

impl Default for CodexExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CliExecutor for CodexExecutor {
    fn name(&self) -> &str {
        "Codex"
    }

    fn command(&self) -> &str {
        &self.command_name
    }

    fn specialization(&self) -> &str {
        "syntax"
    }

    async fn evaluate(&self, request: &EvaluationRequest) -> TetradResult<ModelVote> {
        let prompt = self.build_prompt(request);

        // Constrói o comando: codex exec --json "prompt"
        let mut cmd = Command::new(&self.command_name);

        // Adiciona argumentos do config (deve incluir "exec" e "--json")
        for arg in &self.args {
            cmd.arg(arg);
        }

        // Adiciona o prompt
        cmd.arg(&prompt);

        // Executa a CLI com timeout
        let result = tokio::time::timeout(self.timeout, cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // codex exec retorna exit code 0 mesmo com erros em alguns casos
                // então verificamos o stdout primeiro
                if !stdout.is_empty() {
                    // Tenta parsear os eventos JSON Lines
                    if let Some(agent_message) = Self::parse_codex_events(&stdout) {
                        // Tenta extrair JSON estruturado da mensagem
                        if let Ok(response) =
                            ExecutorResponse::parse_from_output(&agent_message, self.name())
                        {
                            return Ok(response.into_vote(self.name()));
                        }

                        // Fallback: analisa o texto da mensagem
                        let response = Self::analyze_text_response(&agent_message);
                        return Ok(response.into_vote(self.name()));
                    }
                }

                // Se não conseguiu parsear, verifica se há erro
                if !stderr.is_empty() && stderr.contains("Error") {
                    return Err(TetradError::ExecutorFailed(
                        self.name().to_string(),
                        stderr.to_string(),
                    ));
                }

                // Fallback: tenta parsear stdout diretamente
                if let Ok(response) = ExecutorResponse::parse_from_output(&stdout, self.name()) {
                    return Ok(response.into_vote(self.name()));
                }

                Err(TetradError::ExecutorFailed(
                    self.name().to_string(),
                    "Não foi possível parsear resposta do Codex".to_string(),
                ))
            }
            Ok(Err(e)) => {
                // CLI não encontrada ou erro de execução
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(ModelVote::new(self.name(), Vote::Warn, 50)
                        .with_reasoning("Codex CLI não disponível"))
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
        let output = r#"
            Some text before
            {"vote": "PASS", "score": 85, "reasoning": "Good code", "issues": [], "suggestions": []}
            Some text after
        "#;

        let response = ExecutorResponse::parse_from_output(output, "Codex");
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.vote, "PASS");
        assert_eq!(response.score, 85);
    }

    #[test]
    fn test_parse_response_invalid() {
        let output = "No JSON here";

        let response = ExecutorResponse::parse_from_output(output, "Codex");
        assert!(response.is_err());
    }

    #[test]
    fn test_specialization() {
        let executor = CodexExecutor::new();
        assert_eq!(executor.specialization(), "syntax");
    }

    #[test]
    fn test_parse_codex_events() {
        let output = r#"{"type":"thread.started","thread_id":"test-123"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"item_0","type":"reasoning","text":"Thinking..."}}
{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"Código aprovado sem problemas."}}
{"type":"turn.completed","usage":{"input_tokens":100,"output_tokens":50}}"#;

        let message = CodexExecutor::parse_codex_events(output);
        assert!(message.is_some());
        assert_eq!(message.unwrap(), "Código aprovado sem problemas.");
    }

    #[test]
    fn test_parse_codex_events_no_agent_message() {
        let output = r#"{"type":"thread.started","thread_id":"test-123"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"item_0","type":"reasoning","text":"Thinking..."}}"#;

        let message = CodexExecutor::parse_codex_events(output);
        assert!(message.is_none());
    }

    #[test]
    fn test_analyze_text_response_pass() {
        let text = "O código está correto e bem estruturado. Bom trabalho!";
        let response = CodexExecutor::analyze_text_response(text);
        assert_eq!(response.vote, "PASS");
        assert!(response.score >= 80);
    }

    #[test]
    fn test_analyze_text_response_warn() {
        let text = "O código funciona, mas considere adicionar tratamento de overflow.";
        let response = CodexExecutor::analyze_text_response(text);
        assert_eq!(response.vote, "WARN");
        assert!(response.score >= 50 && response.score < 80);
    }

    #[test]
    fn test_analyze_text_response_fail() {
        let text = "Erro crítico: vulnerabilidade de segurança detectada.";
        let response = CodexExecutor::analyze_text_response(text);
        assert_eq!(response.vote, "FAIL");
        assert!(response.score < 50);
    }

    #[test]
    fn test_default_args() {
        let executor = CodexExecutor::new();
        assert_eq!(executor.args, vec!["exec", "--json"]);
    }
}
