//! Configuração do Tetrad.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::TetradResult;

/// Configuração principal do Tetrad.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configurações gerais.
    #[serde(default)]
    pub general: GeneralConfig,

    /// Configurações dos executores.
    #[serde(default)]
    pub executors: ExecutorsConfig,

    /// Configurações de consenso.
    #[serde(default)]
    pub consensus: ConsensusConfig,

    /// Configurações do ReasoningBank.
    #[serde(default)]
    pub reasoning: ReasoningConfig,

    /// Configurações do cache.
    #[serde(default)]
    pub cache: CacheConfig,
}

/// Configurações gerais.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Nível de log (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Formato de log (text, json).
    #[serde(default = "default_log_format")]
    pub log_format: String,

    /// Timeout padrão para operações (em segundos).
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            log_format: default_log_format(),
            timeout_secs: default_timeout(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "text".to_string()
}

fn default_timeout() -> u64 {
    60
}

/// Configurações dos executores CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorsConfig {
    /// Configuração do Codex.
    #[serde(default)]
    pub codex: ExecutorConfig,

    /// Configuração do Gemini.
    #[serde(default)]
    pub gemini: ExecutorConfig,

    /// Configuração do Qwen.
    #[serde(default)]
    pub qwen: ExecutorConfig,
}

impl Default for ExecutorsConfig {
    fn default() -> Self {
        Self {
            // Codex: usa exec --json para modo não-interativo
            codex: ExecutorConfig::new("codex", &["exec", "--json"]),
            // Gemini: -o json para formato de saída, prompt é posicional
            gemini: ExecutorConfig::new("gemini", &["-o", "json"]),
            // Qwen: prompt é argumento posicional
            qwen: ExecutorConfig::new("qwen", &[]),
        }
    }
}

/// Configuração de um executor específico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// Habilitado.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Comando para executar.
    pub command: String,

    /// Argumentos padrão.
    #[serde(default)]
    pub args: Vec<String>,

    /// Timeout específico (em segundos).
    #[serde(default = "default_executor_timeout")]
    pub timeout_secs: u64,

    /// Peso no consenso (1-10).
    #[serde(default = "default_weight")]
    pub weight: u8,
}

impl ExecutorConfig {
    /// Cria uma nova configuração de executor.
    pub fn new(command: &str, args: &[&str]) -> Self {
        Self {
            enabled: true,
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            timeout_secs: default_executor_timeout(),
            weight: default_weight(),
        }
    }
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            command: String::new(),
            args: Vec::new(),
            timeout_secs: default_executor_timeout(),
            weight: default_weight(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_executor_timeout() -> u64 {
    30
}

fn default_weight() -> u8 {
    5
}

/// Configurações de consenso.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Regra de consenso padrão.
    #[serde(default = "default_consensus_rule")]
    pub default_rule: ConsensusRule,

    /// Score mínimo para passar (0-100).
    #[serde(default = "default_min_score")]
    pub min_score: u8,

    /// Número máximo de loops de refinamento.
    #[serde(default = "default_max_loops")]
    pub max_loops: u8,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            default_rule: default_consensus_rule(),
            min_score: default_min_score(),
            max_loops: default_max_loops(),
        }
    }
}

fn default_consensus_rule() -> ConsensusRule {
    ConsensusRule::Strong
}

fn default_min_score() -> u8 {
    70
}

fn default_max_loops() -> u8 {
    3
}

/// Regras de consenso disponíveis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusRule {
    /// Regra de Ouro: unanimidade necessária.
    Golden,
    /// Consenso Fraco: 2+ votos necessários.
    Weak,
    /// Consenso Forte: 3/3 votos necessários.
    Strong,
}

/// Configurações do ReasoningBank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// Habilitado.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Caminho do banco de dados SQLite.
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,

    /// Número máximo de patterns por consulta.
    #[serde(default = "default_max_patterns")]
    pub max_patterns_per_query: usize,

    /// Intervalo de consolidação (a cada N avaliações).
    #[serde(default = "default_consolidation_interval")]
    pub consolidation_interval: usize,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            db_path: default_db_path(),
            max_patterns_per_query: default_max_patterns(),
            consolidation_interval: default_consolidation_interval(),
        }
    }
}

fn default_db_path() -> PathBuf {
    PathBuf::from(".tetrad/tetrad.db")
}

fn default_max_patterns() -> usize {
    10
}

fn default_consolidation_interval() -> usize {
    100
}

/// Configurações do cache LRU.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Habilitado.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Capacidade máxima do cache (número de entradas).
    #[serde(default = "default_cache_capacity")]
    pub capacity: usize,

    /// Tempo de vida das entradas em segundos.
    #[serde(default = "default_cache_ttl")]
    pub ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            capacity: default_cache_capacity(),
            ttl_secs: default_cache_ttl(),
        }
    }
}

fn default_cache_capacity() -> usize {
    1000
}

fn default_cache_ttl() -> u64 {
    300 // 5 minutos
}

impl Config {
    /// Carrega configuração de um arquivo TOML.
    pub fn load<P: AsRef<Path>>(path: P) -> TetradResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Salva configuração em um arquivo TOML.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> TetradResult<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Cria configuração padrão.
    pub fn default_config() -> Self {
        Self {
            general: GeneralConfig::default(),
            executors: ExecutorsConfig::default(),
            consensus: ConsensusConfig::default(),
            reasoning: ReasoningConfig::default(),
            cache: CacheConfig::default(),
        }
    }

    /// Tenta carregar configuração do diretório atual ou usa padrão.
    pub fn load_or_default() -> Self {
        Self::load("tetrad.toml").unwrap_or_else(|_| Self::default_config())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default_config()
    }
}
