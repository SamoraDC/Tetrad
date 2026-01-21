# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### In Development
- Homebrew formula
- Additional language support

## [0.1.1] - 2026-01-21

### Fixed
- Fixed clippy warnings for redundant closures in `transport.rs`
- Fixed cache expiration test for cross-platform compatibility (macOS)
- Fixed formatting issues

### Changed
- Updated Codex executor to use `exec --json` for non-interactive mode
- Updated Gemini executor to parse wrapper JSON response
- Improved error handling in executors

## [0.1.0] - 2026-01-21

### Added
- **Full CLI** with commands: init, serve, status, config, doctor, version, evaluate, history, export, import
- **MCP Server** with JSON-RPC 2.0 protocol over stdio
- **6 MCP tools**: tetrad_review_plan, tetrad_review_code, tetrad_review_tests, tetrad_confirm, tetrad_final_check, tetrad_status
- **Consensus engine** with 3 rules: Golden (unanimity), Strong (3/3), Weak (majority)
- **ReasoningBank** with RETRIEVE→JUDGE→DISTILL→CONSOLIDATE cycle
- **LRU Cache** with configurable TTL
- **Hook System**: pre_evaluate, post_evaluate, on_consensus, on_block
- **Interactive CLI** with dialoguer for configuration
- **Executors**: Codex, Gemini CLI, Qwen
- **SQLite persistence** for patterns and trajectories
- **Export/Import** of ReasoningBank patterns
- **GitHub Actions CI/CD** for build, test and release
- **219 tests** (141 unit + 78 integration)

### Features
- Written in Rust for high performance
- Parallel execution via Tokio
- Configuration via TOML (tetrad.toml)
- Direct integration with Claude Code via `claude mcp add`

[Unreleased]: https://github.com/SamoraDC/Tetrad/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/SamoraDC/Tetrad/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/SamoraDC/Tetrad/releases/tag/v0.1.0
