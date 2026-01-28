use clap::Parser;
use tetrad::cli::{Cli, Commands};
use tetrad::types::config::Config;
use tetrad::TetradResult;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> TetradResult<()> {
    let cli = Cli::parse();

    // Load configuration first (no logging yet)
    let config = if cli.config.exists() {
        Config::load(&cli.config).unwrap_or_else(|_| Config::default_config())
    } else {
        Config::default_config()
    };

    // Determine log level: CLI flags take precedence over config
    let log_level = if cli.quiet {
        "error".to_string()
    } else if cli.verbose {
        "debug".to_string()
    } else {
        // Use config value if no flag was specified
        config.general.log_level.clone()
    };

    // Initialize logging with appropriate level
    let filter = EnvFilter::from_default_env().add_directive(
        format!("tetrad={}", log_level)
            .parse()
            .unwrap_or_else(|_| "tetrad=info".parse().expect("fallback directive is valid")),
    );

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(filter)
        .init();

    tracing::debug!("Configuration loaded from: {}", cli.config.display());

    match cli.command {
        Commands::Init { path } => {
            tetrad::cli::commands::init(path).await?;
        }
        Commands::Serve { port } => {
            tetrad::cli::commands::serve(port, &config).await?;
        }
        Commands::Status => {
            tetrad::cli::commands::status(&config).await?;
        }
        Commands::Config => {
            tetrad::cli::commands::config_cmd(&cli.config).await?;
        }
        Commands::Doctor => {
            tetrad::cli::commands::doctor(&config).await?;
        }
        Commands::Version => {
            tetrad::cli::commands::version();
        }
        Commands::Evaluate { code, language } => {
            tetrad::cli::commands::evaluate(&code, &language, &config).await?;
        }
        Commands::History { limit } => {
            tetrad::cli::commands::history(limit, &config).await?;
        }
        Commands::Export { output } => {
            tetrad::cli::commands::export_patterns(&output, &config).await?;
        }
        Commands::Import { input } => {
            tetrad::cli::commands::import_patterns(&input, &config).await?;
        }
    }

    Ok(())
}
