//! Cache LRU para resultados de avaliação.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use chrono::{DateTime, Utc};
use lru::LruCache;
use sha2::{Digest, Sha256};

use crate::types::requests::EvaluationType;
use crate::types::responses::EvaluationResult;

/// Resultado em cache.
#[derive(Debug, Clone)]
pub struct CachedResult {
    /// Resultado da avaliação.
    pub result: EvaluationResult,

    /// Momento em que foi cacheado.
    pub cached_at: DateTime<Utc>,
}

impl CachedResult {
    /// Cria um novo resultado em cache.
    pub fn new(result: EvaluationResult) -> Self {
        Self {
            result,
            cached_at: Utc::now(),
        }
    }

    /// Verifica se o cache expirou.
    pub fn is_expired(&self, ttl: Duration) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.cached_at)
            .to_std()
            .unwrap_or(Duration::MAX);
        elapsed > ttl
    }
}

/// Estatísticas do cache.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Número atual de entradas.
    pub size: usize,

    /// Capacidade máxima.
    pub capacity: usize,

    /// Número de acertos (cache hits).
    pub hits: u64,

    /// Número de erros (cache misses).
    pub misses: u64,
}

impl CacheStats {
    /// Calcula a taxa de acerto.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Cache LRU para resultados de avaliação.
pub struct EvaluationCache {
    cache: LruCache<String, CachedResult>,
    ttl: Duration,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl EvaluationCache {
    /// Cria um novo cache.
    ///
    /// # Argumentos
    /// - `capacity`: Número máximo de entradas
    /// - `ttl`: Tempo de vida das entradas
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        let cap = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            cache: LruCache::new(cap),
            ttl,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Cria um cache com configuração padrão.
    pub fn default_config() -> Self {
        Self::new(100, Duration::from_secs(300)) // 5 minutos TTL
    }

    /// Gera uma chave de cache baseada no código.
    ///
    /// A chave é um hash SHA256 do código normalizado + linguagem + tipo de avaliação.
    pub fn cache_key(code: &str, language: &str, eval_type: &EvaluationType) -> String {
        let normalized = Self::normalize_code(code);
        let eval_type_str = match eval_type {
            EvaluationType::Plan => "plan",
            EvaluationType::Code => "code",
            EvaluationType::Tests => "tests",
            EvaluationType::FinalCheck => "final",
        };

        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        hasher.update(language.as_bytes());
        hasher.update(eval_type_str.as_bytes());

        hex::encode(hasher.finalize())
    }

    /// Normaliza código para cache (remove whitespace extra).
    fn normalize_code(code: &str) -> String {
        code.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Busca no cache.
    ///
    /// Retorna `None` se não encontrado ou se expirado.
    pub fn get(&mut self, key: &str) -> Option<&EvaluationResult> {
        // Primeiro verifica se existe e se está expirado (usando peek para não alterar LRU)
        let is_expired = self.cache.peek(key).map(|c| c.is_expired(self.ttl));

        match is_expired {
            Some(true) => {
                // Expirado - remove e retorna None
                self.cache.pop(key);
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
            Some(false) => {
                // Válido - acessa via get para atualizar LRU
                self.hits.fetch_add(1, Ordering::Relaxed);
                self.cache.get(key).map(|c| &c.result)
            }
            None => {
                // Não encontrado
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Busca por código (gera a chave automaticamente).
    pub fn get_by_code(
        &mut self,
        code: &str,
        language: &str,
        eval_type: &EvaluationType,
    ) -> Option<&EvaluationResult> {
        let key = Self::cache_key(code, language, eval_type);
        self.get(&key)
    }

    /// Insere no cache.
    pub fn insert(&mut self, key: String, result: EvaluationResult) {
        self.cache.put(key, CachedResult::new(result));
    }

    /// Insere por código (gera a chave automaticamente).
    pub fn insert_by_code(
        &mut self,
        code: &str,
        language: &str,
        eval_type: &EvaluationType,
        result: EvaluationResult,
    ) {
        let key = Self::cache_key(code, language, eval_type);
        self.insert(key, result);
    }

    /// Invalida uma entrada específica.
    pub fn invalidate(&mut self, key: &str) {
        self.cache.pop(key);
    }

    /// Limpa todo o cache.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Retorna estatísticas do cache.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            capacity: self.cache.cap().get(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
        }
    }

    /// Remove entradas expiradas.
    pub fn cleanup_expired(&mut self) {
        // Coleta chaves expiradas
        let expired_keys: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, v)| v.is_expired(self.ttl))
            .map(|(k, _)| k.clone())
            .collect();

        // Remove cada uma
        for key in expired_keys {
            self.cache.pop(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::responses::Decision;

    fn create_test_result() -> EvaluationResult {
        EvaluationResult {
            request_id: "test-123".to_string(),
            decision: Decision::Pass,
            score: 85,
            consensus_achieved: true,
            votes: std::collections::HashMap::new(),
            findings: vec![],
            feedback: "Test feedback".to_string(),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = EvaluationCache::cache_key("fn main() {}", "rust", &EvaluationType::Code);
        let key2 = EvaluationCache::cache_key("fn main() {}", "rust", &EvaluationType::Code);
        let key3 = EvaluationCache::cache_key("fn main() {}", "python", &EvaluationType::Code);

        // Mesmo código = mesma chave
        assert_eq!(key1, key2);

        // Linguagem diferente = chave diferente
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_key_normalization() {
        let key1 = EvaluationCache::cache_key("fn main() {}", "rust", &EvaluationType::Code);
        let key2 = EvaluationCache::cache_key("  fn main() {}  ", "rust", &EvaluationType::Code);

        // Whitespace extra é ignorado
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_hit() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(60));
        let result = create_test_result();

        cache.insert("test-key".to_string(), result.clone());

        let cached = cache.get("test-key");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().request_id, "test-123");

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(60));

        let cached = cache.get("nonexistent");
        assert!(cached.is_none());

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_expiration() {
        // TTL de 0 segundos = sempre expirado
        let mut cache = EvaluationCache::new(10, Duration::from_secs(0));
        let result = create_test_result();

        cache.insert("test-key".to_string(), result);

        // Deve retornar None porque expirou
        let cached = cache.get("test-key");
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = EvaluationCache::new(2, Duration::from_secs(60));
        let result = create_test_result();

        cache.insert("key1".to_string(), result.clone());
        cache.insert("key2".to_string(), result.clone());
        cache.insert("key3".to_string(), result); // Deve evictar key1

        assert!(cache.get("key1").is_none()); // Evictado
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(60));
        let result = create_test_result();

        cache.insert("test-key".to_string(), result);
        assert!(cache.get("test-key").is_some());

        cache.invalidate("test-key");
        assert!(cache.get("test-key").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(60));
        let result = create_test_result();

        cache.insert("key1".to_string(), result.clone());
        cache.insert("key2".to_string(), result);

        cache.clear();

        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_none());
        assert_eq!(cache.stats().size, 0);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(60));
        let result = create_test_result();

        cache.insert("key1".to_string(), result);

        cache.get("key1"); // Hit
        cache.get("key2"); // Miss
        cache.get("key1"); // Hit

        let stats = cache.stats();
        assert_eq!(stats.size, 1);
        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_insert_by_code() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(60));
        let result = create_test_result();

        cache.insert_by_code("fn main() {}", "rust", &EvaluationType::Code, result);

        let cached = cache.get_by_code("fn main() {}", "rust", &EvaluationType::Code);
        assert!(cached.is_some());
    }

    #[test]
    fn test_cached_result_is_expired() {
        let result = create_test_result();
        let cached = CachedResult::new(result);

        // Com TTL de 1 hora, não deve estar expirado
        assert!(!cached.is_expired(Duration::from_secs(3600)));

        // Com TTL de 0, deve estar expirado
        assert!(cached.is_expired(Duration::from_secs(0)));
    }
}
