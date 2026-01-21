# Tetrad

> MCP de Consenso Quádruplo para Claude Code

**Tetrad** é um servidor MCP (Model Context Protocol) de alta performance escrito em Rust que orquestra três ferramentas CLI de código (Codex, Gemini CLI, Qwen) para avaliar e validar todo trabalho produzido pelo Claude Code.

O sistema implementa um protocolo de **consenso quádruplo** onde nenhum código ou plano é aceito sem a aprovação unânime de quatro inteligências: os três avaliadores externos + o próprio Claude Code.

## Características

- **Consenso Quádruplo**: 4 modelos devem concordar para aprovar código
- **ReasoningBank**: Sistema de aprendizado contínuo com ciclo RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
- **Alta Performance**: Escrito em Rust com execução paralela via Tokio
- **CLI Completa**: Comandos intuitivos (`init`, `serve`, `status`, `doctor`, etc.)
- **Extensível**: Sistema de hooks e plugins para customização
- **Cross-session**: Persistência com SQLite

## Instalação

```bash
# Via cargo (recomendado)
cargo install tetrad

# Via Homebrew (macOS/Linux) - em breve
brew install tetrad

# Build local
git clone https://github.com/SamoraDC/tetrad
cd tetrad
cargo build --release
```

## Uso Rápido

```bash
# Inicializa configuração no projeto atual
tetrad init

# Verifica status das CLIs (Codex, Gemini, Qwen)
tetrad status

# Diagnostica problemas de configuração
tetrad doctor

# Inicia o servidor MCP
tetrad serve
```

## Integração com Claude Code

```bash
# Adiciona como MCP server
claude mcp add tetrad -- tetrad serve
```

Ou manualmente em `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "tetrad": {
      "command": "tetrad",
      "args": ["serve"],
      "env": {
        "GEMINI_API_KEY": "${GEMINI_API_KEY}",
        "OPENAI_API_KEY": "${OPENAI_API_KEY}"
      }
    }
  }
}
```

## Comandos CLI

```
tetrad - CLI de Consenso Quádruplo para Claude Code

COMANDOS:
    init        Inicializa configuração no diretório atual
    serve       Inicia o servidor MCP (usado pelo Claude Code)
    status      Mostra status das CLIs (codex, gemini, qwen)
    config      Configura opções interativamente
    doctor      Diagnostica problemas de configuração
    version     Mostra versão

OPÇÕES:
    -c, --config <FILE>    Arquivo de configuração (default: tetrad.toml)
    -v, --verbose          Modo verbose
    -q, --quiet            Modo silencioso
    -h, --help             Mostra ajuda
```

## Arquitetura

```
Claude Code → MCP Protocol (stdio) → Tetrad Server (Rust)
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    ▼                     ▼                     ▼
              Codex CLI            Gemini CLI              Qwen CLI
              (sintaxe)           (arquitetura)            (lógica)
                    │                     │                     │
                    └─────────────────────┼─────────────────────┘
                                          ▼
                                  Consensus Engine
                                          │
                                          ▼
                              ReasoningBank (SQLite)
                           RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
```

### Especialização dos Executores

| Executor | CLI | Especialização |
|----------|-----|----------------|
| **Codex** | `codex -p` | Sintaxe e convenções de código |
| **Gemini** | `gemini --output-format json` | Arquitetura e design |
| **Qwen** | `qwen -p` | Bugs lógicos e correção |

### Regras de Consenso

- **Regra de Ouro**: Unanimidade necessária (3/3 votos)
- **Consenso Forte**: 3/3 votos necessários
- **Consenso Fraco**: 2+ votos necessários

## Configuração

O arquivo `tetrad.toml` é criado automaticamente com `tetrad init`:

```toml
[general]
log_level = "info"
timeout_secs = 60

[executors.codex]
enabled = true
command = "codex"
args = ["-p"]
timeout_secs = 30

[executors.gemini]
enabled = true
command = "gemini"
args = ["--output-format", "json"]
timeout_secs = 30

[executors.qwen]
enabled = true
command = "qwen"
args = ["-p"]
timeout_secs = 30

[consensus]
default_rule = "strong"
min_score = 70
max_loops = 3

[reasoning]
enabled = true
db_path = "tetrad.db"
```

## Status do Desenvolvimento

### ✅ Fase 1 & 2: Fundação + Executores (Completa)

- [x] Setup projeto Rust com estrutura de crate publicável
- [x] CLI com clap (init, serve, status, config, doctor, version)
- [x] Trait `CliExecutor` com implementações para Codex, Gemini, Qwen
- [x] Sistema de configuração TOML
- [x] Health checks (`is_available()`, `version()`)
- [x] Parsing robusto de JSON
- [x] 12 testes unitários passando
- [x] Formatação (rustfmt) e linting (clippy) sem erros

### 🔲 Fase 3: Consenso + ReasoningBank (Próxima)

- [ ] Motor de consenso
- [ ] ReasoningBank com SQLite
- [ ] Ciclo RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
- [ ] Export/Import de patterns

### 🔲 Fase 4: MCP Server

- [ ] Protocolo MCP (stdio)
- [ ] 6 ferramentas expostas
- [ ] Cache LRU
- [ ] Sistema de Hooks

### 🔲 Fase 5: Polish

- [ ] CLI interativo completo
- [ ] Testes de integração
- [ ] GitHub Actions CI/CD

### 🔲 Fase 6: Release

- [ ] Publicar no crates.io
- [ ] GitHub Releases com binários
- [ ] Homebrew formula

## Estrutura do Projeto

```
tetrad/
├── Cargo.toml              # Manifesto do crate
├── CLAUDE.md               # Documentação para Claude Code
├── README.md               # Este arquivo
├── src/
│   ├── main.rs             # Entry point (CLI)
│   ├── lib.rs              # Biblioteca exportável
│   ├── cli/
│   │   ├── mod.rs          # Definição CLI com clap
│   │   └── commands.rs     # Implementação dos comandos
│   ├── executors/
│   │   ├── mod.rs
│   │   ├── base.rs         # Trait CliExecutor
│   │   ├── codex.rs        # Executor Codex
│   │   ├── gemini.rs       # Executor Gemini
│   │   └── qwen.rs         # Executor Qwen
│   ├── types/
│   │   ├── mod.rs
│   │   ├── config.rs       # Configuração TOML
│   │   ├── errors.rs       # TetradError/TetradResult
│   │   ├── requests.rs     # EvaluationRequest
│   │   └── responses.rs    # EvaluationResult, ModelVote
│   ├── consensus/mod.rs    # (Fase 3)
│   ├── reasoning/mod.rs    # (Fase 3)
│   ├── mcp/mod.rs          # (Fase 4)
│   └── hooks/mod.rs        # (Fase 4)
└── Tetrad.md               # Especificação completa
```

## Desenvolvimento

```bash
# Build
cargo build

# Testes
cargo test

# Lint
cargo clippy

# Formatação
cargo fmt

# Rodar CLI
cargo run -- status
cargo run -- doctor
cargo run -- version
```

## Pré-requisitos

Para usar o Tetrad, você precisa ter instalado pelo menos uma das CLIs:

- **Codex CLI**: [github.com/openai/codex-cli](https://github.com/openai/codex-cli)
- **Gemini CLI**: [github.com/google/gemini-cli](https://github.com/google/gemini-cli)
- **Qwen CLI**: [github.com/qwenlm/qwen-cli](https://github.com/qwenlm/qwen-cli)

Verifique a disponibilidade com:

```bash
tetrad status
```

## Licença

MIT

## Autor

SamoraDC
