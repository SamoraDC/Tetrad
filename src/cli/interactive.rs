//! Configuração interativa do Tetrad.
//!
//! Este módulo implementa a configuração interativa usando dialoguer.

use std::path::PathBuf;

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::types::config::{Config, ConsensusRule};
use crate::TetradResult;

/// Executa a configuração interativa.
pub fn run_interactive_config(config_path: &PathBuf) -> TetradResult<()> {
    let theme = ColorfulTheme::default();

    println!("\n🔧 Configuração Interativa do Tetrad\n");

    // Carrega config existente ou cria nova
    let mut config = if config_path.exists() {
        Config::load(config_path)?
    } else {
        println!("Criando nova configuração...\n");
        Config::default_config()
    };

    // Menu principal
    loop {
        let options = vec![
            "Configurações Gerais",
            "Executores (Codex, Gemini, Qwen)",
            "Consenso",
            "ReasoningBank",
            "Cache",
            "Salvar e Sair",
            "Sair sem Salvar",
        ];

        let selection = Select::with_theme(&theme)
            .with_prompt("O que deseja configurar?")
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
                println!("\n✓ Configuração salva em: {}\n", config_path.display());
                break;
            }
            6 => {
                if Confirm::with_theme(&theme)
                    .with_prompt("Deseja realmente sair sem salvar?")
                    .default(false)
                    .interact()?
                {
                    println!("\nSaindo sem salvar.\n");
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Configura opções gerais.
fn configure_general(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n📋 Configurações Gerais\n");

    // Log level
    let log_levels = vec!["error", "warn", "info", "debug", "trace"];
    let current_idx = log_levels
        .iter()
        .position(|&l| l == config.general.log_level)
        .unwrap_or(2);

    let log_level_idx = Select::with_theme(theme)
        .with_prompt("Nível de log")
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
        .with_prompt("Formato de log")
        .items(&log_formats)
        .default(current_format_idx)
        .interact()?;

    config.general.log_format = log_formats[log_format_idx].to_string();

    // Timeout
    let timeout: u64 = Input::with_theme(theme)
        .with_prompt("Timeout geral (segundos)")
        .default(config.general.timeout_secs)
        .interact_text()?;

    config.general.timeout_secs = timeout;

    println!("\n✓ Configurações gerais atualizadas.\n");
    Ok(())
}

/// Configura executores.
fn configure_executors(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n🤖 Configuração dos Executores\n");

    let executors = vec!["Codex", "Gemini", "Qwen", "Voltar"];

    loop {
        let selection = Select::with_theme(theme)
            .with_prompt("Qual executor configurar?")
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

/// Configura um executor específico.
fn configure_single_executor(
    theme: &ColorfulTheme,
    name: &str,
    executor: &mut crate::types::config::ExecutorConfig,
) -> TetradResult<()> {
    println!("\n⚙️  Configurando {}\n", name);

    // Habilitado
    executor.enabled = Confirm::with_theme(theme)
        .with_prompt(format!("{} habilitado?", name))
        .default(executor.enabled)
        .interact()?;

    if !executor.enabled {
        println!("{} desabilitado.\n", name);
        return Ok(());
    }

    // Comando
    let command: String = Input::with_theme(theme)
        .with_prompt("Comando")
        .default(executor.command.clone())
        .interact_text()?;

    executor.command = command;

    // Args
    let args_str: String = Input::with_theme(theme)
        .with_prompt("Argumentos (separados por espaço)")
        .default(executor.args.join(" "))
        .interact_text()?;

    executor.args = args_str.split_whitespace().map(String::from).collect();

    // Timeout
    let timeout: u64 = Input::with_theme(theme)
        .with_prompt("Timeout (segundos)")
        .default(executor.timeout_secs)
        .interact_text()?;

    executor.timeout_secs = timeout;

    // Weight
    let weight: u8 = Input::with_theme(theme)
        .with_prompt("Peso no consenso (1-10)")
        .default(executor.weight)
        .interact_text()?;

    executor.weight = weight.clamp(1, 10);

    println!("\n✓ {} configurado.\n", name);
    Ok(())
}

/// Configura consenso.
fn configure_consensus(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n🤝 Configuração de Consenso\n");

    // Regra padrão
    let rules = vec![
        "Golden (unanimidade)",
        "Strong (3/3 ou 2/3 com alta confiança)",
        "Weak (maioria simples)",
    ];

    let current_idx = match config.consensus.default_rule {
        ConsensusRule::Golden => 0,
        ConsensusRule::Strong => 1,
        ConsensusRule::Weak => 2,
    };

    let rule_idx = Select::with_theme(theme)
        .with_prompt("Regra de consenso padrão")
        .items(&rules)
        .default(current_idx)
        .interact()?;

    config.consensus.default_rule = match rule_idx {
        0 => ConsensusRule::Golden,
        1 => ConsensusRule::Strong,
        _ => ConsensusRule::Weak,
    };

    // Score mínimo
    let min_score: u8 = Input::with_theme(theme)
        .with_prompt("Score mínimo para aprovação (0-100)")
        .default(config.consensus.min_score)
        .interact_text()?;

    config.consensus.min_score = min_score.min(100);

    // Max loops
    let max_loops: u8 = Input::with_theme(theme)
        .with_prompt("Número máximo de loops de refinamento")
        .default(config.consensus.max_loops)
        .interact_text()?;

    config.consensus.max_loops = max_loops;

    println!("\n✓ Consenso configurado.\n");
    Ok(())
}

/// Configura ReasoningBank.
fn configure_reasoning(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n🧠 Configuração do ReasoningBank\n");

    // Habilitado
    config.reasoning.enabled = Confirm::with_theme(theme)
        .with_prompt("ReasoningBank habilitado?")
        .default(config.reasoning.enabled)
        .interact()?;

    if !config.reasoning.enabled {
        println!("ReasoningBank desabilitado.\n");
        return Ok(());
    }

    // Caminho do banco
    let db_path: String = Input::with_theme(theme)
        .with_prompt("Caminho do banco de dados")
        .default(config.reasoning.db_path.display().to_string())
        .interact_text()?;

    config.reasoning.db_path = PathBuf::from(db_path);

    // Max patterns por query
    let max_patterns: usize = Input::with_theme(theme)
        .with_prompt("Máximo de patterns por consulta")
        .default(config.reasoning.max_patterns_per_query)
        .interact_text()?;

    config.reasoning.max_patterns_per_query = max_patterns;

    // Intervalo de consolidação
    let consolidation_interval: usize = Input::with_theme(theme)
        .with_prompt("Intervalo de consolidação (avaliações)")
        .default(config.reasoning.consolidation_interval)
        .interact_text()?;

    config.reasoning.consolidation_interval = consolidation_interval;

    println!("\n✓ ReasoningBank configurado.\n");
    Ok(())
}

/// Configura cache.
fn configure_cache(theme: &ColorfulTheme, config: &mut Config) -> TetradResult<()> {
    println!("\n💾 Configuração do Cache\n");

    // Habilitado
    config.cache.enabled = Confirm::with_theme(theme)
        .with_prompt("Cache habilitado?")
        .default(config.cache.enabled)
        .interact()?;

    if !config.cache.enabled {
        println!("Cache desabilitado.\n");
        return Ok(());
    }

    // Capacidade
    let capacity: usize = Input::with_theme(theme)
        .with_prompt("Capacidade máxima (número de entradas)")
        .default(config.cache.capacity)
        .interact_text()?;

    config.cache.capacity = capacity;

    // TTL
    let ttl: u64 = Input::with_theme(theme)
        .with_prompt("Tempo de vida (segundos)")
        .default(config.cache.ttl_secs)
        .interact_text()?;

    config.cache.ttl_secs = ttl;

    println!("\n✓ Cache configurado.\n");
    Ok(())
}

/// Mostra resumo da configuração.
pub fn show_config_summary(config: &Config) {
    println!("\n📊 Resumo da Configuração\n");
    println!("┌─────────────────────────────────────────┐");
    println!("│ Geral                                   │");
    println!("├─────────────────────────────────────────┤");
    println!("│ Log level: {:<28} │", config.general.log_level);
    println!("│ Timeout: {:<29}s │", config.general.timeout_secs);
    println!("├─────────────────────────────────────────┤");
    println!("│ Executores                              │");
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
    println!("│ Consenso                                │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│ Regra: {:<32} │",
        format!("{:?}", config.consensus.default_rule)
    );
    println!("│ Score mínimo: {:<25} │", config.consensus.min_score);
    println!("│ Max loops: {:<28} │", config.consensus.max_loops);
    println!("├─────────────────────────────────────────┤");
    println!("│ ReasoningBank                           │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│ Habilitado: {:<27} │",
        if config.reasoning.enabled {
            "Sim"
        } else {
            "Não"
        }
    );
    if config.reasoning.enabled {
        println!(
            "│ Consolidação: a cada {:<17} │",
            format!("{} avaliações", config.reasoning.consolidation_interval)
        );
    }
    println!("├─────────────────────────────────────────┤");
    println!("│ Cache                                   │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│ Habilitado: {:<27} │",
        if config.cache.enabled { "Sim" } else { "Não" }
    );
    if config.cache.enabled {
        println!("│ Capacidade: {:<27} │", config.cache.capacity);
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
        // Apenas verifica que não causa panic
        show_config_summary(&config);
    }
}
