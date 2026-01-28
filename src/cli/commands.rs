//! CLI commands implementation for Tetrad.

use std::path::{Path, PathBuf};

use crate::executors::{CliExecutor, CodexExecutor, GeminiExecutor, QwenExecutor};
use crate::types::config::Config;
use crate::TetradResult;

/// Initializes configuration in the specified directory.
pub async fn init(path: Option<PathBuf>) -> TetradResult<()> {
    let target_dir = path.unwrap_or_else(|| PathBuf::from("."));

    // Create directory if it doesn't exist
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)?;
        tracing::info!("Directory created: {}", target_dir.display());
    }

    let config_path = target_dir.join("tetrad.toml");

    if config_path.exists() {
        println!("Configuration already exists at: {}", config_path.display());
        println!("Use 'tetrad config' to modify.");
        return Ok(());
    }

    // Create .tetrad/ directory for the database
    let tetrad_dir = target_dir.join(".tetrad");
    if !tetrad_dir.exists() {
        std::fs::create_dir_all(&tetrad_dir)?;
        tracing::info!(".tetrad/ directory created");
    }

    // Update .gitignore to ignore .tetrad/
    update_gitignore(&target_dir)?;

    // Create default configuration
    let config = Config::default_config();
    config.save(&config_path)?;

    println!("Tetrad initialized successfully!");
    println!("Configuration created at: {}", config_path.display());
    println!("Data directory: .tetrad/");
    println!();
    println!("Next steps:");
    println!("  1. Check if CLIs are installed: tetrad status");
    println!("  2. Configure options: tetrad config");
    println!("  3. Add to Claude Code: claude mcp add tetrad -- tetrad serve");

    Ok(())
}

/// Updates or creates .gitignore to include .tetrad/
fn update_gitignore(target_dir: &Path) -> TetradResult<()> {
    let gitignore_path = target_dir.join(".gitignore");
    let tetrad_entry = ".tetrad/";
    let tetrad_comment = "# Tetrad - local database and cache";

    if gitignore_path.exists() {
        // Read existing content
        let content = std::fs::read_to_string(&gitignore_path)?;

        // Check if it already contains .tetrad/
        if content
            .lines()
            .any(|line| line.trim() == tetrad_entry || line.trim() == ".tetrad")
        {
            tracing::debug!(".gitignore already contains .tetrad/");
            return Ok(());
        }

        // Append to end of file
        let mut new_content = content.trim_end().to_string();
        if !new_content.is_empty() {
            new_content.push_str("\n\n");
        }
        new_content.push_str(tetrad_comment);
        new_content.push('\n');
        new_content.push_str(tetrad_entry);
        new_content.push('\n');

        std::fs::write(&gitignore_path, new_content)?;
        println!(".gitignore updated with .tetrad/");
    } else {
        // Create new .gitignore
        let content = format!("{}\n{}\n", tetrad_comment, tetrad_entry);
        std::fs::write(&gitignore_path, content)?;
        println!(".gitignore created with .tetrad/");
    }

    Ok(())
}

/// Starts the MCP server.
pub async fn serve(port: Option<u16>, config: &Config) -> TetradResult<()> {
    use crate::mcp::McpServer;

    tracing::debug!(
        "Configuration loaded: timeout={}s, consensus={:?}",
        config.general.timeout_secs,
        config.consensus.default_rule
    );

    if let Some(p) = port {
        // HTTP transport not yet implemented
        tracing::warn!("HTTP transport on port {} not yet implemented", p);
        eprintln!("Warning: HTTP transport not yet supported. Use stdio (without --port).");
        return Ok(());
    }

    // Start MCP server via stdio
    tracing::info!("Starting Tetrad MCP server via stdio...");

    let mut server = McpServer::new(config.clone())?;
    server.run().await
}

/// Shows CLI status.
pub async fn status(config: &Config) -> TetradResult<()> {
    println!("Checking executor status...\n");

    // Create executors with TOML configuration
    let executors: Vec<(Box<dyn CliExecutor>, bool)> = vec![
        (
            Box::new(CodexExecutor::from_config(&config.executors.codex)),
            config.executors.codex.enabled,
        ),
        (
            Box::new(GeminiExecutor::from_config(&config.executors.gemini)),
            config.executors.gemini.enabled,
        ),
        (
            Box::new(QwenExecutor::from_config(&config.executors.qwen)),
            config.executors.qwen.enabled,
        ),
    ];

    for (executor, enabled) in executors {
        let name = executor.name();

        if !enabled {
            println!("  ○ {} - disabled", name);
            continue;
        }

        let available = executor.is_available().await;
        let status_icon = if available { "✓" } else { "✗" };
        let status_text = if available {
            "available"
        } else {
            "not found"
        };

        println!("  {} {} - {}", status_icon, name, status_text);

        if available {
            if let Ok(version) = executor.version().await {
                println!("      version: {}", version);
            }
        }
    }

    println!();
    println!("Tip: Install missing CLIs to enable full consensus.");

    Ok(())
}

/// Configures options interactively.
pub async fn config_cmd(config_path: &Path) -> TetradResult<()> {
    use super::interactive::{run_interactive_config, show_config_summary};

    // Show summary before editing
    if config_path.exists() {
        let config = Config::load(config_path)?;
        show_config_summary(&config);
    }

    // Executa configuração interativa
    run_interactive_config(config_path)
}

/// Diagnoses configuration issues.
pub async fn doctor(config: &Config) -> TetradResult<()> {
    println!("Diagnosing Tetrad configuration...\n");

    let mut issues: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    println!("✓ Configuration loaded");

    // Create executors with TOML configuration
    let executors: Vec<(Box<dyn CliExecutor>, bool, &str)> = vec![
        (
            Box::new(CodexExecutor::from_config(&config.executors.codex)),
            config.executors.codex.enabled,
            "Codex",
        ),
        (
            Box::new(GeminiExecutor::from_config(&config.executors.gemini)),
            config.executors.gemini.enabled,
            "Gemini",
        ),
        (
            Box::new(QwenExecutor::from_config(&config.executors.qwen)),
            config.executors.qwen.enabled,
            "Qwen",
        ),
    ];

    let mut available_count = 0;
    let mut enabled_count = 0;

    for (executor, enabled, name) in executors {
        if !enabled {
            println!("○ {} is disabled in config", name);
            continue;
        }

        enabled_count += 1;

        if executor.is_available().await {
            available_count += 1;
            println!(
                "✓ {} is available (command: {})",
                name,
                executor.command()
            );
        } else {
            warnings.push(format!(
                "{} is not installed (expected command: {})",
                name,
                executor.command()
            ));
        }
    }

    if enabled_count == 0 {
        issues.push("No executor enabled in config - consensus is not possible".to_string());
    } else if available_count == 0 {
        issues.push("No executor available - consensus is not possible".to_string());
    } else if available_count < enabled_count {
        warnings.push(format!(
            "Only {}/{} enabled executors are available",
            available_count, enabled_count
        ));
    }

    // Summary
    println!();
    if issues.is_empty() && warnings.is_empty() {
        println!("✓ All OK! Tetrad is ready to use.");
    } else {
        if !warnings.is_empty() {
            println!("Warnings:");
            for warning in warnings {
                println!("  ⚠ {}", warning);
            }
        }
        if !issues.is_empty() {
            println!("Issues:");
            for issue in issues {
                println!("  ✗ {}", issue);
            }
        }
    }

    Ok(())
}

/// Shows version.
pub fn version() {
    println!("tetrad {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Quadruple Consensus MCP for Claude Code");
    println!("https://github.com/SamoraDC/tetrad");
}

/// Evaluates code manually (without MCP).
pub async fn evaluate(code: &str, language: &str, config: &Config) -> TetradResult<()> {
    use crate::consensus::ConsensusEngine;
    use crate::reasoning::{PatternMatcher, ReasoningBank};
    use crate::types::requests::{EvaluationRequest, EvaluationType};
    use crate::types::responses::ModelVote;
    use std::collections::HashMap;

    println!("Evaluating code...\n");

    // Load code from file if starts with @
    let (code_content, file_path_opt) = if let Some(file_path) = code.strip_prefix('@') {
        (
            std::fs::read_to_string(file_path)?,
            Some(file_path.to_string()),
        )
    } else {
        (code.to_string(), None)
    };

    // Detect language if "auto"
    let detected_language = if language == "auto" {
        PatternMatcher::detect_language(&code_content)
    } else {
        language.to_string()
    };
    println!("Language: {}", detected_language);

    // Use ReasoningBank configuration
    let db_path = &config.reasoning.db_path;

    // Create database directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Open ReasoningBank if enabled
    let mut bank = if config.reasoning.enabled {
        ReasoningBank::new_with_config(db_path, &config.reasoning).ok()
    } else {
        None
    };

    // RETRIEVE - Search for similar patterns
    if let Some(ref b) = bank {
        let matches = b.retrieve(&code_content, &detected_language);
        if !matches.is_empty() {
            println!("\nPatterns found in ReasoningBank:");
            for m in &matches {
                let icon = match m.pattern.pattern_type {
                    crate::reasoning::PatternType::AntiPattern => "⚠",
                    crate::reasoning::PatternType::GoodPattern => "✓",
                    crate::reasoning::PatternType::Ambiguous => "?",
                };
                println!(
                    "  {} {} - {} (confidence: {:.0}%)",
                    icon,
                    m.pattern.issue_category,
                    m.pattern.description,
                    m.pattern.confidence * 100.0
                );
            }
        }
    }

    // Cria executores e coleta votos
    let executors: Vec<Box<dyn CliExecutor>> = vec![
        Box::new(CodexExecutor::from_config(&config.executors.codex)),
        Box::new(GeminiExecutor::from_config(&config.executors.gemini)),
        Box::new(QwenExecutor::from_config(&config.executors.qwen)),
    ];

    let mut votes: HashMap<String, ModelVote> = HashMap::new();
    let request_id = format!("eval-{}", chrono::Utc::now().timestamp());

    // Cria requisição de avaliação
    let request = EvaluationRequest {
        request_id: request_id.clone(),
        code: code_content.clone(),
        language: detected_language.clone(),
        evaluation_type: EvaluationType::Code,
        context: None,
        file_path: file_path_opt,
    };

    println!("\nRunning evaluators...");

    for executor in executors {
        let name = executor.name();
        if !executor.is_available().await {
            println!("  {} - not available, skipping", name);
            continue;
        }

        print!("  {} - evaluating... ", name);

        match executor.evaluate(&request).await {
            Ok(vote) => {
                println!("{:?} (score: {})", vote.vote, vote.score);
                votes.insert(name.to_string(), vote);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    if votes.is_empty() {
        println!("\nNo evaluator available. Install at least one CLI.");
        return Ok(());
    }

    // Aplica consenso
    let engine = ConsensusEngine::new(config.consensus.clone());
    let result = engine.evaluate(votes, &request_id);

    // JUDGE - Register result in ReasoningBank
    if let Some(ref mut b) = bank {
        let loops_to_consensus = 1; // CLI runs only 1 loop
        match b.judge(
            &request_id,
            &code_content,
            &detected_language,
            &result,
            loops_to_consensus,
            config.consensus.max_loops,
        ) {
            Ok(judgment) => {
                if judgment.new_patterns_created > 0 || judgment.patterns_updated > 0 {
                    println!(
                        "\nReasoningBank: {} new patterns, {} updated",
                        judgment.new_patterns_created, judgment.patterns_updated
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Error registering in ReasoningBank: {}", e);
            }
        }

        // CONSOLIDATE - Check if it's time to consolidate
        if let Ok(eval_count) = b.count_trajectories() {
            if eval_count > 0 && eval_count % config.reasoning.consolidation_interval == 0 {
                if let Ok(consolidation) = b.consolidate() {
                    if consolidation.patterns_merged > 0 || consolidation.patterns_pruned > 0 {
                        println!(
                            "ReasoningBank consolidated: {} merged, {} pruned",
                            consolidation.patterns_merged, consolidation.patterns_pruned
                        );
                    }
                }
            }
        }
    }

    // Show result
    println!("\n{}", "=".repeat(50));
    println!("{}", result.feedback);

    println!("Final score: {}", result.score);
    println!(
        "Consensus: {}",
        if result.consensus_achieved {
            "YES"
        } else {
            "NO"
        }
    );

    Ok(())
}

/// Shows evaluation history from ReasoningBank.
pub async fn history(limit: usize, config: &Config) -> TetradResult<()> {
    use crate::reasoning::ReasoningBank;

    if !config.reasoning.enabled {
        println!("ReasoningBank is disabled in configuration.");
        return Ok(());
    }

    let db_path = &config.reasoning.db_path;

    if !db_path.exists() {
        println!("ReasoningBank has not been created yet.");
        println!("Run 'tetrad evaluate' to start collecting data.");
        return Ok(());
    }

    let bank = ReasoningBank::new_with_config(db_path, &config.reasoning)?;
    let knowledge = bank.distill();

    println!("ReasoningBank - Distilled Knowledge\n");
    println!("Total patterns: {}", knowledge.total_patterns);
    println!("Total trajectories: {}", knowledge.total_trajectories);
    println!(
        "Average loops to consensus: {:.2}",
        knowledge.avg_loops_to_consensus
    );

    if !knowledge.top_antipatterns.is_empty() {
        println!("\nTop Anti-patterns:");
        for (i, pattern) in knowledge.top_antipatterns.iter().take(limit).enumerate() {
            println!(
                "  {}. {} ({}) - {} failures, {:.0}% confidence",
                i + 1,
                pattern.issue_category,
                pattern.language,
                pattern.failure_count,
                pattern.confidence * 100.0
            );
        }
    }

    if !knowledge.top_good_patterns.is_empty() {
        println!("\nTop Good Patterns:");
        for (i, pattern) in knowledge.top_good_patterns.iter().take(limit).enumerate() {
            println!(
                "  {}. {} ({}) - {} successes, {:.0}% confidence",
                i + 1,
                pattern.issue_category,
                pattern.language,
                pattern.success_count,
                pattern.confidence * 100.0
            );
        }
    }

    if !knowledge.language_stats.is_empty() {
        println!("\nStatistics by language:");
        for (lang, stats) in &knowledge.language_stats {
            println!(
                "  {}: {} evaluations, {:.0}% success, avg score {:.1}",
                lang,
                stats.total_evaluations,
                stats.success_rate * 100.0,
                stats.avg_score
            );
        }
    }

    Ok(())
}

/// Exports patterns from ReasoningBank.
pub async fn export_patterns(output: &std::path::Path, config: &Config) -> TetradResult<()> {
    use crate::reasoning::ReasoningBank;

    if !config.reasoning.enabled {
        println!("ReasoningBank is disabled in configuration.");
        return Ok(());
    }

    let db_path = &config.reasoning.db_path;

    if !db_path.exists() {
        println!("ReasoningBank has not been created yet.");
        println!("No patterns to export.");
        return Ok(());
    }

    let bank = ReasoningBank::new_with_config(db_path, &config.reasoning)?;
    bank.export(output)?;

    println!("Patterns exported to: {}", output.display());

    Ok(())
}

/// Imports patterns into ReasoningBank.
pub async fn import_patterns(input: &std::path::Path, config: &Config) -> TetradResult<()> {
    use crate::reasoning::ReasoningBank;

    if !config.reasoning.enabled {
        println!("ReasoningBank is disabled in configuration.");
        return Ok(());
    }

    if !input.exists() {
        println!("File not found: {}", input.display());
        return Ok(());
    }

    let db_path = &config.reasoning.db_path;

    // Create directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut bank = ReasoningBank::new_with_config(db_path, &config.reasoning)?;
    let result = bank.import(input)?;

    println!("Import completed:");
    println!("  Patterns imported: {}", result.imported);
    println!("  Patterns skipped (already exist): {}", result.skipped);
    println!("  Patterns merged: {}", result.merged);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_version() {
        // Just verify it doesn't panic
        version();
    }

    #[tokio::test]
    async fn test_status() {
        // Verify status runs without errors
        let config = Config::default_config();
        let result = status(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_doctor() {
        // Verify doctor runs without errors
        let config = Config::default_config();
        let result = doctor(&config).await;
        assert!(result.is_ok());
    }
}
