//! Handlers das ferramentas MCP do Tetrad.
//!
//! Este módulo implementa as 6 ferramentas expostas pelo servidor MCP:
//!
//! 1. `tetrad_review_plan` - Revisa planos de implementação
//! 2. `tetrad_review_code` - Revisa código antes de salvar
//! 3. `tetrad_review_tests` - Revisa testes
//! 4. `tetrad_confirm` - Confirma acordo com feedback
//! 5. `tetrad_final_check` - Verificação final antes de commit
//! 6. `tetrad_status` - Status dos avaliadores

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{Mutex, RwLock};

use crate::cache::EvaluationCache;
use crate::consensus::ConsensusEngine;
use crate::executors::{CliExecutor, CodexExecutor, GeminiExecutor, QwenExecutor};
use crate::hooks::HookSystem;
use crate::reasoning::ReasoningBank;
use crate::types::config::Config;
use crate::types::requests::{EvaluationRequest, EvaluationType};
use crate::types::responses::{Decision, EvaluationResult, ModelVote};
use crate::TetradResult;

use super::protocol::{ToolDescription, ToolResult};

// ═══════════════════════════════════════════════════════════════════════════
// Parâmetros das ferramentas
// ═══════════════════════════════════════════════════════════════════════════

/// Parâmetros para review_plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewPlanParams {
    /// Plano a ser revisado.
    pub plan: String,

    /// Contexto adicional.
    #[serde(default)]
    pub context: Option<String>,
}

/// Parâmetros para review_code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCodeParams {
    /// Código a ser revisado.
    pub code: String,

    /// Linguagem do código.
    pub language: String,

    /// Caminho do arquivo.
    #[serde(default)]
    pub file_path: Option<String>,

    /// Contexto adicional.
    #[serde(default)]
    pub context: Option<String>,
}

/// Parâmetros para review_tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewTestsParams {
    /// Código dos testes.
    pub tests: String,

    /// Linguagem.
    pub language: String,

    /// Contexto adicional.
    #[serde(default)]
    pub context: Option<String>,
}

/// Parâmetros para confirm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmParams {
    /// ID da request original.
    pub request_id: String,

    /// Se concorda com o feedback.
    pub agreed: bool,

    /// Notas adicionais.
    #[serde(default)]
    pub notes: Option<String>,
}

/// Parâmetros para final_check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalCheckParams {
    /// Código a verificar.
    pub code: String,

    /// Linguagem.
    pub language: String,

    /// ID da request anterior (para comparação).
    #[serde(default)]
    pub previous_request_id: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Handler de ferramentas
// ═══════════════════════════════════════════════════════════════════════════

/// Handler das ferramentas MCP do Tetrad.
pub struct ToolHandler {
    config: Config,
    codex: CodexExecutor,
    gemini: GeminiExecutor,
    qwen: QwenExecutor,
    consensus: ConsensusEngine,
    // Usa Mutex em vez de RwLock porque rusqlite::Connection não é Sync
    reasoning_bank: Arc<Mutex<Option<ReasoningBank>>>,
    cache: Arc<RwLock<EvaluationCache>>,
    hooks: HookSystem,
    confirmations: Arc<RwLock<HashMap<String, bool>>>,
}

impl ToolHandler {
    /// Cria um novo handler de ferramentas.
    pub fn new(config: Config) -> TetradResult<Self> {
        let codex = CodexExecutor::from_config(&config.executors.codex);
        let gemini = GeminiExecutor::from_config(&config.executors.gemini);
        let qwen = QwenExecutor::from_config(&config.executors.qwen);
        let consensus = ConsensusEngine::new(config.consensus.clone());

        // Inicializa ReasoningBank se habilitado
        let reasoning_bank = if config.reasoning.enabled {
            let bank = ReasoningBank::new(&config.reasoning.db_path)?;
            Some(bank)
        } else {
            None
        };

        // Inicializa cache usando configurações
        let cache = EvaluationCache::new(
            config.cache.capacity,
            Duration::from_secs(config.cache.ttl_secs),
        );

        Ok(Self {
            config,
            codex,
            gemini,
            qwen,
            consensus,
            reasoning_bank: Arc::new(Mutex::new(reasoning_bank)),
            cache: Arc::new(RwLock::new(cache)),
            hooks: HookSystem::with_defaults(),
            confirmations: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Lista todas as ferramentas disponíveis.
    pub fn list_tools() -> Vec<ToolDescription> {
        vec![
            ToolDescription::new(
                "tetrad_review_plan",
                "Revisa um plano de implementação antes de começar a codificar. Use ANTES de implementar qualquer feature.",
                json!({
                    "type": "object",
                    "properties": {
                        "plan": {
                            "type": "string",
                            "description": "O plano de implementação a ser revisado"
                        },
                        "context": {
                            "type": "string",
                            "description": "Contexto adicional sobre o projeto ou requisitos"
                        }
                    },
                    "required": ["plan"]
                }),
            ),
            ToolDescription::new(
                "tetrad_review_code",
                "Revisa código antes de salvar. Use ANTES de salvar qualquer arquivo de código.",
                json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "O código a ser revisado"
                        },
                        "language": {
                            "type": "string",
                            "description": "Linguagem de programação (rust, python, javascript, etc.)"
                        },
                        "file_path": {
                            "type": "string",
                            "description": "Caminho do arquivo (opcional)"
                        },
                        "context": {
                            "type": "string",
                            "description": "Contexto adicional"
                        }
                    },
                    "required": ["code", "language"]
                }),
            ),
            ToolDescription::new(
                "tetrad_review_tests",
                "Revisa testes antes de finalizar. Use ANTES de considerar os testes prontos.",
                json!({
                    "type": "object",
                    "properties": {
                        "tests": {
                            "type": "string",
                            "description": "O código dos testes a serem revisados"
                        },
                        "language": {
                            "type": "string",
                            "description": "Linguagem de programação"
                        },
                        "context": {
                            "type": "string",
                            "description": "Contexto sobre o que está sendo testado"
                        }
                    },
                    "required": ["tests", "language"]
                }),
            ),
            ToolDescription::new(
                "tetrad_confirm",
                "Confirma que você concorda com o feedback recebido e fez as correções necessárias.",
                json!({
                    "type": "object",
                    "properties": {
                        "request_id": {
                            "type": "string",
                            "description": "ID da avaliação anterior"
                        },
                        "agreed": {
                            "type": "boolean",
                            "description": "Se concorda com o feedback"
                        },
                        "notes": {
                            "type": "string",
                            "description": "Notas sobre as correções feitas"
                        }
                    },
                    "required": ["request_id", "agreed"]
                }),
            ),
            ToolDescription::new(
                "tetrad_final_check",
                "Verificação final antes de commit. Use após todas as correções para obter certificação.",
                json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "O código final a ser verificado"
                        },
                        "language": {
                            "type": "string",
                            "description": "Linguagem de programação"
                        },
                        "previous_request_id": {
                            "type": "string",
                            "description": "ID da avaliação anterior para comparação"
                        }
                    },
                    "required": ["code", "language"]
                }),
            ),
            ToolDescription::new(
                "tetrad_status",
                "Mostra o status dos avaliadores (Codex, Gemini, Qwen).",
                json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            ),
        ]
    }

    /// Processa uma chamada de ferramenta.
    pub async fn handle_tool_call(&self, name: &str, arguments: Value) -> ToolResult {
        tracing::info!(tool = name, "Processing tool call");

        match name {
            "tetrad_review_plan" => self.handle_review_plan(arguments).await,
            "tetrad_review_code" => self.handle_review_code(arguments).await,
            "tetrad_review_tests" => self.handle_review_tests(arguments).await,
            "tetrad_confirm" => self.handle_confirm(arguments).await,
            "tetrad_final_check" => self.handle_final_check(arguments).await,
            "tetrad_status" => self.handle_status().await,
            _ => ToolResult::error(format!("Unknown tool: {}", name)),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Handlers individuais
    // ═══════════════════════════════════════════════════════════════════════

    async fn handle_review_plan(&self, arguments: Value) -> ToolResult {
        let params: ReviewPlanParams = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        let mut request =
            EvaluationRequest::new(&params.plan, "text").with_type(EvaluationType::Plan);

        if let Some(ctx) = params.context {
            request = request.with_context(&ctx);
        }

        self.evaluate_request(request).await
    }

    async fn handle_review_code(&self, arguments: Value) -> ToolResult {
        let params: ReviewCodeParams = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        // Verifica cache
        {
            let mut cache = self.cache.write().await;
            if let Some(cached) =
                cache.get_by_code(&params.code, &params.language, &EvaluationType::Code)
            {
                tracing::info!("Cache hit for review_code");
                return self.format_result(cached);
            }
        }

        let mut request =
            EvaluationRequest::new(&params.code, &params.language).with_type(EvaluationType::Code);

        if let Some(fp) = params.file_path.clone() {
            request = request.with_file_path(&fp);
        }
        if let Some(ctx) = params.context.clone() {
            request = request.with_context(&ctx);
        }

        // Executa avaliação internamente para poder cachear o resultado
        match self.evaluate_internal(request).await {
            Ok(eval_result) => {
                // Armazena em cache
                {
                    let mut cache = self.cache.write().await;
                    cache.insert_by_code(
                        &params.code,
                        &params.language,
                        &EvaluationType::Code,
                        eval_result.clone(),
                    );
                }
                self.format_result(&eval_result)
            }
            Err(e) => ToolResult::error(format!("Evaluation failed: {}", e)),
        }
    }

    async fn handle_review_tests(&self, arguments: Value) -> ToolResult {
        let params: ReviewTestsParams = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        let mut request = EvaluationRequest::new(&params.tests, &params.language)
            .with_type(EvaluationType::Tests);

        if let Some(ctx) = params.context {
            request = request.with_context(&ctx);
        }

        self.evaluate_request(request).await
    }

    async fn handle_confirm(&self, arguments: Value) -> ToolResult {
        let params: ConfirmParams = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        // Registra confirmação
        {
            let mut confirmations = self.confirmations.write().await;
            confirmations.insert(params.request_id.clone(), params.agreed);
        }

        let response = json!({
            "confirmed": true,
            "request_id": params.request_id,
            "agreed": params.agreed,
            "notes": params.notes,
            "can_proceed": params.agreed,
            "message": if params.agreed {
                "Confirmação registrada. Você pode prosseguir com o próximo passo."
            } else {
                "Discordância registrada. Por favor, revise o código novamente."
            }
        });

        ToolResult::success_json(&response)
    }

    async fn handle_final_check(&self, arguments: Value) -> ToolResult {
        let params: FinalCheckParams = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid parameters: {}", e)),
        };

        // Verifica se há confirmação prévia do previous_request_id
        let previous_confirmed = if let Some(ref prev_id) = params.previous_request_id {
            let confirmations = self.confirmations.read().await;
            confirmations.get(prev_id).copied().unwrap_or(false)
        } else {
            false
        };

        let request = EvaluationRequest::new(&params.code, &params.language)
            .with_type(EvaluationType::FinalCheck);

        let result = self.evaluate_internal(request).await;

        match result {
            Ok(eval_result) => {
                // Certificação requer: consenso + score mínimo + confirmação prévia (se fornecida)
                let meets_requirements = eval_result.consensus_achieved
                    && eval_result.score >= self.config.consensus.min_score;

                // Se previous_request_id foi fornecido, exige confirmação
                let certified = if params.previous_request_id.is_some() {
                    meets_requirements && previous_confirmed
                } else {
                    meets_requirements
                };

                let message = if certified {
                    "CERTIFICADO: Código aprovado pelo consenso quádruplo do Tetrad."
                } else if !meets_requirements {
                    "NÃO CERTIFICADO: Código não atingiu consenso ou score mínimo."
                } else {
                    "NÃO CERTIFICADO: Confirmação prévia pendente. Use tetrad_confirm primeiro."
                };

                let response = json!({
                    "certified": certified,
                    "decision": format!("{:?}", eval_result.decision),
                    "score": eval_result.score,
                    "consensus_achieved": eval_result.consensus_achieved,
                    "previous_request_id": params.previous_request_id,
                    "previous_confirmed": previous_confirmed,
                    "certificate_id": if certified {
                        Some(format!("TETRAD-{}", eval_result.request_id))
                    } else {
                        None
                    },
                    "feedback": eval_result.feedback,
                    "findings_count": eval_result.findings.len(),
                    "message": message
                });

                ToolResult::success_json(&response)
            }
            Err(e) => ToolResult::error(format!("Evaluation failed: {}", e)),
        }
    }

    async fn handle_status(&self) -> ToolResult {
        let codex_available = self.codex.is_available().await;
        let gemini_available = self.gemini.is_available().await;
        let qwen_available = self.qwen.is_available().await;

        let codex_version = if codex_available {
            self.codex
                .version()
                .await
                .unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unavailable".to_string()
        };

        let gemini_version = if gemini_available {
            self.gemini
                .version()
                .await
                .unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unavailable".to_string()
        };

        let qwen_version = if qwen_available {
            self.qwen
                .version()
                .await
                .unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unavailable".to_string()
        };

        let cache_stats = {
            let cache = self.cache.read().await;
            cache.stats()
        };

        let response = json!({
            "codex": {
                "available": codex_available,
                "version": codex_version,
                "specialization": self.codex.specialization(),
                "enabled": self.config.executors.codex.enabled
            },
            "gemini": {
                "available": gemini_available,
                "version": gemini_version,
                "specialization": self.gemini.specialization(),
                "enabled": self.config.executors.gemini.enabled
            },
            "qwen": {
                "available": qwen_available,
                "version": qwen_version,
                "specialization": self.qwen.specialization(),
                "enabled": self.config.executors.qwen.enabled
            },
            "consensus": {
                "rule": format!("{:?}", self.config.consensus.default_rule),
                "min_score": self.config.consensus.min_score,
                "max_loops": self.config.consensus.max_loops
            },
            "cache": {
                "size": cache_stats.size,
                "capacity": cache_stats.capacity,
                "hit_rate": format!("{:.1}%", cache_stats.hit_rate() * 100.0)
            },
            "reasoning_bank": {
                "enabled": self.config.reasoning.enabled
            }
        });

        ToolResult::success_json(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Métodos auxiliares
    // ═══════════════════════════════════════════════════════════════════════

    /// Executa uma avaliação e retorna o resultado formatado.
    async fn evaluate_request(&self, request: EvaluationRequest) -> ToolResult {
        match self.evaluate_internal(request).await {
            Ok(result) => self.format_result(&result),
            Err(e) => ToolResult::error(format!("Evaluation failed: {}", e)),
        }
    }

    /// Executa a avaliação interna.
    async fn evaluate_internal(
        &self,
        request: EvaluationRequest,
    ) -> TetradResult<EvaluationResult> {
        // Executa hooks pre_evaluate
        let hook_result = self.hooks.run_pre_evaluate(&request).await?;

        // Trata resultado do hook
        let request = match hook_result {
            crate::hooks::HookResult::Skip => {
                // Retorna resultado de skip
                return Ok(EvaluationResult::success(
                    &request.request_id,
                    100,
                    "Skipped by hook",
                ));
            }
            crate::hooks::HookResult::ModifyRequest(modified) => {
                // Usa a request modificada pelo hook
                tracing::info!("Request modified by pre_evaluate hook");
                modified
            }
            crate::hooks::HookResult::Continue => request,
        };

        // Consulta ReasoningBank
        let known_patterns = {
            let bank = self.reasoning_bank.lock().await;
            if let Some(ref b) = *bank {
                b.retrieve(&request.code, &request.language)
            } else {
                vec![]
            }
        };

        // Log de patterns conhecidos
        if !known_patterns.is_empty() {
            tracing::info!(
                patterns_count = known_patterns.len(),
                "Found known patterns from ReasoningBank"
            );
        }

        // Coleta votos dos executores em paralelo
        let votes = self.collect_votes(&request).await;

        // Aplica consenso
        let result = self.consensus.evaluate(votes, &request.request_id);

        // Executa hooks post_evaluate
        self.hooks.run_post_evaluate(&request, &result).await?;

        // Executa hooks específicos
        if result.consensus_achieved {
            self.hooks.run_on_consensus(&result).await?;
        }
        if matches!(result.decision, Decision::Block) {
            self.hooks.run_on_block(&result).await?;
        }

        // Registra no ReasoningBank
        {
            let mut bank = self.reasoning_bank.lock().await;
            if let Some(ref mut b) = *bank {
                let _ = b.judge(
                    &result.request_id,
                    &request.code,
                    &request.language,
                    &result,
                    1,
                    self.config.consensus.max_loops,
                );
            }
        }

        Ok(result)
    }

    /// Coleta votos de todos os executores habilitados.
    async fn collect_votes(&self, request: &EvaluationRequest) -> HashMap<String, ModelVote> {
        let mut votes = HashMap::new();

        // Executa em paralelo
        let (codex_vote, gemini_vote, qwen_vote) = tokio::join!(
            self.get_vote_if_enabled(&self.codex, request, self.config.executors.codex.enabled),
            self.get_vote_if_enabled(&self.gemini, request, self.config.executors.gemini.enabled),
            self.get_vote_if_enabled(&self.qwen, request, self.config.executors.qwen.enabled),
        );

        if let Some(vote) = codex_vote {
            votes.insert("Codex".to_string(), vote);
        }
        if let Some(vote) = gemini_vote {
            votes.insert("Gemini".to_string(), vote);
        }
        if let Some(vote) = qwen_vote {
            votes.insert("Qwen".to_string(), vote);
        }

        votes
    }

    /// Obtém voto de um executor se habilitado.
    async fn get_vote_if_enabled<E: CliExecutor>(
        &self,
        executor: &E,
        request: &EvaluationRequest,
        enabled: bool,
    ) -> Option<ModelVote> {
        if !enabled {
            return None;
        }

        match executor.evaluate(request).await {
            Ok(vote) => Some(vote),
            Err(e) => {
                tracing::warn!(
                    executor = executor.name(),
                    error = %e,
                    "Executor failed, using fallback vote"
                );
                // Voto neutro em caso de erro
                Some(ModelVote::new(
                    executor.name(),
                    crate::types::responses::Vote::Warn,
                    50,
                ))
            }
        }
    }

    /// Formata o resultado para retorno MCP.
    fn format_result(&self, result: &EvaluationResult) -> ToolResult {
        let status = match result.decision {
            Decision::Pass => "PASS",
            Decision::Revise => "REVISE",
            Decision::Block => "BLOCK",
        };

        let response = json!({
            "request_id": result.request_id,
            "decision": status,
            "score": result.score,
            "consensus_achieved": result.consensus_achieved,
            "findings": result.findings.iter().map(|f| json!({
                "severity": format!("{:?}", f.severity),
                "category": f.category,
                "issue": f.issue,
                "suggestion": f.suggestion,
                "consensus_strength": f.consensus_strength
            })).collect::<Vec<_>>(),
            "feedback": result.feedback,
            "votes": result.votes.iter().map(|(name, vote)| {
                json!({
                    "executor": name,
                    "vote": format!("{:?}", vote.vote),
                    "score": vote.score
                })
            }).collect::<Vec<_>>()
        });

        ToolResult::success_json(&response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools() {
        let tools = ToolHandler::list_tools();
        assert_eq!(tools.len(), 6);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"tetrad_review_plan"));
        assert!(tool_names.contains(&"tetrad_review_code"));
        assert!(tool_names.contains(&"tetrad_review_tests"));
        assert!(tool_names.contains(&"tetrad_confirm"));
        assert!(tool_names.contains(&"tetrad_final_check"));
        assert!(tool_names.contains(&"tetrad_status"));
    }

    #[test]
    fn test_review_code_params_deserialize() {
        let json = json!({
            "code": "fn main() {}",
            "language": "rust",
            "file_path": "src/main.rs"
        });

        let params: ReviewCodeParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.code, "fn main() {}");
        assert_eq!(params.language, "rust");
        assert_eq!(params.file_path, Some("src/main.rs".to_string()));
    }

    #[test]
    fn test_confirm_params_deserialize() {
        let json = json!({
            "request_id": "test-123",
            "agreed": true,
            "notes": "Fixed all issues"
        });

        let params: ConfirmParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.request_id, "test-123");
        assert!(params.agreed);
        assert_eq!(params.notes, Some("Fixed all issues".to_string()));
    }

    #[test]
    fn test_tool_description() {
        let tools = ToolHandler::list_tools();
        let review_code = tools
            .iter()
            .find(|t| t.name == "tetrad_review_code")
            .unwrap();

        assert!(!review_code.description.is_empty());

        // Verifica que o schema tem as propriedades esperadas
        let schema = &review_code.input_schema;
        assert!(schema["properties"]["code"].is_object());
        assert!(schema["properties"]["language"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("code")));
    }
}
