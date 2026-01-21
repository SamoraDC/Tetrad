//! ReasoningBank - Sistema de aprendizado contínuo do Tetrad.
//!
//! Este módulo implementa o ciclo RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
//! para aprendizado contínuo baseado em avaliações de código.
//!
//! ## Componentes
//!
//! - **ReasoningBank**: Banco de dados SQLite que armazena patterns e trajetórias
//! - **PatternMatcher**: Utilitários para matching e análise de código
//! - **Export/Import**: Compartilhamento de conhecimento entre instalações

mod bank;
mod export;
mod patterns;

pub use bank::{
    ConsolidationResult, DistilledKnowledge, JudgmentResult, LanguageStats, MatchType, Pattern,
    PatternMatch, PatternType, ReasoningBank,
};
pub use export::{format_knowledge, ImportResult, ReasoningBankExport};
pub use patterns::PatternMatcher;
