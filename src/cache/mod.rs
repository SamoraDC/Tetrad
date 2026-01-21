//! Cache LRU para resultados de avaliação.
//!
//! Este módulo implementa um cache Least Recently Used (LRU) para
//! armazenar resultados de avaliações recentes, evitando reavaliações
//! desnecessárias do mesmo código.

mod lru;

pub use lru::{CacheStats, CachedResult, EvaluationCache};
