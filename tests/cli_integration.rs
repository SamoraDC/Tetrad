//! Testes de integração para a CLI do Tetrad.

use std::process::Command;

/// Verifica que o binário pode ser executado.
fn tetrad_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tetrad"))
}

#[test]
fn test_version_command() {
    let output = tetrad_bin()
        .arg("version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tetrad") || stdout.contains("Tetrad"));
}

#[test]
fn test_help_command() {
    let output = tetrad_bin()
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("init"));
    assert!(stdout.contains("serve"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("config"));
    assert!(stdout.contains("doctor"));
}

#[test]
fn test_status_command_runs() {
    let output = tetrad_bin()
        .arg("status")
        .output()
        .expect("Failed to execute command");

    // status pode falhar se as CLIs não estiverem instaladas, mas deve rodar
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Deve conter referências aos executores
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("Codex")
            || combined.contains("codex")
            || combined.contains("Status")
            || combined.contains("status")
    );
}

#[test]
fn test_doctor_command_runs() {
    let output = tetrad_bin()
        .arg("doctor")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Deve mostrar diagnóstico
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("Diagn") || combined.contains("config") || combined.contains("Doctor")
    );
}

#[test]
fn test_init_creates_config() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("tetrad.toml");

    let output = tetrad_bin()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "init command failed");
    assert!(config_path.exists(), "Config file was not created");

    // Verifica conteúdo básico
    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    assert!(content.contains("[general]"));
    assert!(content.contains("[executors"));
    assert!(content.contains("[consensus]"));
}

#[test]
fn test_invalid_command() {
    let output = tetrad_bin()
        .arg("invalid-command-that-does-not-exist")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}

#[test]
fn test_verbose_flag() {
    let output = tetrad_bin()
        .arg("-v")
        .arg("version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_quiet_flag() {
    let output = tetrad_bin()
        .arg("-q")
        .arg("version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_custom_config_path() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("custom.toml");

    // Usando config inexistente deve falhar ou mostrar erro
    let output = tetrad_bin()
        .arg("--config")
        .arg(&config_path)
        .arg("status")
        .output()
        .expect("Failed to execute command");

    // Não precisa ter sucesso, só precisa rodar sem crash
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);
}
