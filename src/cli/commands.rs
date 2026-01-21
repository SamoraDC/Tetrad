//! Implementação dos comandos CLI do Tetrad.

use std::path::PathBuf;

use crate::executors::{CliExecutor, CodexExecutor, GeminiExecutor, QwenExecutor};
use crate::types::config::Config;
use crate::TetradResult;

/// Inicializa configuração no diretório especificado.
pub async fn init(path: Option<PathBuf>) -> TetradResult<()> {
    let target_dir = path.unwrap_or_else(|| PathBuf::from("."));
    let config_path = target_dir.join("tetrad.toml");

    if config_path.exists() {
        println!("Configuração já existe em: {}", config_path.display());
        println!("Use 'tetrad config' para modificar.");
        return Ok(());
    }

    // Cria configuração padrão
    let config = Config::default_config();
    config.save(&config_path)?;

    println!("Tetrad inicializado com sucesso!");
    println!("Configuração criada em: {}", config_path.display());
    println!();
    println!("Próximos passos:");
    println!("  1. Verifique se as CLIs estão instaladas: tetrad status");
    println!("  2. Configure as opções: tetrad config");
    println!("  3. Adicione ao Claude Code: claude mcp add tetrad -- tetrad serve");

    Ok(())
}

/// Inicia o servidor MCP.
pub async fn serve(port: Option<u16>) -> TetradResult<()> {
    let _config = Config::load_or_default();

    if let Some(p) = port {
        println!("Iniciando servidor MCP na porta {}...", p);
        println!("(HTTP transport ainda não implementado)");
    } else {
        println!("Iniciando servidor MCP via stdio...");
        println!("(Servidor MCP será implementado na Fase 4)");
    }

    // TODO: Implementar servidor MCP na Fase 4
    // Por enquanto, apenas mostra mensagem

    Ok(())
}

/// Mostra status das CLIs.
pub async fn status() -> TetradResult<()> {
    println!("Verificando status dos executores...\n");

    let executors: Vec<Box<dyn CliExecutor>> = vec![
        Box::new(CodexExecutor::new()),
        Box::new(GeminiExecutor::new()),
        Box::new(QwenExecutor::new()),
    ];

    for executor in executors {
        let name = executor.name();
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
pub async fn config() -> TetradResult<()> {
    println!("Configuração interativa do Tetrad\n");

    let config_path = PathBuf::from("tetrad.toml");

    if !config_path.exists() {
        println!("Nenhuma configuração encontrada.");
        println!("Execute 'tetrad init' primeiro.");
        return Ok(());
    }

    let config = Config::load(&config_path)?;

    println!("Configuração atual:");
    println!("  Log level: {}", config.general.log_level);
    println!("  Timeout: {}s", config.general.timeout_secs);
    println!("  Consenso: {:?}", config.consensus.default_rule);
    println!("  Score mínimo: {}", config.consensus.min_score);
    println!();

    // TODO: Implementar configuração interativa com dialoguer na próxima fase
    println!("(Configuração interativa será implementada em breve)");
    println!("Por enquanto, edite diretamente o arquivo tetrad.toml");

    Ok(())
}

/// Diagnostica problemas de configuração.
pub async fn doctor() -> TetradResult<()> {
    println!("Diagnosticando configuração do Tetrad...\n");

    let mut issues: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Verifica arquivo de configuração
    let config_path = PathBuf::from("tetrad.toml");
    if !config_path.exists() {
        warnings
            .push("Arquivo tetrad.toml não encontrado (usando configuração padrão)".to_string());
    } else {
        match Config::load(&config_path) {
            Ok(_) => println!("✓ Arquivo de configuração válido"),
            Err(e) => issues.push(format!("Erro no arquivo de configuração: {}", e)),
        }
    }

    // Verifica executores
    let executors: Vec<Box<dyn CliExecutor>> = vec![
        Box::new(CodexExecutor::new()),
        Box::new(GeminiExecutor::new()),
        Box::new(QwenExecutor::new()),
    ];

    let mut available_count = 0;
    for executor in executors {
        if executor.is_available().await {
            available_count += 1;
            println!("✓ {} está disponível", executor.name());
        } else {
            warnings.push(format!("{} não está instalado", executor.name()));
        }
    }

    if available_count == 0 {
        issues.push("Nenhum executor disponível - consenso não é possível".to_string());
    } else if available_count < 3 {
        warnings.push(format!(
            "Apenas {}/3 executores disponíveis - consenso parcial",
            available_count
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
        let result = status().await;
        assert!(result.is_ok());
    }
}
