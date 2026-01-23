# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tetrad is a high-performance MCP (Model Context Protocol) server written in Rust that orchestrates three CLI code evaluation tools (Codex, Gemini CLI, Qwen) to validate code produced by Claude Code. It implements a quadruple consensus protocol where no code is accepted without unanimous approval from four intelligences: the three external evaluators + Claude Code itself.

**Current Status**: Phases 1-5 complete. The project has a functional MCP server, consensus engine, ReasoningBank with SQLite, LRU cache, and hook system. Phase 6 (Release) is in progress - published to crates.io and npm.

## Build and Development Commands

```bash
# Build
cargo build
cargo build --release

# Run
cargo run -- <command>

# Test
cargo test                          # All tests
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

## CLI Commands

```bash
tetrad init              # Initialize config in current directory
tetrad serve             # Start MCP server (used by Claude Code)
tetrad status            # Show CLI status (codex, gemini, qwen)
tetrad config            # Interactive configuration (dialoguer)
tetrad doctor            # Diagnose configuration issues
tetrad version           # Show version
tetrad evaluate -c CODE  # Manual code evaluation (without MCP)
tetrad history           # Show evaluation history from ReasoningBank
tetrad export -o FILE    # Export ReasoningBank patterns
tetrad import FILE       # Import patterns into ReasoningBank
```

## Architecture

```
Claude Code → MCP Protocol (stdio) → Tetrad Server (Rust)
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    ▼                     ▼                     ▼
              Codex CLI            Gemini CLI              Qwen CLI
              (syntax)            (architecture)           (logic)
                    │                     │                     │
                    └─────────────────────┼─────────────────────┘
                                          ▼
                                  Consensus Engine
                                          │
                            ┌─────────────┴─────────────┐
                            ▼                           ▼
                      Cache LRU                  ReasoningBank
                      (results)                    (SQLite)
                                    RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
```

### Core Modules (src/)

| Module | Purpose |
|--------|---------|
| `cli/` | Command-line interface (clap + dialoguer for interactive config) |
| `mcp/` | MCP protocol: server, tools, transport (stdio), protocol types |
| `executors/` | CLI wrappers for Codex, Gemini, Qwen with health checks |
| `consensus/` | Voting aggregation with Golden/Strong/Weak rules |
| `reasoning/` | ReasoningBank - SQLite-backed pattern learning system |
| `hooks/` | Pre/post evaluation callbacks (logging, metrics, notifications) |
| `cache/` | LRU cache with TTL for evaluation results |
| `types/` | Shared types: config, errors, requests, responses |

### Key Concepts

- **Quadruple Consensus**: Code must pass validation by 3 external CLIs + Claude Code
- **Consensus Rules**: Golden (unanimous 3/3), Strong (3/3 or 2/3 high confidence), Weak (majority 2/3)
- **ReasoningBank**: SQLite-based learning with RETRIEVE→JUDGE→DISTILL→CONSOLIDATE cycle
- **Pattern Types**: AntiPattern (bugs), GoodPattern (best practices), Ambiguous (uncertain)
- **Cache LRU**: Avoids re-evaluating identical code within TTL window
- **Hooks**: pre_evaluate, post_evaluate, on_consensus, on_block

## Key Dependencies

- **tokio**: Async runtime for parallel CLI execution
- **clap**: CLI argument parsing with derive macros
- **dialoguer**: Interactive terminal prompts
- **rusqlite**: SQLite for ReasoningBank persistence
- **serde/serde_json**: JSON serialization for MCP protocol
- **tracing**: Structured logging
- **sha2/hex**: Code signature hashing for cache keys
- **lru**: LRU cache implementation
- **thiserror**: Error type derivation

## Feature Flags

```toml
[features]
default = ["cli", "sqlite"]
cli = ["clap", "dialoguer", "indicatif"]
sqlite = ["rusqlite"]
```

## Testing

Unit tests are co-located with source files using `#[cfg(test)]` modules.

Run tests:
```bash
cargo test              # All tests (219 tests passing: 141 unit + 78 integration)
cargo test consensus    # Tests containing "consensus"
cargo test reasoning    # Tests containing "reasoning"
cargo test mcp          # Tests containing "mcp"
```

## MCP Tools Exposed

When running as MCP server (`tetrad serve`), Tetrad exposes 6 tools:

| Tool | Input | Output |
|------|-------|--------|
| `tetrad_review_plan` | `{ plan, context? }` | `{ decision, score, feedback, findings[] }` |
| `tetrad_review_code` | `{ code, language, file_path?, context? }` | `{ decision, score, feedback, findings[] }` |
| `tetrad_review_tests` | `{ tests, language, context? }` | `{ decision, score, feedback, findings[] }` |
| `tetrad_confirm` | `{ request_id, agreed, notes? }` | `{ confirmed, can_proceed }` |
| `tetrad_final_check` | `{ code, language, previous_request_id? }` | `{ certified, decision, score, certificate_id? }` |
| `tetrad_status` | `{}` | `{ codex: {...}, gemini: {...}, qwen: {...} }` |

### MCP Workflow

```
1. tetrad_review_plan → Feedback on implementation plan
2. tetrad_review_code → Feedback on code
3. tetrad_confirm → Acknowledge feedback was addressed
4. tetrad_final_check → Final certification (requires confirmation if previous_request_id provided)
```

## Configuration

Default config file: `tetrad.toml`

```toml
[general]
log_level = "info"
timeout_secs = 60

[executors.codex]
enabled = true
command = "codex"
args = ["exec", "--json"]
timeout_secs = 30

[executors.gemini]
enabled = true
command = "gemini"
args = ["-o", "json"]
timeout_secs = 30

[executors.qwen]
enabled = true
command = "qwen"
args = []
timeout_secs = 30

[consensus]
default_rule = "strong"  # golden, strong, weak
min_score = 70
max_loops = 3

[reasoning]
enabled = true
db_path = ".tetrad/tetrad.db"
max_patterns_per_query = 10
consolidation_interval = 100

[cache]
enabled = true
capacity = 1000
ttl_secs = 300
```

### Configuration Commands

```bash
tetrad --config custom.toml serve   # Use custom config
tetrad -v serve                      # Verbose mode
tetrad -q serve                      # Quiet mode
tetrad config                        # Interactive configuration
```

## Project Structure

```
tetrad/
├── Cargo.toml
├── CLAUDE.md               # This file
├── README.md               # User documentation
├── Tetrad.md               # Complete specification
├── src/
│   ├── main.rs             # Entry point
│   ├── lib.rs              # Library exports
│   ├── cli/
│   │   ├── mod.rs          # CLI definition (clap)
│   │   ├── commands.rs     # Command implementations
│   │   └── interactive.rs  # Interactive config (dialoguer)
│   ├── executors/
│   │   ├── mod.rs
│   │   ├── base.rs         # CliExecutor trait
│   │   ├── codex.rs
│   │   ├── gemini.rs
│   │   └── qwen.rs
│   ├── consensus/
│   │   ├── mod.rs
│   │   ├── engine.rs       # ConsensusEngine
│   │   └── rules.rs        # Voting rules
│   ├── reasoning/
│   │   ├── mod.rs
│   │   ├── bank.rs         # ReasoningBank
│   │   ├── patterns.rs     # Pattern types
│   │   └── sqlite.rs       # SQLite storage
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── server.rs       # McpServer
│   │   ├── protocol.rs     # JSON-RPC types
│   │   ├── tools.rs        # Tool handlers
│   │   └── transport.rs    # Stdio transport
│   ├── cache/
│   │   ├── mod.rs
│   │   └── lru.rs          # EvaluationCache
│   ├── hooks/
│   │   ├── mod.rs          # Hook trait, HookSystem
│   │   └── builtin.rs      # LoggingHook, MetricsHook
│   └── types/
│       ├── mod.rs
│       ├── config.rs
│       ├── errors.rs
│       ├── requests.rs
│       └── responses.rs
```

## Development Status

- **Phase 1-2**: Foundation + Executors ✅
- **Phase 3**: Consensus + ReasoningBank ✅
- **Phase 4**: MCP Server + Cache + Hooks ✅
- **Phase 5**: Polish - Interactive CLI, docs, tests, CI/CD ✅
- **Phase 6**: Release - crates.io, npm, Homebrew, GitHub Releases 🔄
  - [x] Published to crates.io (v0.1.4)
  - [x] Published to npm (v0.1.4)
  - [ ] GitHub Releases with binaries
  - [ ] Homebrew formula
