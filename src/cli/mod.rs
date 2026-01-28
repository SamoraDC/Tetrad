//! Command line interface for Tetrad.

pub mod commands;
pub mod interactive;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Tetrad - Quadruple Consensus CLI for Claude Code.
#[derive(Parser, Debug)]
#[command(name = "tetrad")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Configuration file.
    #[arg(short, long, default_value = "tetrad.toml")]
    pub config: PathBuf,

    /// Verbose mode.
    #[arg(short, long)]
    pub verbose: bool,

    /// Quiet mode.
    #[arg(short, long)]
    pub quiet: bool,

    /// Command to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize configuration in the current directory.
    Init {
        /// Target directory (default: current directory).
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Start the MCP server.
    Serve {
        /// Port for the server (if using HTTP transport).
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Show CLI status (codex, gemini, qwen).
    Status,

    /// Configure options interactively.
    Config,

    /// Diagnose configuration issues.
    Doctor,

    /// Show version.
    Version,

    /// Evaluate code manually (without MCP).
    Evaluate {
        /// Code to evaluate (or file path with @).
        #[arg(short = 'c', long)]
        code: String,

        /// Code language.
        #[arg(short, long, default_value = "auto")]
        language: String,
    },

    /// Show evaluation history from ReasoningBank.
    History {
        /// Limit of entries to show.
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Export patterns from ReasoningBank.
    Export {
        /// Output file.
        #[arg(short, long, default_value = "tetrad-patterns.json")]
        output: PathBuf,
    },

    /// Import patterns into ReasoningBank.
    Import {
        /// Input file.
        input: PathBuf,
    },
}
