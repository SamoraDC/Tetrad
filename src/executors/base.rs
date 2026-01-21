//! Trait base para executores CLI.

use async_trait::async_trait;

use crate::types::requests::EvaluationRequest;
use crate::types::responses::ModelVote;
use crate::{TetradError, TetradResult};

/// Trait para executores CLI de avaliação de código.
///
/// Cada executor encapsula uma CLI externa (Codex, Gemini, Qwen)
/// e fornece uma interface unificada para avaliação de código.
#[async_trait]
pub trait CliExecutor: Send + Sync {
    /// Retorna o nome do executor.
    fn name(&self) -> &str;

    /// Retorna o comando CLI.
    fn command(&self) -> &str;

    /// Verifica se a CLI está disponível no sistema.
    async fn is_available(&self) -> bool {
        tokio::process::Command::new(self.command())
            .arg("--version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Retorna a versão da CLI.
    async fn version(&self) -> TetradResult<String> {
        let output = tokio::process::Command::new(self.command())
            .arg("--version")
            .output()
            .await?;

        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("unknown")
            .to_string();

        Ok(version)
    }

    /// Executa uma avaliação de código.
    ///
    /// # Arguments
    ///
    /// * `request` - Requisição de avaliação
    ///
    /// # Returns
    ///
    /// Voto do modelo com score, issues e sugestões.
    async fn evaluate(&self, request: &EvaluationRequest) -> TetradResult<ModelVote>;

    /// Retorna a especialização deste executor.
    ///
    /// - "syntax" para foco em sintaxe e convenções
    /// - "architecture" para foco em arquitetura e design
    /// - "logic" para foco em bugs lógicos
    fn specialization(&self) -> &str;

    /// Constrói o prompt para a avaliação.
    fn build_prompt(&self, request: &EvaluationRequest) -> String {
        let eval_type = request.evaluation_type.to_string();
        let language = &request.language;
        let code = &request.code;

        let mut prompt = format!(
            "Avalie o seguinte código {} para {}.\n\n",
            language, eval_type
        );

        prompt.push_str("Código:\n```\n");
        prompt.push_str(code);
        prompt.push_str("\n```\n\n");

        if let Some(context) = &request.context {
            prompt.push_str("Contexto adicional:\n");
            prompt.push_str(context);
            prompt.push_str("\n\n");
        }

        prompt.push_str("Responda em JSON com o formato:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"vote\": \"PASS\" | \"WARN\" | \"FAIL\",\n");
        prompt.push_str("  \"score\": 0-100,\n");
        prompt.push_str("  \"reasoning\": \"explicação\",\n");
        prompt.push_str("  \"issues\": [\"issue1\", \"issue2\"],\n");
        prompt.push_str("  \"suggestions\": [\"sugestão1\", \"sugestão2\"]\n");
        prompt.push_str("}\n");

        prompt
    }
}

/// Resposta parseada de um executor.
#[derive(Debug, serde::Deserialize)]
pub struct ExecutorResponse {
    pub vote: String,
    pub score: u8,
    pub reasoning: String,
    #[serde(default)]
    pub issues: Vec<String>,
    #[serde(default)]
    pub suggestions: Vec<String>,
}

impl ExecutorResponse {
    /// Parseia uma resposta JSON de um executor.
    ///
    /// Busca o primeiro objeto JSON válido na saída e o converte em ExecutorResponse.
    pub fn parse_from_output(output: &str, executor_name: &str) -> TetradResult<Self> {
        let json_start = output.find('{');
        let json_end = output.rfind('}');

        match (json_start, json_end) {
            (Some(start), Some(end)) if start < end => {
                let json_str = &output[start..=end];
                serde_json::from_str(json_str).map_err(|e| {
                    TetradError::ExecutorFailed(
                        executor_name.to_string(),
                        format!("Falha ao parsear JSON: {}", e),
                    )
                })
            }
            _ => Err(TetradError::ExecutorFailed(
                executor_name.to_string(),
                "Resposta não contém JSON válido".to_string(),
            )),
        }
    }

    /// Converte a resposta em um ModelVote.
    pub fn into_vote(self, executor_name: &str) -> ModelVote {
        use crate::types::responses::Vote;

        let vote = match self.vote.to_uppercase().as_str() {
            "PASS" => Vote::Pass,
            "WARN" => Vote::Warn,
            _ => Vote::Fail,
        };

        ModelVote::new(executor_name, vote, self.score)
            .with_reasoning(self.reasoning)
            .with_issues(self.issues)
            .with_suggestions(self.suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExecutor;

    #[async_trait]
    impl CliExecutor for MockExecutor {
        fn name(&self) -> &str {
            "mock"
        }

        fn command(&self) -> &str {
            "echo"
        }

        async fn evaluate(&self, _request: &EvaluationRequest) -> TetradResult<ModelVote> {
            use crate::types::responses::Vote;
            Ok(ModelVote::new("mock", Vote::Pass, 100))
        }

        fn specialization(&self) -> &str {
            "test"
        }
    }

    #[test]
    fn test_build_prompt() {
        let executor = MockExecutor;
        let request = EvaluationRequest::new("fn main() {}", "rust");

        let prompt = executor.build_prompt(&request);

        assert!(prompt.contains("rust"));
        assert!(prompt.contains("fn main() {}"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_build_prompt_with_context() {
        let executor = MockExecutor;
        let request =
            EvaluationRequest::new("fn main() {}", "rust").with_context("Este é um teste");

        let prompt = executor.build_prompt(&request);

        assert!(prompt.contains("Este é um teste"));
    }

    #[test]
    fn test_executor_response_into_vote() {
        let response = ExecutorResponse {
            vote: "PASS".to_string(),
            score: 85,
            reasoning: "Código bom".to_string(),
            issues: vec![],
            suggestions: vec!["Adicionar testes".to_string()],
        };

        let vote = response.into_vote("test");

        assert_eq!(vote.executor, "test");
        assert_eq!(vote.score, 85);
        assert_eq!(vote.suggestions.len(), 1);
    }
}
