# Tetrad MCP

[![npm](https://img.shields.io/npm/v/tetrad-mcp.svg)](https://www.npmjs.com/package/tetrad-mcp)
[![CI](https://github.com/SamoraDC/Tetrad/actions/workflows/ci.yml/badge.svg)](https://github.com/SamoraDC/Tetrad/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

> Quadruple Consensus MCP Server for Claude Code

**Tetrad** is a high-performance MCP (Model Context Protocol) server that orchestrates three AI-powered CLI tools (Codex, Gemini CLI, Qwen) to validate all code produced by Claude Code.

## Quick Start

### 1. Install

```bash
npm install -g tetrad-mcp
```

### 2. Add to Claude Code

```bash
# Add as MCP server (available in all projects)
claude mcp add --scope user tetrad -- npx tetrad-mcp serve

# Or for current project only
claude mcp add tetrad -- npx tetrad-mcp serve
```

### 3. Verify

```bash
# Check version
npx tetrad-mcp version

# Check CLI availability
npx tetrad-mcp status

# Diagnose issues
npx tetrad-mcp doctor
```

## Manual MCP Configuration

Add to your `.mcp.json`:

```json
{
  "mcpServers": {
    "tetrad": {
      "type": "stdio",
      "command": "npx",
      "args": ["tetrad-mcp", "serve"]
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
- **ReasoningBank**: Continuous learning system
- **High Performance**: Written in Rust
- **LRU Cache**: Result caching with configurable TTL
- **Hook System**: Pre/post evaluation callbacks

## Documentation

Full documentation: [GitHub Repository](https://github.com/SamoraDC/Tetrad)

## License

MIT
