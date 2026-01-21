//! Executores CLI do Tetrad.
//!
//! Este módulo contém as implementações dos wrappers para as CLIs
//! de avaliação de código: Codex, Gemini e Qwen.

mod base;
mod codex;
mod gemini;
mod qwen;

pub use base::CliExecutor;
pub use codex::CodexExecutor;
pub use gemini::GeminiExecutor;
pub use qwen::QwenExecutor;
