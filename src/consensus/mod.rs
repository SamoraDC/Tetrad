//! Motor de consenso do Tetrad.
//!
//! Este módulo implementa o sistema de consenso quádruplo,
//! que agrega votos de múltiplos executores e determina
//! se o código deve ser aprovado, revisado ou bloqueado.
//!
//! ## Regras de Consenso
//!
//! - **Golden**: Unanimidade necessária (todos devem votar PASS)
//! - **Strong**: Consenso forte (3/3 CLIs concordam)
//! - **Weak**: Consenso fraco (2+ CLIs concordam)
//!
//! ## Exemplo
//!
//! ```rust,ignore
//! use tetrad::consensus::ConsensusEngine;
//! use tetrad::types::config::ConsensusConfig;
//!
//! let config = ConsensusConfig::default();
//! let engine = ConsensusEngine::new(config);
//!
//! let result = engine.evaluate(votes, "request-123");
//! if result.consensus_achieved {
//!     println!("Consenso alcançado: {:?}", result.decision);
//! }
//! ```

mod aggregator;
mod engine;
mod rules;

pub use aggregator::VoteAggregator;
pub use engine::ConsensusEngine;
pub use rules::{create_rule, ConsensusRule, GoldenRule, StrongRule, WeakRule};
