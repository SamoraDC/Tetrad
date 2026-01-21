//! Hooks padrão do Tetrad.
//!
//! Este módulo contém hooks que vêm pré-configurados com o Tetrad:
//! - `LoggingHook`: Registra avaliações no log
//! - `MetricsHook`: Coleta métricas de avaliação

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;

use crate::TetradResult;

use super::{Hook, HookContext, HookEvent, HookResult};

// ═══════════════════════════════════════════════════════════════════════════
// LoggingHook
// ═══════════════════════════════════════════════════════════════════════════

/// Hook que registra avaliações no log.
///
/// Este hook é executado após cada avaliação (post_evaluate) e registra
/// informações sobre o resultado usando o sistema de logging (tracing).
#[derive(Debug, Default)]
pub struct LoggingHook;

impl LoggingHook {
    /// Cria um novo LoggingHook.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Hook for LoggingHook {
    fn name(&self) -> &str {
        "logging"
    }

    fn event(&self) -> HookEvent {
        HookEvent::PostEvaluate
    }

    async fn execute(&self, context: &HookContext<'_>) -> TetradResult<HookResult> {
        if let HookContext::PostEvaluate { request, result } = context {
            tracing::info!(
                request_id = %result.request_id,
                language = %request.language,
                decision = ?result.decision,
                score = result.score,
                consensus = result.consensus_achieved,
                findings_count = result.findings.len(),
                "Evaluation completed"
            );

            // Log detalhado para decisões Block
            if matches!(result.decision, crate::types::responses::Decision::Block) {
                tracing::warn!(
                    request_id = %result.request_id,
                    feedback = %result.feedback,
                    "Code blocked - review required"
                );
            }
        }

        Ok(HookResult::Continue)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MetricsHook
// ═══════════════════════════════════════════════════════════════════════════

/// Hook que coleta métricas de avaliação.
///
/// Mantém contadores de avaliações, passes, bloqueios e score médio.
#[derive(Debug, Default)]
pub struct MetricsHook {
    /// Total de avaliações.
    evaluations: AtomicU64,

    /// Total de passes.
    passes: AtomicU64,

    /// Total de revises.
    revises: AtomicU64,

    /// Total de bloqueios.
    blocks: AtomicU64,

    /// Soma de todos os scores (para calcular média).
    score_sum: AtomicU64,
}

impl MetricsHook {
    /// Cria um novo MetricsHook.
    pub fn new() -> Self {
        Self::default()
    }

    /// Retorna o total de avaliações.
    pub fn total_evaluations(&self) -> u64 {
        self.evaluations.load(Ordering::Relaxed)
    }

    /// Retorna o total de passes.
    pub fn total_passes(&self) -> u64 {
        self.passes.load(Ordering::Relaxed)
    }

    /// Retorna o total de revises.
    pub fn total_revises(&self) -> u64 {
        self.revises.load(Ordering::Relaxed)
    }

    /// Retorna o total de bloqueios.
    pub fn total_blocks(&self) -> u64 {
        self.blocks.load(Ordering::Relaxed)
    }

    /// Retorna a taxa de sucesso (passes / total).
    pub fn success_rate(&self) -> f64 {
        let total = self.total_evaluations();
        if total == 0 {
            0.0
        } else {
            self.total_passes() as f64 / total as f64
        }
    }

    /// Retorna o score médio.
    pub fn average_score(&self) -> f64 {
        let total = self.total_evaluations();
        if total == 0 {
            0.0
        } else {
            self.score_sum.load(Ordering::Relaxed) as f64 / total as f64
        }
    }

    /// Retorna as métricas em formato estruturado.
    pub fn metrics(&self) -> Metrics {
        Metrics {
            total_evaluations: self.total_evaluations(),
            passes: self.total_passes(),
            revises: self.total_revises(),
            blocks: self.total_blocks(),
            success_rate: self.success_rate(),
            average_score: self.average_score(),
        }
    }
}

/// Métricas coletadas pelo MetricsHook.
#[derive(Debug, Clone)]
pub struct Metrics {
    pub total_evaluations: u64,
    pub passes: u64,
    pub revises: u64,
    pub blocks: u64,
    pub success_rate: f64,
    pub average_score: f64,
}

#[async_trait]
impl Hook for MetricsHook {
    fn name(&self) -> &str {
        "metrics"
    }

    fn event(&self) -> HookEvent {
        HookEvent::PostEvaluate
    }

    async fn execute(&self, context: &HookContext<'_>) -> TetradResult<HookResult> {
        if let HookContext::PostEvaluate { result, .. } = context {
            // Incrementa contador de avaliações
            self.evaluations.fetch_add(1, Ordering::Relaxed);

            // Incrementa contador específico da decisão
            match result.decision {
                crate::types::responses::Decision::Pass => {
                    self.passes.fetch_add(1, Ordering::Relaxed);
                }
                crate::types::responses::Decision::Revise => {
                    self.revises.fetch_add(1, Ordering::Relaxed);
                }
                crate::types::responses::Decision::Block => {
                    self.blocks.fetch_add(1, Ordering::Relaxed);
                }
            }

            // Acumula score
            self.score_sum.fetch_add(result.score as u64, Ordering::Relaxed);
        }

        Ok(HookResult::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::requests::EvaluationRequest;
    use crate::types::responses::{Decision, EvaluationResult};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_request() -> EvaluationRequest {
        EvaluationRequest::new("fn main() {}", "rust")
    }

    fn create_test_result(decision: Decision, score: u8) -> EvaluationResult {
        EvaluationResult {
            request_id: "test-123".to_string(),
            decision,
            score,
            consensus_achieved: true,
            votes: HashMap::new(),
            findings: vec![],
            feedback: "Test feedback".to_string(),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_logging_hook_name() {
        let hook = LoggingHook::new();
        assert_eq!(hook.name(), "logging");
    }

    #[test]
    fn test_logging_hook_event() {
        let hook = LoggingHook::new();
        assert_eq!(hook.event(), HookEvent::PostEvaluate);
    }

    #[tokio::test]
    async fn test_logging_hook_execute() {
        let hook = LoggingHook::new();
        let request = create_test_request();
        let result = create_test_result(Decision::Pass, 85);

        let context = HookContext::PostEvaluate {
            request: &request,
            result: &result,
        };

        let hook_result = hook.execute(&context).await.unwrap();
        assert!(matches!(hook_result, HookResult::Continue));
    }

    #[test]
    fn test_metrics_hook_name() {
        let hook = MetricsHook::new();
        assert_eq!(hook.name(), "metrics");
    }

    #[test]
    fn test_metrics_hook_event() {
        let hook = MetricsHook::new();
        assert_eq!(hook.event(), HookEvent::PostEvaluate);
    }

    #[tokio::test]
    async fn test_metrics_hook_counts_evaluations() {
        let hook = MetricsHook::new();
        let request = create_test_request();

        // Executa algumas avaliações
        let pass_result = create_test_result(Decision::Pass, 90);
        let revise_result = create_test_result(Decision::Revise, 60);
        let block_result = create_test_result(Decision::Block, 30);

        let pass_ctx = HookContext::PostEvaluate {
            request: &request,
            result: &pass_result,
        };
        let revise_ctx = HookContext::PostEvaluate {
            request: &request,
            result: &revise_result,
        };
        let block_ctx = HookContext::PostEvaluate {
            request: &request,
            result: &block_result,
        };

        hook.execute(&pass_ctx).await.unwrap();
        hook.execute(&pass_ctx).await.unwrap();
        hook.execute(&revise_ctx).await.unwrap();
        hook.execute(&block_ctx).await.unwrap();

        assert_eq!(hook.total_evaluations(), 4);
        assert_eq!(hook.total_passes(), 2);
        assert_eq!(hook.total_revises(), 1);
        assert_eq!(hook.total_blocks(), 1);
    }

    #[tokio::test]
    async fn test_metrics_hook_success_rate() {
        let hook = MetricsHook::new();
        let request = create_test_request();

        let pass_result = create_test_result(Decision::Pass, 90);
        let block_result = create_test_result(Decision::Block, 30);

        let pass_ctx = HookContext::PostEvaluate {
            request: &request,
            result: &pass_result,
        };
        let block_ctx = HookContext::PostEvaluate {
            request: &request,
            result: &block_result,
        };

        hook.execute(&pass_ctx).await.unwrap();
        hook.execute(&pass_ctx).await.unwrap();
        hook.execute(&pass_ctx).await.unwrap();
        hook.execute(&block_ctx).await.unwrap();

        // 3 passes de 4 total = 75%
        assert!((hook.success_rate() - 0.75).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_metrics_hook_average_score() {
        let hook = MetricsHook::new();
        let request = create_test_request();

        let result1 = create_test_result(Decision::Pass, 80);
        let result2 = create_test_result(Decision::Pass, 90);
        let result3 = create_test_result(Decision::Pass, 100);

        hook.execute(&HookContext::PostEvaluate {
            request: &request,
            result: &result1,
        })
        .await
        .unwrap();
        hook.execute(&HookContext::PostEvaluate {
            request: &request,
            result: &result2,
        })
        .await
        .unwrap();
        hook.execute(&HookContext::PostEvaluate {
            request: &request,
            result: &result3,
        })
        .await
        .unwrap();

        // (80 + 90 + 100) / 3 = 90
        assert!((hook.average_score() - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_metrics_hook_empty() {
        let hook = MetricsHook::new();

        assert_eq!(hook.total_evaluations(), 0);
        assert_eq!(hook.success_rate(), 0.0);
        assert_eq!(hook.average_score(), 0.0);
    }

    #[tokio::test]
    async fn test_metrics_struct() {
        let hook = MetricsHook::new();
        let request = create_test_request();
        let result = create_test_result(Decision::Pass, 85);

        hook.execute(&HookContext::PostEvaluate {
            request: &request,
            result: &result,
        })
        .await
        .unwrap();

        let metrics = hook.metrics();
        assert_eq!(metrics.total_evaluations, 1);
        assert_eq!(metrics.passes, 1);
        assert_eq!(metrics.revises, 0);
        assert_eq!(metrics.blocks, 0);
        assert!((metrics.success_rate - 1.0).abs() < 0.01);
        assert!((metrics.average_score - 85.0).abs() < 0.01);
    }
}
