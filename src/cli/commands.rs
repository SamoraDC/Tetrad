//! Implementação dos comandos CLI do Tetrad.

use std::path::PathBuf;

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
fn update_gitignore(target_dir: &PathBuf) -> TetradResult<()> {
    let gitignore_path = target_dir.join(".gitignore");
    let tetrad_entry = ".tetrad/";
    let tetrad_comment = "# Tetrad - local database and cache";

    if gitignore_path.exists() {
        // Read existing content
        let content = std::fs::read_to_string(&gitignore_path)?;

        // Check if it already contains .tetrad/
        if content.lines().any(|line| line.trim() == tetrad_entry || line.trim() == ".tetrad") {
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

/// Inicia o servidor MCP.
pub async fn serve(port: Option<u16>, config: &Config) -> TetradResult<()> {
    use crate::mcp::McpServer;

    tracing::debug!(
        "Configuração carregada: timeout={}s, consenso={:?}",
        config.general.timeout_secs,
        config.consensus.default_rule
    );

    if let Some(p) = port {
        // HTTP transport ainda não implementado
        tracing::warn!("HTTP transport na porta {} ainda não implementado", p);
        eprintln!("Aviso: HTTP transport ainda não suportado. Use stdio (sem --port).");
        return Ok(());
    }

    // Inicia servidor MCP via stdio
    tracing::info!("Iniciando servidor MCP Tetrad via stdio...");

    let mut server = McpServer::new(config.clone())?;
    server.run().await
}

/// Mostra status das CLIs.
pub async fn status(config: &Config) -> TetradResult<()> {
    println!("Verificando status dos executores...\n");

    // Cria executores com configuração do TOML
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
            println!("  ○ {} - desabilitado", name);
            continue;
        }

        let available = executor.is_available().await;
        let status_icon = if available { "✓" } else { "✗" };
        let status_text = if available {
            "disponível"
        } else {
            "não encontrado"
        };

        println!("  {} {} - {}", status_icon, name, status_text);

        if available {
            if let Ok(version) = executor.version().await {
                println!("      versão: {}", version);
            }
        }
    }

    println!();
    println!("Dica: Instale as CLIs faltantes para habilitar o consenso completo.");

    Ok(())
}

/// Configura opções interativamente.
pub async fn config_cmd(config_path: &PathBuf) -> TetradResult<()> {
    use super::interactive::{run_interactive_config, show_config_summary};

    // Mostra resumo antes de editar
    if config_path.exists() {
        let config = Config::load(config_path)?;
        show_config_summary(&config);
    }

    // Executa configuração interativa
    run_interactive_config(config_path)
}

/// Diagnostica problemas de configuração.
pub async fn doctor(config: &Config) -> TetradResult<()> {
    println!("Diagnosticando configuração do Tetrad...\n");

    let mut issues: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    println!("✓ Configuração carregada");

    // Cria executores com configuração do TOML
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
            println!("○ {} está desabilitado no config", name);
            continue;
        }

        enabled_count += 1;

        if executor.is_available().await {
            available_count += 1;
            println!(
                "✓ {} está disponível (comando: {})",
                name,
                executor.command()
            );
        } else {
            warnings.push(format!(
                "{} não está instalado (comando esperado: {})",
                name,
                executor.command()
            ));
        }
    }

    if enabled_count == 0 {
        issues.push("Nenhum executor habilitado no config - consenso não é possível".to_string());
    } else if available_count == 0 {
        issues.push("Nenhum executor disponível - consenso não é possível".to_string());
    } else if available_count < enabled_count {
        warnings.push(format!(
            "Apenas {}/{} executores habilitados estão disponíveis",
            available_count, enabled_count
        ));
    }

    // Resumo
    println!();
    if issues.is_empty() && warnings.is_empty() {
        println!("✓ Tudo OK! Tetrad está pronto para uso.");
    } else {
        if !warnings.is_empty() {
            println!("Avisos:");
            for warning in warnings {
                println!("  ⚠ {}", warning);
            }
        }
        if !issues.is_empty() {
            println!("Problemas:");
            for issue in issues {
                println!("  ✗ {}", issue);
            }
        }
    }

    Ok(())
}

/// Mostra versão.
pub fn version() {
    println!("tetrad {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("MCP de Consenso Quádruplo para Claude Code");
    println!("https://github.com/SamoraDC/tetrad");
}

/// Avalia código manualmente (sem MCP).
pub async fn evaluate(code: &str, language: &str, config: &Config) -> TetradResult<()> {
    use crate::consensus::ConsensusEngine;
    use crate::reasoning::{PatternMatcher, ReasoningBank};
    use crate::types::requests::{EvaluationRequest, EvaluationType};
    use crate::types::responses::ModelVote;
    use std::collections::HashMap;

    println!("Avaliando código...\n");

    // Carrega código de arquivo se começar com @
    let (code_content, file_path_opt) = if let Some(file_path) = code.strip_prefix('@') {
        (
            std::fs::read_to_string(file_path)?,
            Some(file_path.to_string()),
        )
    } else {
        (code.to_string(), None)
    };

    // Detecta linguagem se for "auto"
    let detected_language = if language == "auto" {
        PatternMatcher::detect_language(&code_content)
    } else {
        language.to_string()
    };
    println!("Linguagem: {}", detected_language);

    // Usa configuração do ReasoningBank
    let db_path = &config.reasoning.db_path;

    // Cria diretório do banco se não existir
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Abre o ReasoningBank se habilitado
    let mut bank = if config.reasoning.enabled {
        ReasoningBank::new_with_config(db_path, &config.reasoning).ok()
    } else {
        None
    };

    // RETRIEVE - Busca patterns similares
    if let Some(ref b) = bank {
        let matches = b.retrieve(&code_content, &detected_language);
        if !matches.is_empty() {
            println!("\nPatterns encontrados no ReasoningBank:");
            for m in &matches {
                let icon = match m.pattern.pattern_type {
                    crate::reasoning::PatternType::AntiPattern => "⚠",
                    crate::reasoning::PatternType::GoodPattern => "✓",
                    crate::reasoning::PatternType::Ambiguous => "?",
                };
                println!(
                    "  {} {} - {} (confiança: {:.0}%)",
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

    println!("\nExecutando avaliadores...");

    for executor in executors {
        let name = executor.name();
        if !executor.is_available().await {
            println!("  {} - não disponível, pulando", name);
            continue;
        }

        print!("  {} - avaliando... ", name);

        match executor.evaluate(&request).await {
            Ok(vote) => {
                println!("{:?} (score: {})", vote.vote, vote.score);
                votes.insert(name.to_string(), vote);
            }
            Err(e) => {
                println!("erro: {}", e);
            }
        }
    }

    if votes.is_empty() {
        println!("\nNenhum avaliador disponível. Instale pelo menos uma CLI.");
        return Ok(());
    }

    // Aplica consenso
    let engine = ConsensusEngine::new(config.consensus.clone());
    let result = engine.evaluate(votes, &request_id);

    // JUDGE - Registra resultado no ReasoningBank
    if let Some(ref mut b) = bank {
        let loops_to_consensus = 1; // CLI executa apenas 1 loop
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
                        "\nReasoningBank: {} patterns novos, {} atualizados",
                        judgment.new_patterns_created, judgment.patterns_updated
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Erro ao registrar no ReasoningBank: {}", e);
            }
        }

        // CONSOLIDATE - Verifica se é hora de consolidar
        if let Ok(eval_count) = b.count_trajectories() {
            if eval_count > 0 && eval_count % config.reasoning.consolidation_interval == 0 {
                if let Ok(consolidation) = b.consolidate() {
                    if consolidation.patterns_merged > 0 || consolidation.patterns_pruned > 0 {
                        println!(
                            "ReasoningBank consolidado: {} merged, {} pruned",
                            consolidation.patterns_merged, consolidation.patterns_pruned
                        );
                    }
                }
            }
        }
    }

    // Mostra resultado
    println!("\n{}", "=".repeat(50));
    println!("{}", result.feedback);

    println!("Score final: {}", result.score);
    println!(
        "Consenso: {}",
        if result.consensus_achieved {
            "SIM"
        } else {
            "NÃO"
        }
    );

    Ok(())
}

/// Mostra histórico de avaliações do ReasoningBank.
pub async fn history(limit: usize, config: &Config) -> TetradResult<()> {
    use crate::reasoning::ReasoningBank;

    if !config.reasoning.enabled {
        println!("ReasoningBank está desabilitado na configuração.");
        return Ok(());
    }

    let db_path = &config.reasoning.db_path;

    if !db_path.exists() {
        println!("ReasoningBank ainda não foi criado.");
        println!("Execute 'tetrad evaluate' para começar a coletar dados.");
        return Ok(());
    }

    let bank = ReasoningBank::new_with_config(db_path, &config.reasoning)?;
    let knowledge = bank.distill();

    println!("ReasoningBank - Conhecimento Destilado\n");
    println!("Total de patterns: {}", knowledge.total_patterns);
    println!("Total de trajetórias: {}", knowledge.total_trajectories);
    println!(
        "Média de loops para consenso: {:.2}",
        knowledge.avg_loops_to_consensus
    );

    if !knowledge.top_antipatterns.is_empty() {
        println!("\nTop Anti-patterns:");
        for (i, pattern) in knowledge.top_antipatterns.iter().take(limit).enumerate() {
            println!(
                "  {}. {} ({}) - {} falhas, {:.0}% confiança",
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
                "  {}. {} ({}) - {} sucessos, {:.0}% confiança",
                i + 1,
                pattern.issue_category,
                pattern.language,
                pattern.success_count,
                pattern.confidence * 100.0
            );
        }
    }

    if !knowledge.language_stats.is_empty() {
        println!("\nEstatísticas por linguagem:");
        for (lang, stats) in &knowledge.language_stats {
            println!(
                "  {}: {} avaliações, {:.0}% sucesso, score médio {:.1}",
                lang,
                stats.total_evaluations,
                stats.success_rate * 100.0,
                stats.avg_score
            );
        }
    }

    Ok(())
}

/// Exporta patterns do ReasoningBank.
pub async fn export_patterns(output: &std::path::Path, config: &Config) -> TetradResult<()> {
    use crate::reasoning::ReasoningBank;

    if !config.reasoning.enabled {
        println!("ReasoningBank está desabilitado na configuração.");
        return Ok(());
    }

    let db_path = &config.reasoning.db_path;

    if !db_path.exists() {
        println!("ReasoningBank ainda não foi criado.");
        println!("Nenhum pattern para exportar.");
        return Ok(());
    }

    let bank = ReasoningBank::new_with_config(db_path, &config.reasoning)?;
    bank.export(output)?;

    println!("Patterns exportados para: {}", output.display());

    Ok(())
}

/// Importa patterns para o ReasoningBank.
pub async fn import_patterns(input: &std::path::Path, config: &Config) -> TetradResult<()> {
    use crate::reasoning::ReasoningBank;

    if !config.reasoning.enabled {
        println!("ReasoningBank está desabilitado na configuração.");
        return Ok(());
    }

    if !input.exists() {
        println!("Arquivo não encontrado: {}", input.display());
        return Ok(());
    }

    let db_path = &config.reasoning.db_path;

    // Cria diretório se não existir
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut bank = ReasoningBank::new_with_config(db_path, &config.reasoning)?;
    let result = bank.import(input)?;

    println!("Importação concluída:");
    println!("  Patterns importados: {}", result.imported);
    println!("  Patterns ignorados (já existentes): {}", result.skipped);
    println!("  Patterns mesclados: {}", result.merged);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_version() {
        // Apenas verifica que não causa panic
        version();
    }

    #[tokio::test]
    async fn test_status() {
        // Verifica que status roda sem erros
        let config = Config::default_config();
        let result = status(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_doctor() {
        // Verifica que doctor roda sem erros
        let config = Config::default_config();
        let result = doctor(&config).await;
        assert!(result.is_ok());
    }
}
