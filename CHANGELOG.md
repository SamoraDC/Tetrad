# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Em desenvolvimento
- Publicação no crates.io
- GitHub Releases com binários
- Homebrew formula

## [0.1.0] - 2025-01-21

### Adicionado
- **CLI completa** com comandos: init, serve, status, config, doctor, version, evaluate, history, export, import
- **MCP Server** com protocolo JSON-RPC 2.0 sobre stdio
- **6 ferramentas MCP**: tetrad_review_plan, tetrad_review_code, tetrad_review_tests, tetrad_confirm, tetrad_final_check, tetrad_status
- **Motor de consenso** com 3 regras: Golden (unanimidade), Strong (3/3), Weak (maioria)
- **ReasoningBank** com ciclo RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
- **Cache LRU** com TTL configurável
- **Sistema de Hooks**: pre_evaluate, post_evaluate, on_consensus, on_block
- **CLI interativo** com dialoguer para configuração
- **Executores**: Codex, Gemini CLI, Qwen
- **Persistência SQLite** para patterns e trajetórias
- **Export/Import** de patterns do ReasoningBank
- **GitHub Actions CI/CD** para build, test e release
- **205 testes** (127 unitários + 78 integração)

### Características
- Escrito em Rust para alta performance
- Execução paralela via Tokio
- Configuração via TOML (tetrad.toml)
- Integração direta com Claude Code via `claude mcp add`

[Unreleased]: https://github.com/SamoraDC/tetrad/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/SamoraDC/tetrad/releases/tag/v0.1.0
