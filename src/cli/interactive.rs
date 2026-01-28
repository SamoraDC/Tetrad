//! Interactive configuration for Tetrad.
//!
//! This module implements interactive configuration using dialoguer.

use std::path::{Path, PathBuf};

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::types::config::{Config, ConsensusRule};
use crate::TetradResult;

/// Runs interactive configuration.
pub fn run_interactive_config(config_path: &Path) -> TetradResult<()> {
    let theme = ColorfulTheme::default();

    println!("\n🔧 Tetrad Interactive Configuration\n");

    // Load existing config or create new one
    let mut config = if config_path.exists() {
        Config::load(config_path)?
    } else {
        println!("Creating new configuration...\n");
        Config::default_config()
    };

    // Main menu
    loop {
        let options = vec![
            "General Settings",
            "Executors (Codex, Gemini, Qwen)",
            "Consensus",
            "ReasoningBank",
            "Cache",
            "Save and Exit",
            "Exit without Saving",
        ];

        let selection = Select::with_theme(&theme)
            .with_prompt("What would you like to configure?")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => configure_general(&theme, &mut config)?,
            1 => configure_executors(&theme, &mut config)?,
            2 => configure_consensus(&theme, &mut config)?,
            3 => configure_reasoning(&theme, &mut config)?,
            4 => configure_cache(&theme, &mut config)?,
            5 => {
                config.save(config_path)?;
                println!("\n✓ Configuration saved to: {}\n", config_path.display());
                break;
            }
            6 => {
                if Confirm::with_theme(&theme)
                    .with_prompt("Are you sure you want to exit without saving?")
                    .default(false)
                    .interact()?
                {
                    println!("\nExiting without saving.\n");
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Configures general options.
fn configure_general(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n📋 General Settings\n");

    // Log level
    let log_levels = vec!["error", "warn", "info", "debug", "trace"];
    let current_idx = log_levels
        .iter()
        .position(|&l| l == config.general.log_level)
        .unwrap_or(2);

    let log_level_idx = Select::with_theme(theme)
        .with_prompt("Log level")
        .items(&log_levels)
        .default(current_idx)
        .interact()?;

    config.general.log_level = log_levels[log_level_idx].to_string();

    // Log format
    let log_formats = vec!["text", "json"];
    let current_format_idx = log_formats
        .iter()
        .position(|&f| f == config.general.log_format)
        .unwrap_or(0);

    let log_format_idx = Select::with_theme(theme)
        .with_prompt("Log format")
        .items(&log_formats)
        .default(current_format_idx)
        .interact()?;

    config.general.log_format = log_formats[log_format_idx].to_string();

    // Timeout
    let timeout: u64 = Input::with_theme(theme)
        .with_prompt("General timeout (seconds)")
        .default(config.general.timeout_secs)
        .interact_text()?;

    config.general.timeout_secs = timeout;

    println!("\n✓ General settings updated.\n");
    Ok(())
}

/// Configures executors.
fn configure_executors(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n🤖 Executor Configuration\n");

    let executors = vec!["Codex", "Gemini", "Qwen", "Back"];

    loop {
        let selection = Select::with_theme(theme)
            .with_prompt("Which executor to configure?")
            .items(&executors)
            .default(0)
            .interact()?;

        match selection {
            0 => configure_single_executor(theme, "Codex", &mut config.executors.codex)?,
            1 => configure_single_executor(theme, "Gemini", &mut config.executors.gemini)?,
            2 => configure_single_executor(theme, "Qwen", &mut config.executors.qwen)?,
            3 => break,
            _ => {}
        }
    }

    Ok(())
}

/// Configures a specific executor.
fn configure_single_executor(
    theme: &ColorfulTheme,
    name: &str,
    executor: &mut crate::types::config::ExecutorConfig,
) -> TetradResult<()> {
    println!("\n⚙️  Configuring {}\n", name);

    // Enabled
    executor.enabled = Confirm::with_theme(theme)
        .with_prompt(format!("{} enabled?", name))
        .default(executor.enabled)
        .interact()?;

    if !executor.enabled {
        println!("{} disabled.\n", name);
        return Ok(());
    }

    // Command
    let command: String = Input::with_theme(theme)
        .with_prompt("Command")
        .default(executor.command.clone())
        .interact_text()?;

    executor.command = command;

    // Args
    let args_str: String = Input::with_theme(theme)
        .with_prompt("Arguments (space separated)")
        .default(executor.args.join(" "))
        .interact_text()?;

    executor.args = args_str.split_whitespace().map(String::from).collect();

    // Timeout
    let timeout: u64 = Input::with_theme(theme)
        .with_prompt("Timeout (seconds)")
        .default(executor.timeout_secs)
        .interact_text()?;

    executor.timeout_secs = timeout;

    // Weight
    let weight: u8 = Input::with_theme(theme)
        .with_prompt("Consensus weight (1-10)")
        .default(executor.weight)
        .interact_text()?;

    executor.weight = weight.clamp(1, 10);

    println!("\n✓ {} configured.\n", name);
    Ok(())
}

/// Configures consensus.
fn configure_consensus(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n🤝 Consensus Configuration\n");

    // Default rule
    let rules = vec![
        "Golden (unanimity)",
        "Strong (3/3 or 2/3 with high confidence)",
        "Weak (simple majority)",
    ];

    let current_idx = match config.consensus.default_rule {
        ConsensusRule::Golden => 0,
        ConsensusRule::Strong => 1,
        ConsensusRule::Weak => 2,
    };

    let rule_idx = Select::with_theme(theme)
        .with_prompt("Default consensus rule")
        .items(&rules)
        .default(current_idx)
        .interact()?;

    config.consensus.default_rule = match rule_idx {
        0 => ConsensusRule::Golden,
        1 => ConsensusRule::Strong,
        _ => ConsensusRule::Weak,
    };

    // Minimum score
    let min_score: u8 = Input::with_theme(theme)
        .with_prompt("Minimum score for approval (0-100)")
        .default(config.consensus.min_score)
        .interact_text()?;

    config.consensus.min_score = min_score.min(100);

    // Max loops
    let max_loops: u8 = Input::with_theme(theme)
        .with_prompt("Maximum number of refinement loops")
        .default(config.consensus.max_loops)
        .interact_text()?;

    config.consensus.max_loops = max_loops;

    println!("\n✓ Consensus configured.\n");
    Ok(())
}

/// Configures ReasoningBank.
fn configure_reasoning(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n🧠 ReasoningBank Configuration\n");

    // Enabled
    config.reasoning.enabled = Confirm::with_theme(theme)
        .with_prompt("ReasoningBank enabled?")
        .default(config.reasoning.enabled)
        .interact()?;

    if !config.reasoning.enabled {
        println!("ReasoningBank disabled.\n");
        return Ok(());
    }

    // Database path
    let db_path: String = Input::with_theme(theme)
        .with_prompt("Database path")
        .default(config.reasoning.db_path.display().to_string())
        .interact_text()?;

    config.reasoning.db_path = PathBuf::from(db_path);

    // Max patterns per query
    let max_patterns: usize = Input::with_theme(theme)
        .with_prompt("Maximum patterns per query")
        .default(config.reasoning.max_patterns_per_query)
        .interact_text()?;

    config.reasoning.max_patterns_per_query = max_patterns;

    // Consolidation interval
    let consolidation_interval: usize = Input::with_theme(theme)
        .with_prompt("Consolidation interval (evaluations)")
        .default(config.reasoning.consolidation_interval)
        .interact_text()?;

    config.reasoning.consolidation_interval = consolidation_interval;

    println!("\n✓ ReasoningBank configured.\n");
    Ok(())
}

/// Configures cache.
fn configure_cache(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n💾 Cache Configuration\n");

    // Enabled
    config.cache.enabled = Confirm::with_theme(theme)
        .with_prompt("Cache enabled?")
        .default(config.cache.enabled)
        .interact()?;

    if !config.cache.enabled {
        println!("Cache disabled.\n");
        return Ok(());
    }

    // Capacity
    let capacity: usize = Input::with_theme(theme)
        .with_prompt("Maximum capacity (number of entries)")
        .default(config.cache.capacity)
        .interact_text()?;

    config.cache.capacity = capacity;

    // TTL
    let ttl: u64 = Input::with_theme(theme)
        .with_prompt("Time to live (seconds)")
        .default(config.cache.ttl_secs)
        .interact_text()?;

    config.cache.ttl_secs = ttl;

    println!("\n✓ Cache configured.\n");
    Ok(())
}

/// Shows configuration summary.
pub fn show_config_summary(config: &Config) {
    println!("\n📊 Configuration Summary\n");
    println!("┌─────────────────────────────────────────┐");
    println!("│ General                                 │");
    println!("├─────────────────────────────────────────┤");
    println!("│ Log level: {:<28} │", config.general.log_level);
    println!("│ Timeout: {:<29}s │", config.general.timeout_secs);
    println!("├─────────────────────────────────────────┤");
    println!("│ Executors                               │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│ Codex:  {} ({:<26}) │",
        if config.executors.codex.enabled {
            "✓"
        } else {
            "✗"
        },
        config.executors.codex.command
    );
    println!(
        "│ Gemini: {} ({:<26}) │",
        if config.executors.gemini.enabled {
            "✓"
        } else {
            "✗"
        },
        config.executors.gemini.command
    );
    println!(
        "│ Qwen:   {} ({:<26}) │",
        if config.executors.qwen.enabled {
            "✓"
        } else {
            "✗"
        },
        config.executors.qwen.command
    );
    println!("├─────────────────────────────────────────┤");
    println!("│ Consensus                               │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│ Rule: {:<33} │",
        format!("{:?}", config.consensus.default_rule)
    );
    println!("│ Min score: {:<28} │", config.consensus.min_score);
    println!("│ Max loops: {:<28} │", config.consensus.max_loops);
    println!("├─────────────────────────────────────────┤");
    println!("│ ReasoningBank                           │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│ Enabled: {:<30} │",
        if config.reasoning.enabled {
            "Yes"
        } else {
            "No"
        }
    );
    if config.reasoning.enabled {
        println!(
            "│ Consolidation: every {:<17} │",
            format!("{} evaluations", config.reasoning.consolidation_interval)
        );
    }
    println!("├─────────────────────────────────────────┤");
    println!("│ Cache                                   │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│ Enabled: {:<30} │",
        if config.cache.enabled { "Yes" } else { "No" }
    );
    if config.cache.enabled {
        println!("│ Capacity: {:<29} │", config.cache.capacity);
        println!("│ TTL: {:<33}s │", config.cache.ttl_secs);
    }
    println!("└─────────────────────────────────────────┘");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_config_summary() {
        let config = Config::default_config();
        // Just verify it doesn't panic
        show_config_summary(&config);
    }
}
