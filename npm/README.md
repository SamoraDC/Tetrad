# Tetrad

[![npm](https://img.shields.io/npm/v/@samoradc/tetrad.svg)](https://www.npmjs.com/package/@samoradc/tetrad)
[![CI](https://github.com/SamoraDC/Tetrad/actions/workflows/ci.yml/badge.svg)](https://github.com/SamoraDC/Tetrad/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

> Quadruple Consensus MCP Server for Claude Code

**Tetrad** is a high-performance MCP (Model Context Protocol) server that orchestrates three AI-powered CLI tools (Codex, Gemini CLI, Qwen) to validate all code produced by Claude Code.

## Quick Start

### 1. Initialize in your project

```bash
npx @samoradc/tetrad init
```

This will:
- Create `tetrad.toml` configuration file
- Create `.tetrad/` directory for the database
- Add `.tetrad/` to your `.gitignore`

### 2. Add to Claude Code

```bash
# Add as MCP server (available in all projects)
claude mcp add --scope user tetrad -- npx @samoradc/tetrad serve

# Or for current project only
claude mcp add tetrad -- npx @samoradc/tetrad serve
```

### 3. Verify

```bash
# Check version
npx @samoradc/tetrad version

# Check CLI availability
npx @samoradc/tetrad status

# Diagnose issues
npx @samoradc/tetrad doctor
```

## Commands

```bash
npx @samoradc/tetrad init              # Initialize config in current directory
npx @samoradc/tetrad serve             # Start MCP server (used by Claude Code)
npx @samoradc/tetrad status            # Show CLI status (codex, gemini, qwen)
npx @samoradc/tetrad config            # Interactive configuration
npx @samoradc/tetrad doctor            # Diagnose configuration issues
npx @samoradc/tetrad version           # Show version
npx @samoradc/tetrad evaluate -c CODE  # Manual code evaluation (without MCP)
npx @samoradc/tetrad history           # Show evaluation history from ReasoningBank
```

## Manual MCP Configuration

Add to your `.mcp.json`:

```json
{
  "mcpServers": {
    "tetrad": {
      "type": "stdio",
      "command": "npx",
      "args": ["@samoradc/tetrad", "serve"]
    }
  }
}
```

## Requirements

Tetrad requires at least one of these AI CLI tools:

- **Codex CLI** (OpenAI): `npm install -g @openai/codex`
- **Gemini CLI** (Google): `npm install -g @google/gemini-cli`
- **Qwen CLI** (Alibaba): `pip install dashscope`

## Features

- **Quadruple Consensus**: 4 AI models must agree to approve code
- **ReasoningBank**: Continuous learning system with SQLite
- **High Performance**: Written in Rust
- **LRU Cache**: Result caching with configurable TTL
- **Hook System**: Pre/post evaluation callbacks
- **Auto .gitignore**: Automatically ignores local data

## Configuration

After `npx @samoradc/tetrad init`, edit `tetrad.toml`:

```toml
[general]
log_level = "info"
timeout_secs = 60

[executors.codex]
enabled = true
command = "codex"
args = ["exec", "--json"]

[executors.gemini]
enabled = true
command = "gemini"
args = ["-o", "json"]

[executors.qwen]
enabled = true
command = "qwen"

[consensus]
default_rule = "strong"  # golden, strong, weak
min_score = 70
max_loops = 3

[reasoning]
enabled = true
db_path = ".tetrad/tetrad.db"

[cache]
enabled = true
capacity = 1000
ttl_secs = 300
```

## Documentation

Full documentation: [GitHub Repository](https://github.com/SamoraDC/Tetrad)

## License

MIT
