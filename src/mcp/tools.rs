//! MCP tool handlers for Tetrad.
//!
//! This module implements the 6 tools exposed by the MCP server:
//!
//! 1. `tetrad_review_plan` - Reviews implementation plans
//! 2. `tetrad_review_code` - Reviews code before saving
//! 3. `tetrad_review_tests` - Reviews tests
//! 4. `tetrad_confirm` - Confirms agreement with feedback
//! 5. `tetrad_final_check` - Final check before commit
//! 6. `tetrad_status` - Evaluator status

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
// Tool parameters
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for review_plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewPlanParams {
    /// Plan to be reviewed.
    pub plan: String,

    /// Additional context.
    #[serde(default)]
    pub context: Option<String>,
}

/// Parameters for review_code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCodeParams {
    /// Code to be reviewed.
    pub code: String,

    /// Code language.
    pub language: String,

    /// File path.
    #[serde(default)]
    pub file_path: Option<String>,

    /// Additional context.
    #[serde(default)]
    pub context: Option<String>,
}

/// Parameters for review_tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewTestsParams {
    /// Test code.
    pub tests: String,

    /// Language.
    pub language: String,

    /// Additional context.
    #[serde(default)]
    pub context: Option<String>,
}

/// Parameters for confirm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmParams {
    /// Original request ID.
    pub request_id: String,

    /// Whether agrees with feedback.
    pub agreed: bool,

    /// Additional notes.
    #[serde(default)]
    pub notes: Option<String>,
}

/// Parameters for final_check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalCheckParams {
    /// Code to verify.
    pub code: String,

    /// Language.
    pub language: String,

    /// Previous request ID (for comparison).
    #[serde(default)]
    pub previous_request_id: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tool handler
// ═══════════════════════════════════════════════════════════════════════════

/// MCP tool handler for Tetrad.
pub struct ToolHandler {
    config: Config,
    codex: CodexExecutor,
    gemini: GeminiExecutor,
    qwen: QwenExecutor,
    consensus: ConsensusEngine,
    // Uses Mutex instead of RwLock because rusqlite::Connection is not Sync
    reasoning_bank: Arc<Mutex<Option<ReasoningBank>>>,
    cache: Arc<RwLock<EvaluationCache>>,
    hooks: HookSystem,
    confirmations: Arc<RwLock<HashMap<String, bool>>>,
}

impl ToolHandler {
    /// Creates a new tool handler.
    pub fn new(config: Config) -> TetradResult<Self> {
        let codex = CodexExecutor::from_config(&config.executors.codex);
        let gemini = GeminiExecutor::from_config(&config.executors.gemini);
        let qwen = QwenExecutor::from_config(&config.executors.qwen);
        let consensus = ConsensusEngine::new(config.consensus.clone());

        // Initialize ReasoningBank if enabled
        let reasoning_bank = if config.reasoning.enabled {
            let bank = ReasoningBank::new(&config.reasoning.db_path)?;
            Some(bank)
        } else {
            None
        };

        // Initialize cache using settings
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

    /// Lists all available tools.
    pub fn list_tools() -> Vec<ToolDescription> {
        vec![
            ToolDescription::new(
                "tetrad_review_plan",
                "Reviews an implementation plan before starting to code. Use BEFORE implementing any feature.",
                json!({
                    "type": "object",
                    "properties": {
                        "plan": {
                            "type": "string",
                            "description": "The implementation plan to be reviewed"
                        },
                        "context": {
                            "type": "string",
                            "description": "Additional context about the project or requirements"
                        }
                    },
                    "required": ["plan"]
                }),
            ),
            ToolDescription::new(
                "tetrad_review_code",
                "Reviews code before saving. Use BEFORE saving any code file.",
                json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The code to be reviewed"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language (rust, python, javascript, etc.)"
                        },
                        "file_path": {
                            "type": "string",
                            "description": "File path (optional)"
                        },
                        "context": {
                            "type": "string",
                            "description": "Additional context"
                        }
                    },
                    "required": ["code", "language"]
                }),
            ),
            ToolDescription::new(
                "tetrad_review_tests",
                "Reviews tests before finalizing. Use BEFORE considering tests ready.",
                json!({
                    "type": "object",
                    "properties": {
                        "tests": {
                            "type": "string",
                            "description": "The test code to be reviewed"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language"
                        },
                        "context": {
                            "type": "string",
                            "description": "Context about what is being tested"
                        }
                    },
                    "required": ["tests", "language"]
                }),
            ),
            ToolDescription::new(
                "tetrad_confirm",
                "Confirms that you agree with the feedback received and made the necessary corrections.",
                json!({
                    "type": "object",
                    "properties": {
                        "request_id": {
                            "type": "string",
                            "description": "Previous evaluation ID"
                        },
                        "agreed": {
                            "type": "boolean",
                            "description": "Whether agrees with feedback"
                        },
                        "notes": {
                            "type": "string",
                            "description": "Notes about corrections made"
                        }
                    },
                    "required": ["request_id", "agreed"]
                }),
            ),
            ToolDescription::new(
                "tetrad_final_check",
                "Final check before commit. Use after all corrections to obtain certification.",
                json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The final code to be verified"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language"
                        },
                        "previous_request_id": {
                            "type": "string",
                            "description": "Previous evaluation ID for comparison"
                        }
                    },
                    "required": ["code", "language"]
                }),
            ),
            ToolDescription::new(
                "tetrad_status",
                "Shows the status of evaluators (Codex, Gemini, Qwen).",
                json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            ),
        ]
    }

    /// Processes a tool call.
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
    // Individual handlers
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
                "Confirmation registered. You can proceed to the next step."
            } else {
                "Disagreement registered. Please review the code again."
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
                    "CERTIFIED: Code approved by Tetrad's quadruple consensus."
                } else if !meets_requirements {
                    "NOT CERTIFIED: Code did not reach consensus or minimum score."
                } else {
                    "NOT CERTIFIED: Prior confirmation pending. Use tetrad_confirm first."
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
    // Helper methods
    // ═══════════════════════════════════════════════════════════════════════

    /// Executes an evaluation and returns formatted result.
    async fn evaluate_request(&self, request: EvaluationRequest) -> ToolResult {
        match self.evaluate_internal(request).await {
            Ok(result) => self.format_result(&result),
            Err(e) => ToolResult::error(format!("Evaluation failed: {}", e)),
        }
    }

    /// Executes the internal evaluation.
    async fn evaluate_internal(
        &self,
        request: EvaluationRequest,
    ) -> TetradResult<EvaluationResult> {
        // Run pre_evaluate hooks
        let hook_result = self.hooks.run_pre_evaluate(&request).await?;

        // Handle hook result
        let request = match hook_result {
            crate::hooks::HookResult::Skip => {
                // Return skip result
                return Ok(EvaluationResult::success(
                    &request.request_id,
                    100,
                    "Skipped by hook",
                ));
            }
            crate::hooks::HookResult::ModifyRequest(modified) => {
                // Use the modified request from hook
                tracing::info!("Request modified by pre_evaluate hook");
                modified
            }
            crate::hooks::HookResult::Continue => request,
        };

        // Query ReasoningBank
        let known_patterns = {
            let bank = self.reasoning_bank.lock().await;
            if let Some(ref b) = *bank {
                b.retrieve(&request.code, &request.language)
            } else {
                vec![]
            }
        };

        // Log known patterns
        if !known_patterns.is_empty() {
            tracing::info!(
                patterns_count = known_patterns.len(),
                "Found known patterns from ReasoningBank"
            );
        }

        // Collect votes from executors in parallel
        let votes = self.collect_votes(&request).await;

        // Apply consensus
        let result = self.consensus.evaluate(votes, &request.request_id);

        // Run post_evaluate hooks
        self.hooks.run_post_evaluate(&request, &result).await?;

        // Run specific hooks
        if result.consensus_achieved {
            self.hooks.run_on_consensus(&result).await?;
        }
        if matches!(result.decision, Decision::Block) {
            self.hooks.run_on_block(&result).await?;
        }

        // Register in ReasoningBank
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

    /// Collects votes from all enabled executors.
    async fn collect_votes(&self, request: &EvaluationRequest) -> HashMap<String, ModelVote> {
        let mut votes = HashMap::new();

        // Execute in parallel
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

    /// Gets vote from an executor if enabled.
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
                // Neutral vote in case of error
                Some(ModelVote::new(
                    executor.name(),
                    crate::types::responses::Vote::Warn,
                    50,
                ))
            }
        }
    }

    /// Formats the result for MCP return.
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
