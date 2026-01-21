//! Interface de linha de comando do Tetrad.

pub mod commands;
pub mod interactive;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Tetrad - CLI de Consenso Quádruplo para Claude Code.
#[derive(Parser, Debug)]
#[command(name = "tetrad")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Arquivo de configuração.
    #[arg(short, long, default_value = "tetrad.toml")]
    pub config: PathBuf,

    /// Modo verbose.
    #[arg(short, long)]
    pub verbose: bool,

    /// Modo silencioso.
    #[arg(short, long)]
    pub quiet: bool,

    /// Comando a executar.
    #[command(subcommand)]
    pub command: Commands,
}

/// Comandos disponíveis.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inicializa configuração no diretório atual.
    Init {
        /// Diretório de destino (padrão: diretório atual).
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Inicia o servidor MCP.
    Serve {
        /// Porta para o servidor (se usar HTTP transport).
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Mostra status das CLIs (codex, gemini, qwen).
    Status,

    /// Configura opções interativamente.
    Config,

    /// Diagnostica problemas de configuração.
    Doctor,

    /// Mostra versão.
    Version,

    /// Avalia código manualmente (sem MCP).
    Evaluate {
        /// Código a avaliar (ou caminho para arquivo com @).
        #[arg(short = 'c', long)]
        code: String,

        /// Linguagem do código.
        #[arg(short, long, default_value = "auto")]
        language: String,
    },

    /// Mostra histórico de avaliações do ReasoningBank.
    History {
        /// Limite de entradas a mostrar.
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Exporta patterns do ReasoningBank.
    Export {
        /// Arquivo de destino.
        #[arg(short, long, default_value = "tetrad-patterns.json")]
        output: PathBuf,
    },

    /// Importa patterns para o ReasoningBank.
    Import {
        /// Arquivo de origem.
        input: PathBuf,
    },
}
