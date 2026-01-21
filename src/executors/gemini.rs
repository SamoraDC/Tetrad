//! Executor para Gemini CLI.

use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
use tokio::process::Command;

use super::base::{CliExecutor, ExecutorResponse};
use crate::types::config::ExecutorConfig;
use crate::types::requests::EvaluationRequest;
use crate::types::responses::{ModelVote, Vote};
use crate::{TetradError, TetradResult};

/// Estrutura do wrapper JSON retornado pelo Gemini CLI com -o json.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GeminiWrapper {
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    response: String,
    #[serde(default)]
    stats: serde_json::Value,
}

/// Executor para Gemini CLI (Google).
///
/// Especialização: Arquitetura e design de código.
/// Usa `-o json` para output estruturado e parseia o wrapper JSON.
pub struct GeminiExecutor {
    command_name: String,
    args: Vec<String>,
    timeout: Duration,
}

impl GeminiExecutor {
    /// Cria um novo executor Gemini com valores padrão.
    pub fn new() -> Self {
        Self {
            command_name: "gemini".to_string(),
            // -o json para formato de saída estruturado
            args: vec!["-o".to_string(), "json".to_string()],
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

    /// Parseia o output do Gemini CLI que vem em formato wrapper JSON.
    /// O Gemini retorna: {"session_id": "...", "response": "texto", "stats": {...}}
    fn parse_gemini_output(output: &str) -> TetradResult<ExecutorResponse> {
        // Remove linhas de debug/log que podem vir antes do JSON
        let json_start = output.find('{');
        let output = if let Some(start) = json_start {
            &output[start..]
        } else {
            output
        };

        // Tenta parsear o wrapper JSON do Gemini
        if let Ok(wrapper) = serde_json::from_str::<GeminiWrapper>(output) {
            // Tenta extrair JSON estruturado do campo response
            if let Ok(response) = ExecutorResponse::parse_from_output(&wrapper.response, "Gemini") {
                return Ok(response);
            }

            // Fallback: analisa o texto da resposta semanticamente
            return Ok(Self::analyze_text_response(&wrapper.response));
        }

        // Tenta parsear diretamente como ExecutorResponse (caso o modelo retorne JSON)
        if let Ok(response) = ExecutorResponse::parse_from_output(output, "Gemini") {
            return Ok(response);
        }

        Err(TetradError::ExecutorFailed(
            "Gemini".to_string(),
            "Não foi possível parsear resposta do Gemini".to_string(),
        ))
    }

    /// Analisa texto de resposta e extrai informações estruturadas.
    fn analyze_text_response(text: &str) -> ExecutorResponse {
        let lower = text.to_lowercase();

        // Determina o voto baseado em palavras-chave
        let vote = if lower.contains("erro crítico")
            || lower.contains("bug grave")
            || lower.contains("vulnerabilidade")
            || lower.contains("falha de segurança")
            || lower.contains("critical error")
            || lower.contains("security vulnerability")
        {
            "FAIL"
        } else if lower.contains("problema")
            || lower.contains("issue")
            || lower.contains("considere")
            || lower.contains("sugestão")
            || lower.contains("atenção")
            || lower.contains("melhoria")
            || lower.contains("overflow")
            || lower.contains("observação")
            || lower.contains("consider")
            || lower.contains("suggestion")
        {
            "WARN"
        } else {
            "PASS"
        };

        // Score baseado no voto e conteúdo
        let score = if vote == "PASS" {
            if lower.contains("perfeito")
                || lower.contains("excelente")
                || lower.contains("perfect")
            {
                95
            } else if lower.contains("bom")
                || lower.contains("correto")
                || lower.contains("idiomático")
            {
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
                trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("• ")
            })
            .map(|line| {
                line.trim()
                    .trim_start_matches("- ")
                    .trim_start_matches("* ")
                    .trim_start_matches("• ")
                    .to_string()
            })
            .take(5)
            .collect();

        // Extrai sugestões (linhas que contêm "sugest" ou "consider")
        let suggestions: Vec<String> = text
            .lines()
            .filter(|line| {
                let lower_line = line.to_lowercase();
                lower_line.contains("sugest") || lower_line.contains("consider")
            })
            .map(|line| line.trim().to_string())
            .take(3)
            .collect();

        ExecutorResponse {
            vote: vote.to_string(),
            score,
            reasoning: text.chars().take(500).collect(),
            issues,
            suggestions,
        }
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
        &self.command_name
    }

    fn specialization(&self) -> &str {
        "architecture"
    }

    async fn evaluate(&self, request: &EvaluationRequest) -> TetradResult<ModelVote> {
        let prompt = self.build_prompt(request);

        // Constrói o comando: gemini -o json "prompt"
        let mut cmd = Command::new(&self.command_name);

        // Adiciona argumentos do config (deve incluir "-o" e "json")
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

                // Gemini pode escrever logs em stderr mesmo com sucesso
                if !stdout.is_empty() {
                    // Tenta parsear o output do Gemini
                    match Self::parse_gemini_output(&stdout) {
                        Ok(response) => return Ok(response.into_vote(self.name())),
                        Err(e) => {
                            tracing::debug!(
                                "Falha ao parsear output do Gemini: {}. Tentando stderr...",
                                e
                            );
                        }
                    }
                }

                // Verifica se há erro no stderr
                if !stderr.is_empty() && (stderr.contains("Error") || stderr.contains("error")) {
                    // Ignora mensagens de "Loaded cached credentials"
                    if !stderr.contains("Loaded cached credentials") {
                        return Err(TetradError::ExecutorFailed(
                            self.name().to_string(),
                            stderr.to_string(),
                        ));
                    }
                }

                // Se stdout estava vazio, tenta stderr (caso output vá para lá)
                if stdout.is_empty() && !stderr.is_empty() {
                    if let Ok(response) = Self::parse_gemini_output(&stderr) {
                        return Ok(response.into_vote(self.name()));
                    }
                }

                Err(TetradError::ExecutorFailed(
                    self.name().to_string(),
                    "Não foi possível parsear resposta do Gemini".to_string(),
                ))
            }
            Ok(Err(e)) => {
                // CLI não encontrada ou erro de execução
                if e.kind() == std::io::ErrorKind::NotFound {
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

    #[test]
    fn test_parse_gemini_wrapper() {
        let output = r#"{
            "session_id": "test-123",
            "response": "A função está correta e bem estruturada.",
            "stats": {}
        }"#;

        let response = GeminiExecutor::parse_gemini_output(output);
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.vote, "PASS");
        assert!(response.score >= 80);
    }

    #[test]
    fn test_parse_gemini_wrapper_with_json_response() {
        let output = r#"{
            "session_id": "test-123",
            "response": "{\"vote\": \"PASS\", \"score\": 95, \"reasoning\": \"Excelente!\", \"issues\": [], \"suggestions\": []}",
            "stats": {}
        }"#;

        let response = GeminiExecutor::parse_gemini_output(output);
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.vote, "PASS");
        assert_eq!(response.score, 95);
    }

    #[test]
    fn test_parse_gemini_with_log_prefix() {
        let output = r#"Loaded cached credentials.
{
    "session_id": "test-123",
    "response": "Código aprovado.",
    "stats": {}
}"#;

        let response = GeminiExecutor::parse_gemini_output(output);
        assert!(response.is_ok());
    }

    #[test]
    fn test_analyze_text_response_pass() {
        let text = "A função está correta e bem estruturada. Código idiomático.";
        let response = GeminiExecutor::analyze_text_response(text);
        assert_eq!(response.vote, "PASS");
        assert!(response.score >= 80);
    }

    #[test]
    fn test_analyze_text_response_warn() {
        let text = "O código funciona, mas considere adicionar tratamento de overflow para maior segurança.";
        let response = GeminiExecutor::analyze_text_response(text);
        assert_eq!(response.vote, "WARN");
        assert!(response.score >= 50 && response.score < 80);
    }

    #[test]
    fn test_analyze_text_response_fail() {
        let text = "Erro crítico: vulnerabilidade de segurança detectada no código.";
        let response = GeminiExecutor::analyze_text_response(text);
        assert_eq!(response.vote, "FAIL");
        assert!(response.score < 50);
    }

    #[test]
    fn test_analyze_text_extracts_issues() {
        let text = "Problemas encontrados:\n- Falta documentação\n- Nomes de variáveis pouco claros\n* Ausência de testes";
        let response = GeminiExecutor::analyze_text_response(text);
        assert_eq!(response.issues.len(), 3);
    }

    #[test]
    fn test_default_args() {
        let executor = GeminiExecutor::new();
        assert_eq!(executor.args, vec!["-o", "json"]);
    }
}
