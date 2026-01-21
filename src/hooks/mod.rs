//! Sistema de hooks do Tetrad.
//!
//! Hooks permitem customizar o comportamento do Tetrad em pontos
//! específicos do fluxo de avaliação:
//!
//! - `pre_evaluate`: Antes de enviar código para avaliação
//! - `post_evaluate`: Após receber resultado da avaliação
//! - `on_consensus`: Quando consenso é alcançado
//! - `on_block`: Quando código é bloqueado

mod builtin;

pub use builtin::{LoggingHook, MetricsHook};

use async_trait::async_trait;

use crate::types::requests::EvaluationRequest;
use crate::types::responses::EvaluationResult;
use crate::TetradResult;

// ═══════════════════════════════════════════════════════════════════════════
// Tipos de eventos
// ═══════════════════════════════════════════════════════════════════════════

/// Evento que dispara um hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    /// Antes de iniciar avaliação.
    PreEvaluate,

    /// Após avaliação completa.
    PostEvaluate,

    /// Quando consenso é alcançado.
    OnConsensus,

    /// Quando código é bloqueado.
    OnBlock,
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookEvent::PreEvaluate => write!(f, "pre_evaluate"),
            HookEvent::PostEvaluate => write!(f, "post_evaluate"),
            HookEvent::OnConsensus => write!(f, "on_consensus"),
            HookEvent::OnBlock => write!(f, "on_block"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Contexto de hooks
// ═══════════════════════════════════════════════════════════════════════════

/// Contexto passado para hooks.
pub enum HookContext<'a> {
    /// Contexto para pre_evaluate.
    PreEvaluate {
        /// Request de avaliação.
        request: &'a EvaluationRequest,
    },

    /// Contexto para post_evaluate.
    PostEvaluate {
        /// Request original.
        request: &'a EvaluationRequest,
        /// Resultado da avaliação.
        result: &'a EvaluationResult,
    },

    /// Contexto para on_consensus.
    OnConsensus {
        /// Resultado da avaliação.
        result: &'a EvaluationResult,
    },

    /// Contexto para on_block.
    OnBlock {
        /// Resultado da avaliação (com decisão Block).
        result: &'a EvaluationResult,
    },
}

impl<'a> HookContext<'a> {
    /// Retorna o evento correspondente ao contexto.
    pub fn event(&self) -> HookEvent {
        match self {
            HookContext::PreEvaluate { .. } => HookEvent::PreEvaluate,
            HookContext::PostEvaluate { .. } => HookEvent::PostEvaluate,
            HookContext::OnConsensus { .. } => HookEvent::OnConsensus,
            HookContext::OnBlock { .. } => HookEvent::OnBlock,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Resultado de hooks
// ═══════════════════════════════════════════════════════════════════════════

/// Resultado da execução de um hook.
#[derive(Debug, Clone, Default)]
pub enum HookResult {
    /// Continua normalmente.
    #[default]
    Continue,

    /// Pula a avaliação (apenas válido para pre_evaluate).
    Skip,

    /// Modifica a request (apenas válido para pre_evaluate).
    ModifyRequest(EvaluationRequest),
}

// ═══════════════════════════════════════════════════════════════════════════
// Trait Hook
// ═══════════════════════════════════════════════════════════════════════════

/// Trait para hooks customizáveis.
#[async_trait]
pub trait Hook: Send + Sync {
    /// Nome do hook.
    fn name(&self) -> &str;

    /// Evento que dispara este hook.
    fn event(&self) -> HookEvent;

    /// Executa o hook.
    async fn execute(&self, context: &HookContext<'_>) -> TetradResult<HookResult>;
}

// ═══════════════════════════════════════════════════════════════════════════
// Sistema de hooks
// ═══════════════════════════════════════════════════════════════════════════

/// Gerenciador de hooks.
pub struct HookSystem {
    pre_evaluate: Vec<Box<dyn Hook>>,
    post_evaluate: Vec<Box<dyn Hook>>,
    on_consensus: Vec<Box<dyn Hook>>,
    on_block: Vec<Box<dyn Hook>>,
}

impl HookSystem {
    /// Cria um novo sistema de hooks vazio.
    pub fn new() -> Self {
        Self {
            pre_evaluate: Vec::new(),
            post_evaluate: Vec::new(),
            on_consensus: Vec::new(),
            on_block: Vec::new(),
        }
    }

    /// Cria um sistema com hooks padrão (logging).
    pub fn with_defaults() -> Self {
        let mut system = Self::new();
        system.register(Box::new(LoggingHook));
        system
    }

    /// Registra um hook.
    pub fn register(&mut self, hook: Box<dyn Hook>) {
        let event = hook.event();
        tracing::debug!(
            hook_name = hook.name(),
            event = %event,
            "Registering hook"
        );

        match event {
            HookEvent::PreEvaluate => self.pre_evaluate.push(hook),
            HookEvent::PostEvaluate => self.post_evaluate.push(hook),
            HookEvent::OnConsensus => self.on_consensus.push(hook),
            HookEvent::OnBlock => self.on_block.push(hook),
        }
    }

    /// Executa hooks de pre_evaluate.
    ///
    /// Retorna o resultado final (Continue, Skip ou ModifyRequest).
    pub async fn run_pre_evaluate(&self, request: &EvaluationRequest) -> TetradResult<HookResult> {
        let context = HookContext::PreEvaluate { request };

        for hook in &self.pre_evaluate {
            let result = hook.execute(&context).await?;
            match result {
                HookResult::Continue => continue,
                HookResult::Skip => return Ok(HookResult::Skip),
                HookResult::ModifyRequest(new_request) => {
                    return Ok(HookResult::ModifyRequest(new_request))
                }
            }
        }

        Ok(HookResult::Continue)
    }

    /// Executa hooks de post_evaluate.
    pub async fn run_post_evaluate(
        &self,
        request: &EvaluationRequest,
        result: &EvaluationResult,
    ) -> TetradResult<()> {
        let context = HookContext::PostEvaluate { request, result };

        for hook in &self.post_evaluate {
            hook.execute(&context).await?;
        }

        Ok(())
    }

    /// Executa hooks de on_consensus.
    pub async fn run_on_consensus(&self, result: &EvaluationResult) -> TetradResult<()> {
        let context = HookContext::OnConsensus { result };

        for hook in &self.on_consensus {
            hook.execute(&context).await?;
        }

        Ok(())
    }

    /// Executa hooks de on_block.
    pub async fn run_on_block(&self, result: &EvaluationResult) -> TetradResult<()> {
        let context = HookContext::OnBlock { result };

        for hook in &self.on_block {
            hook.execute(&context).await?;
        }

        Ok(())
    }

    /// Retorna o número total de hooks registrados.
    pub fn count(&self) -> usize {
        self.pre_evaluate.len()
            + self.post_evaluate.len()
            + self.on_consensus.len()
            + self.on_block.len()
    }

    /// Retorna o número de hooks para um evento específico.
    pub fn count_for_event(&self, event: HookEvent) -> usize {
        match event {
            HookEvent::PreEvaluate => self.pre_evaluate.len(),
            HookEvent::PostEvaluate => self.post_evaluate.len(),
            HookEvent::OnConsensus => self.on_consensus.len(),
            HookEvent::OnBlock => self.on_block.len(),
        }
    }
}

impl Default for HookSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::responses::Decision;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // Hook de teste que conta execuções
    struct CountingHook {
        name: String,
        event: HookEvent,
        count: Arc<AtomicUsize>,
    }

    impl CountingHook {
        fn new(name: &str, event: HookEvent, count: Arc<AtomicUsize>) -> Self {
            Self {
                name: name.to_string(),
                event,
                count,
            }
        }
    }

    #[async_trait]
    impl Hook for CountingHook {
        fn name(&self) -> &str {
            &self.name
        }

        fn event(&self) -> HookEvent {
            self.event
        }

        async fn execute(&self, _context: &HookContext<'_>) -> TetradResult<HookResult> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(HookResult::Continue)
        }
    }

    fn create_test_request() -> EvaluationRequest {
        EvaluationRequest::new("fn main() {}", "rust")
    }

    fn create_test_result() -> EvaluationResult {
        EvaluationResult {
            request_id: "test-123".to_string(),
            decision: Decision::Pass,
            score: 85,
            consensus_achieved: true,
            votes: HashMap::new(),
            findings: vec![],
            feedback: "Test feedback".to_string(),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_hook_system_new() {
        let system = HookSystem::new();
        assert_eq!(system.count(), 0);
    }

    #[test]
    fn test_hook_system_with_defaults() {
        let system = HookSystem::with_defaults();
        assert!(system.count() > 0);
    }

    #[test]
    fn test_hook_registration() {
        let mut system = HookSystem::new();
        let count = Arc::new(AtomicUsize::new(0));

        system.register(Box::new(CountingHook::new(
            "test",
            HookEvent::PreEvaluate,
            count,
        )));

        assert_eq!(system.count_for_event(HookEvent::PreEvaluate), 1);
        assert_eq!(system.count_for_event(HookEvent::PostEvaluate), 0);
    }

    #[tokio::test]
    async fn test_pre_evaluate_hook() {
        let mut system = HookSystem::new();
        let count = Arc::new(AtomicUsize::new(0));

        system.register(Box::new(CountingHook::new(
            "test",
            HookEvent::PreEvaluate,
            count.clone(),
        )));

        let request = create_test_request();
        let result = system.run_pre_evaluate(&request).await.unwrap();

        assert!(matches!(result, HookResult::Continue));
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_post_evaluate_hook() {
        let mut system = HookSystem::new();
        let count = Arc::new(AtomicUsize::new(0));

        system.register(Box::new(CountingHook::new(
            "test",
            HookEvent::PostEvaluate,
            count.clone(),
        )));

        let request = create_test_request();
        let result = create_test_result();
        system.run_post_evaluate(&request, &result).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_on_consensus_hook() {
        let mut system = HookSystem::new();
        let count = Arc::new(AtomicUsize::new(0));

        system.register(Box::new(CountingHook::new(
            "test",
            HookEvent::OnConsensus,
            count.clone(),
        )));

        let result = create_test_result();
        system.run_on_consensus(&result).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_on_block_hook() {
        let mut system = HookSystem::new();
        let count = Arc::new(AtomicUsize::new(0));

        system.register(Box::new(CountingHook::new(
            "test",
            HookEvent::OnBlock,
            count.clone(),
        )));

        let result = create_test_result();
        system.run_on_block(&result).await.unwrap();

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_multiple_hooks_chain() {
        let mut system = HookSystem::new();
        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));

        system.register(Box::new(CountingHook::new(
            "first",
            HookEvent::PreEvaluate,
            count1.clone(),
        )));
        system.register(Box::new(CountingHook::new(
            "second",
            HookEvent::PreEvaluate,
            count2.clone(),
        )));

        let request = create_test_request();
        system.run_pre_evaluate(&request).await.unwrap();

        // Ambos devem ser executados
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_hook_event_display() {
        assert_eq!(format!("{}", HookEvent::PreEvaluate), "pre_evaluate");
        assert_eq!(format!("{}", HookEvent::PostEvaluate), "post_evaluate");
        assert_eq!(format!("{}", HookEvent::OnConsensus), "on_consensus");
        assert_eq!(format!("{}", HookEvent::OnBlock), "on_block");
    }

    #[test]
    fn test_hook_context_event() {
        let request = create_test_request();
        let result = create_test_result();

        let ctx_pre = HookContext::PreEvaluate { request: &request };
        assert_eq!(ctx_pre.event(), HookEvent::PreEvaluate);

        let ctx_post = HookContext::PostEvaluate {
            request: &request,
            result: &result,
        };
        assert_eq!(ctx_post.event(), HookEvent::PostEvaluate);

        let ctx_consensus = HookContext::OnConsensus { result: &result };
        assert_eq!(ctx_consensus.event(), HookEvent::OnConsensus);

        let ctx_block = HookContext::OnBlock { result: &result };
        assert_eq!(ctx_block.event(), HookEvent::OnBlock);
    }
}
