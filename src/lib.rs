//! # Tetrad
//!
//! MCP de Consenso Quádruplo para Claude Code.
//!
//! Tetrad orquestra três ferramentas CLI de código (Codex, Gemini CLI, Qwen)
//! para avaliar e validar todo trabalho produzido pelo Claude Code.

pub mod cli;
pub mod consensus;
pub mod executors;
pub mod hooks;
pub mod mcp;
pub mod reasoning;
pub mod types;

pub use types::config::Config;
pub use types::errors::{TetradError, TetradResult};
