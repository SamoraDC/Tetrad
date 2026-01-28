//! Configuration for Tetrad.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::TetradResult;

/// Main configuration for Tetrad.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings.
    #[serde(default)]
    pub general: GeneralConfig,

    /// Executor settings.
    #[serde(default)]
    pub executors: ExecutorsConfig,

    /// Consensus settings.
    #[serde(default)]
    pub consensus: ConsensusConfig,

    /// ReasoningBank settings.
    #[serde(default)]
    pub reasoning: ReasoningConfig,

    /// Cache settings.
    #[serde(default)]
    pub cache: CacheConfig,
}

/// General settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Log format (text, json).
    #[serde(default = "default_log_format")]
    pub log_format: String,

    /// Default timeout for operations (in seconds).
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

/// CLI executor settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorsConfig {
    /// Codex configuration.
    #[serde(default)]
    pub codex: ExecutorConfig,

    /// Gemini configuration.
    #[serde(default)]
    pub gemini: ExecutorConfig,

    /// Qwen configuration.
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

/// Configuration for a specific executor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// Enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Command to execute.
    pub command: String,

    /// Default arguments.
    #[serde(default)]
    pub args: Vec<String>,

    /// Specific timeout (in seconds).
    #[serde(default = "default_executor_timeout")]
    pub timeout_secs: u64,

    /// Weight in consensus (1-10).
    #[serde(default = "default_weight")]
    pub weight: u8,
}

impl ExecutorConfig {
    /// Creates a new executor configuration.
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

/// Consensus settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Default consensus rule.
    #[serde(default = "default_consensus_rule")]
    pub default_rule: ConsensusRule,

    /// Minimum score to pass (0-100).
    #[serde(default = "default_min_score")]
    pub min_score: u8,

    /// Maximum number of refinement loops.
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

/// Available consensus rules.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusRule {
    /// Golden Rule: unanimity required.
    Golden,
    /// Weak Consensus: 2+ votes required.
    Weak,
    /// Strong Consensus: 3/3 votes required.
    Strong,
}

/// ReasoningBank settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// Enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// SQLite database path.
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,

    /// Maximum number of patterns per query.
    #[serde(default = "default_max_patterns")]
    pub max_patterns_per_query: usize,

    /// Consolidation interval (every N evaluations).
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

/// LRU cache settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum cache capacity (number of entries).
    #[serde(default = "default_cache_capacity")]
    pub capacity: usize,

    /// Entry time to live in seconds.
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
    300 // 5 minutes
}

impl Config {
    /// Loads configuration from a TOML file.
    pub fn load<P: AsRef<Path>>(path: P) -> TetradResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Saves configuration to a TOML file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> TetradResult<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Creates default configuration.
    pub fn default_config() -> Self {
        Self {
            general: GeneralConfig::default(),
            executors: ExecutorsConfig::default(),
            consensus: ConsensusConfig::default(),
            reasoning: ReasoningConfig::default(),
            cache: CacheConfig::default(),
        }
    }

    /// Tries to load configuration from current directory or uses default.
    pub fn load_or_default() -> Self {
        Self::load("tetrad.toml").unwrap_or_else(|_| Self::default_config())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default_config()
    }
}
