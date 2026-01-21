//! # Tetrad
//!
//! MCP de Consenso Quádruplo para Claude Code.
//!
//! Tetrad orquestra três ferramentas CLI de código (Codex, Gemini CLI, Qwen)
//! para avaliar e validar todo trabalho produzido pelo Claude Code.
//!
//! ## Módulos
//!
//! - [`cli`] - Interface de linha de comando
//! - [`mcp`] - Servidor MCP (Model Context Protocol)
//! - [`executors`] - Wrappers para as CLIs (Codex, Gemini, Qwen)
//! - [`consensus`] - Motor de consenso quádruplo
//! - [`reasoning`] - ReasoningBank para aprendizado contínuo
//! - [`hooks`] - Sistema de hooks para customização
//! - [`cache`] - Cache LRU para resultados de avaliação
//! - [`types`] - Tipos compartilhados

pub mod cache;
pub mod cli;
pub mod consensus;
pub mod executors;
pub mod hooks;
pub mod mcp;
pub mod reasoning;
pub mod types;

pub use types::config::Config;
pub use types::errors::{TetradError, TetradResult};
