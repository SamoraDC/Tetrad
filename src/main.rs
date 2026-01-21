use clap::Parser;
use tetrad::cli::{Cli, Commands};
use tetrad::TetradResult;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> TetradResult<()> {
    // Inicializa logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("tetrad=info".parse().unwrap()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            tetrad::cli::commands::init(path).await?;
        }
        Commands::Serve { port } => {
            tetrad::cli::commands::serve(port).await?;
        }
        Commands::Status => {
            tetrad::cli::commands::status().await?;
        }
        Commands::Config => {
            tetrad::cli::commands::config().await?;
        }
        Commands::Doctor => {
            tetrad::cli::commands::doctor().await?;
        }
        Commands::Version => {
            tetrad::cli::commands::version();
        }
    }

    Ok(())
}
