# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tetrad is a high-performance MCP (Model Context Protocol) server written in Rust that orchestrates three CLI code evaluation tools (Codex, Gemini CLI, Qwen) to validate code produced by Claude Code. It implements a quadruple consensus protocol where no code is accepted without unanimous approval from four intelligences: the three external evaluators + Claude Code itself.

**Current Status**: Design/specification phase. The `Tetrad.md` file contains the complete architectural specification awaiting implementation.

## Build and Development Commands

```bash
# Build
cargo build
cargo build --release

# Run
cargo run -- <command>

# Test
cargo test
cargo test --lib                    # Unit tests only
cargo test --test <test_name>       # Single integration test

# Lint and format
cargo clippy
cargo fmt
cargo fmt --check

# Check without building
cargo check

# Documentation
cargo doc --open

# Install locally for testing
cargo install --path .
```

## CLI Commands (When Implemented)

```bash
tetrad init        # Initialize config in current directory
tetrad serve       # Start MCP server (used by Claude Code)
tetrad status      # Show CLI status (codex, gemini, qwen)
tetrad config      # Interactive configuration
tetrad evaluate    # Manual code evaluation (without MCP)
tetrad history     # Show evaluation history
tetrad export      # Export ReasoningBank patterns
tetrad import      # Import patterns from another ReasoningBank
tetrad doctor      # Diagnose configuration issues
```

## Architecture

```
Claude Code → MCP Protocol (stdio) → Tetrad Server (Rust)
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    ▼                     ▼                     ▼
              Codex CLI            Gemini CLI              Qwen CLI
                    │                     │                     │
                    └─────────────────────┼─────────────────────┘
                                          ▼
                                  Consensus Engine
                                          │
                                          ▼
                              ReasoningBank (SQLite)
                           RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
```

### Core Modules (src/)

| Module | Purpose |
|--------|---------|
| `cli/` | Command-line interface (clap-based) |
| `mcp/` | MCP protocol implementation (server, tools, transport) |
| `executors/` | CLI wrappers for Codex, Gemini, Qwen |
| `consensus/` | Voting aggregation and decision rules |
| `reasoning/` | ReasoningBank - SQLite-backed pattern learning |
| `hooks/` | Pre/post evaluation callbacks |
| `plugins/` | Extensibility framework for custom executors |
| `prompts/` | Prompt templates and builders |
| `cache/` | LRU caching |
| `types/` | Shared types, config, errors |

### Key Concepts

- **Quadruple Consensus**: Code must pass validation by 3 external CLIs + Claude Code
- **ReasoningBank**: SQLite-based learning system with RETRIEVE→JUDGE→DISTILL→CONSOLIDATE cycle
- **Consensus Rules**: Regra de Ouro (unanimous), Consenso Fraco (2+ votes), Consenso Forte (3/3)
- **Pattern Learning**: Tracks anti-patterns and good patterns with confidence scores

## Key Dependencies

- **tokio**: Async runtime for parallel CLI execution
- **clap**: CLI argument parsing with derive macros
- **rusqlite**: SQLite for ReasoningBank persistence
- **serde/serde_json**: Serialization
- **tracing**: Structured logging
- **sha2/hex**: Code signature hashing

## Feature Flags

```toml
default = ["cli", "sqlite"]
cli = ["clap", "dialoguer", "indicatif"]
sqlite = ["rusqlite"]
postgres = ["sqlx"]    # Enterprise option
plugins = ["libloading"]
```

## Testing

Integration tests go in `tests/integration/`:
- `test_cli.rs` - CLI command testing
- `test_mcp.rs` - MCP protocol testing
- `test_reasoning.rs` - ReasoningBank cycle testing
- `test_consensus.rs` - Consensus engine testing

Test fixtures in `tests/fixtures/`:
- `good_code/` - Code that should pass validation
- `bad_code/` - Code that should fail validation
- `patterns/` - Pattern matching test data

## MCP Tools Exposed

When running as MCP server, Tetrad exposes 6 tools:
1. `tetrad_review_plan` - Review implementation plans before coding
2. `tetrad_review_code` - Review code before saving
3. `tetrad_review_tests` - Review tests before finalizing
4. `tetrad_confirm` - Confirm agreement with feedback
5. `tetrad_final_check` - Final validation after all corrections
6. `tetrad_status` - Check evaluator health

## Configuration

Default config file: `tetrad.toml`

```bash
tetrad --config custom.toml serve   # Use custom config
tetrad -v serve                      # Verbose mode
tetrad -q serve                      # Quiet mode
```

## Distribution

- Primary: `cargo install tetrad` (crates.io)
- Secondary: Homebrew formula
- Tertiary: GitHub Releases (multi-platform binaries)
