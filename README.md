# Tetrad

[![CI](https://github.com/SamoraDC/Tetrad/actions/workflows/ci.yml/badge.svg)](https://github.com/SamoraDC/Tetrad/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/tetrad.svg)](https://crates.io/crates/tetrad)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

> Quadruple Consensus MCP Server for Claude Code

**Tetrad** is a high-performance MCP (Model Context Protocol) server written in Rust that orchestrates three AI-powered CLI code evaluation tools (Codex, Gemini CLI, Qwen) to validate all code produced by Claude Code.

The system implements a **quadruple consensus protocol** where no code or plan is accepted without approval from four intelligences: the three external evaluators + Claude Code itself.

## Features

- **Quadruple Consensus**: 4 AI models must agree to approve code
- **ReasoningBank**: Continuous learning system with RETRIEVE→JUDGE→DISTILL→CONSOLIDATE cycle
- **High Performance**: Written in Rust with parallel execution via Tokio
- **MCP Server**: JSON-RPC 2.0 server over stdio for Claude Code integration
- **Full CLI**: Intuitive commands (`init`, `serve`, `status`, `doctor`, `config`, etc.)
- **LRU Cache**: Result caching with configurable TTL
- **Hook System**: Pre/post evaluation callbacks for customization
- **Extensible**: Plugin system for custom executors
- **Cross-session**: SQLite persistence for patterns and history

## Quick Start

### 1. Install Tetrad

```bash
# Via cargo (recommended)
cargo install tetrad

# Or build from source
git clone https://github.com/SamoraDC/Tetrad.git
cd Tetrad
cargo build --release
sudo cp target/release/tetrad /usr/local/bin/
```

### 2. Install External CLI Tools

Tetrad requires at least one of the following AI CLI tools:

```bash
# Codex CLI (OpenAI)
npm install -g @openai/codex
export OPENAI_API_KEY="your-openai-key"

# Gemini CLI (Google)
npm install -g @google/gemini-cli
export GOOGLE_API_KEY="your-google-key"

# Qwen CLI (Alibaba)
pip install dashscope
export DASHSCOPE_API_KEY="your-dashscope-key"
```

### 3. Verify Installation

```bash
# Check Tetrad version
tetrad version

# Check CLI availability
tetrad status

# Diagnose any issues
tetrad doctor
```

### 4. Add to Claude Code CLI

```bash
# Add Tetrad as MCP server (available in all projects)
claude mcp add --scope user tetrad -- tetrad serve

# Or for current project only
claude mcp add tetrad -- tetrad serve

# Verify it's configured
claude mcp list
```

### 5. Alternative: Manual Configuration

Create or edit `.mcp.json` in your project root:

```json
{
  "mcpServers": {
    "tetrad": {
      "type": "stdio",
      "command": "tetrad",
      "args": ["serve"],
      "env": {
        "OPENAI_API_KEY": "${OPENAI_API_KEY}",
        "GOOGLE_API_KEY": "${GOOGLE_API_KEY}",
        "DASHSCOPE_API_KEY": "${DASHSCOPE_API_KEY}"
      }
    }
  }
}
```

Or for global user configuration in `~/.config/claude-code/settings.json`:

```json
{
  "mcpServers": {
    "tetrad": {
      "type": "stdio",
      "command": "tetrad",
      "args": ["serve"]
    }
  }
}
```

## How It Works

When you ask Claude Code to write code, Tetrad automatically validates it:

```
You: "Create a function in Rust that calculates the average of a vector"

Claude Code:
1. Writes the code
2. Calls tetrad_review_code automatically
3. Tetrad sends to Codex, Gemini and Qwen for evaluation
4. Returns consolidated consensus from all 3 evaluators

Tetrad Response:
┌─────────────────────────────────────────────┐
│ DECISION: PASS ✓                            │
│ Score: 92/100                               │
│ Consensus: Yes (3/3 approved)               │
│                                             │
│ Votes:                                      │
│   • Codex:  Pass (95)                       │
│   • Gemini: Pass (90)                       │
│   • Qwen:   Pass (92)                       │
│                                             │
│ Suggestions:                                │
│   - Consider handling empty vector case     │
└─────────────────────────────────────────────┘

Claude Code: Saves the approved code
```

### When Issues Are Found:

```
Tetrad Response:
┌─────────────────────────────────────────────┐
│ DECISION: BLOCK ✗                           │
│ Score: 26/100                               │
│ Consensus: Yes (3/3 rejected)               │
│                                             │
│ Votes:                                      │
│   • Codex:  Fail (30) - division by zero    │
│   • Gemini: Fail (25) - no error handling   │
│   • Qwen:   Fail (25) - unsafe operation    │
│                                             │
│ Issues:                                     │
│   - Division by zero not handled            │
│   - Missing input validation                │
│   - No Result/Option return type            │
└─────────────────────────────────────────────┘

Claude Code: Fixes the code and resubmits
```

## CLI Commands

```
tetrad - Quadruple Consensus CLI for Claude Code

COMMANDS:
    init              Initialize configuration in current directory
    serve             Start the MCP server (used by Claude Code)
    status            Show CLI status (codex, gemini, qwen)
    config            Configure options interactively
    doctor            Diagnose configuration issues
    version           Show version
    evaluate          Evaluate code manually (without MCP)
    history           Show evaluation history from ReasoningBank
    export            Export patterns from ReasoningBank
    import            Import patterns into ReasoningBank

OPTIONS:
    -c, --config <FILE>    Configuration file (default: tetrad.toml)
    -v, --verbose          Verbose mode
    -q, --quiet            Quiet mode
    -h, --help             Show help
```

## MCP Tools

When running as MCP server, Tetrad exposes 6 tools:

| Tool | Description |
|------|-------------|
| `tetrad_review_plan` | Review implementation plans before coding |
| `tetrad_review_code` | Review code before saving |
| `tetrad_review_tests` | Review tests before finalizing |
| `tetrad_confirm` | Confirm agreement with received feedback |
| `tetrad_final_check` | Final verification before commit |
| `tetrad_status` | Check health of evaluators |

### Workflow Example

```
1. Claude Code generates plan → tetrad_review_plan → Feedback
2. Claude Code implements   → tetrad_review_code → Feedback
3. Claude Code adjusts      → tetrad_confirm     → Confirmation
4. Claude Code finalizes    → tetrad_final_check → Certificate
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
                      LRU Cache                  ReasoningBank
                      (results)                    (SQLite)
                                    RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
```

### Executor Specializations

| Executor | CLI | Specialization |
|----------|-----|----------------|
| **Codex** | `codex exec --json` | Syntax and code conventions |
| **Gemini** | `gemini -o json` | Architecture and design |
| **Qwen** | `qwen` | Logic bugs and correctness |

### Consensus Rules

| Rule | Requirement | Use Case |
|------|-------------|----------|
| **Golden** | Unanimity (3/3) | Critical code, security |
| **Strong** | 3/3 or 2/3 with high confidence | Default |
| **Weak** | Simple majority (2/3) | Rapid prototyping |

## ReasoningBank

The ReasoningBank is a continuous learning system that stores and consolidates code patterns:

### Learning Cycle

```
RETRIEVE → JUDGE → DISTILL → CONSOLIDATE
    │         │        │           │
    │         │        │           └─ Merge similar patterns
    │         │        └─ Extract new patterns
    │         └─ Evaluate code with context
    └─ Search for relevant patterns
```

### Pattern Types

- **AntiPattern**: Patterns to avoid (bugs, vulnerabilities, code smells)
- **GoodPattern**: Patterns to follow (best practices, idiomatic patterns)
- **Ambiguous**: Patterns with uncertain classification (needs more data)

### ReasoningBank Commands

```bash
# View evaluation history
tetrad history --limit 50

# Export patterns to share
tetrad export -o team-patterns.json

# Import patterns from another ReasoningBank
tetrad import team-patterns.json
```

## Configuration

The `tetrad.toml` file is created automatically with `tetrad init`:

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
default_rule = "strong"
min_score = 70
max_loops = 3

[reasoning]
enabled = true
db_path = "tetrad.db"
max_patterns_per_query = 10
consolidation_interval = 100

[cache]
enabled = true
capacity = 1000
ttl_secs = 300
```

### Interactive Configuration

Use `tetrad config` for interactive configuration:

```
🔧 Tetrad Interactive Configuration

What would you like to configure?
❯ General Settings
  Executors (Codex, Gemini, Qwen)
  Consensus
  ReasoningBank
  Save and Exit
  Exit without Saving
```

## LRU Cache

The system includes an LRU cache to avoid unnecessary re-evaluations:

- **Capacity**: Configurable (default: 1000 entries)
- **TTL**: Configurable time-to-live (default: 5 minutes)
- **Key**: Hash of code + language + evaluation type
- **Invalidation**: Automatic by TTL or manual

## Hook System

Hooks allow customizing behavior at specific points:

| Hook | When | Use |
|------|------|-----|
| `pre_evaluate` | Before evaluation | Modify request, skip evaluation |
| `post_evaluate` | After evaluation | Logging, metrics, notifications |
| `on_consensus` | When consensus reached | Automatic actions on approval |
| `on_block` | When code blocked | Alerts, automatic rollback |

### Built-in Hooks

- **LoggingHook**: Records all evaluations
- **MetricsHook**: Collects usage statistics

## Project Structure

```
tetrad/
├── Cargo.toml              # Crate manifest
├── CLAUDE.md               # Documentation for Claude Code
├── README.md               # This file
├── Tetrad.md               # Complete specification
├── src/
│   ├── main.rs             # Entry point (CLI)
│   ├── lib.rs              # Exportable library
│   ├── cli/
│   │   ├── mod.rs          # CLI definition with clap
│   │   ├── commands.rs     # Command implementations
│   │   └── interactive.rs  # Interactive configuration (dialoguer)
│   ├── executors/
│   │   ├── mod.rs
│   │   ├── base.rs         # CliExecutor trait
│   │   ├── codex.rs        # Codex executor
│   │   ├── gemini.rs       # Gemini executor
│   │   └── qwen.rs         # Qwen executor
│   ├── types/
│   │   ├── mod.rs
│   │   ├── config.rs       # TOML configuration
│   │   ├── errors.rs       # TetradError/TetradResult
│   │   ├── requests.rs     # EvaluationRequest
│   │   └── responses.rs    # EvaluationResult, ModelVote
│   ├── consensus/
│   │   ├── mod.rs          # Exports
│   │   ├── engine.rs       # ConsensusEngine
│   │   ├── aggregator.rs   # Vote aggregation
│   │   └── rules.rs        # Voting rules
│   ├── reasoning/
│   │   ├── mod.rs          # Exports
│   │   ├── bank.rs         # ReasoningBank
│   │   ├── patterns.rs     # Pattern types
│   │   ├── sqlite.rs       # SQLite storage
│   │   └── export.rs       # Import/Export
│   ├── mcp/
│   │   ├── mod.rs          # Exports
│   │   ├── server.rs       # MCP server
│   │   ├── protocol.rs     # JSON-RPC types
│   │   ├── tools.rs        # Tool handlers
│   │   └── transport.rs    # Stdio transport
│   ├── cache/
│   │   ├── mod.rs          # Exports
│   │   └── lru.rs          # LRU cache
│   └── hooks/
│       ├── mod.rs          # Hook trait and HookSystem
│       └── builtin.rs      # Default hooks
└── tests/
    ├── cli_integration.rs
    ├── consensus_integration.rs
    ├── mcp_integration.rs
    └── reasoning_integration.rs
```

## Development

```bash
# Build
cargo build
cargo build --release

# Tests
cargo test                          # All tests
cargo test --lib                    # Unit tests only
cargo test --tests                  # Integration tests only

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt
cargo fmt --check

# Documentation
cargo doc --open

# Run CLI
cargo run -- status
cargo run -- doctor
cargo run -- version
cargo run -- config
```

## Troubleshooting

### "CLI not found"

```bash
# Check if CLIs are in PATH
which codex
which gemini
which qwen

# Check configuration
tetrad doctor
```

### "stdin is not a terminal" (Codex)

Make sure your config uses `exec --json`:

```toml
[executors.codex]
args = ["exec", "--json"]
```

### "Response does not contain valid JSON" (Gemini)

Make sure your config uses `-o json`:

```toml
[executors.gemini]
args = ["-o", "json"]
```

### Check MCP status in Claude Code

Inside Claude Code, run:

```
/mcp
```

## Prerequisites

To use Tetrad, you need at least one of the AI CLIs installed:

- **Codex CLI**: [OpenAI Codex](https://github.com/openai/codex)
- **Gemini CLI**: [Google Gemini](https://github.com/google-gemini/gemini-cli)
- **Qwen CLI**: [Alibaba Qwen](https://github.com/QwenLM/Qwen)

Check availability with:

```bash
tetrad status
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT

## Author

SamoraDC

---

**Links:**
- [Crates.io](https://crates.io/crates/tetrad)
- [Documentation](https://docs.rs/tetrad)
- [GitHub Repository](https://github.com/SamoraDC/Tetrad)
- [Issue Tracker](https://github.com/SamoraDC/Tetrad/issues)
