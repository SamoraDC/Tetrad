# Tetrad

[![CI](https://github.com/SamoraDC/tetrad/actions/workflows/ci.yml/badge.svg)](https://github.com/SamoraDC/tetrad/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

> MCP de Consenso Quádruplo para Claude Code

**Tetrad** é um servidor MCP (Model Context Protocol) de alta performance escrito em Rust que orquestra três ferramentas CLI de código (Codex, Gemini CLI, Qwen) para avaliar e validar todo trabalho produzido pelo Claude Code.

O sistema implementa um protocolo de **consenso quádruplo** onde nenhum código ou plano é aceito sem a aprovação unânime de quatro inteligências: os três avaliadores externos + o próprio Claude Code.

## Características

- **Consenso Quádruplo**: 4 modelos devem concordar para aprovar código
- **ReasoningBank**: Sistema de aprendizado contínuo com ciclo RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
- **Alta Performance**: Escrito em Rust com execução paralela via Tokio
- **MCP Server**: Servidor JSON-RPC 2.0 sobre stdio para integração com Claude Code
- **CLI Completa**: Comandos intuitivos (`init`, `serve`, `status`, `doctor`, `config`, etc.)
- **Cache LRU**: Cache de resultados com TTL configurável
- **Sistema de Hooks**: Callbacks pré/pós avaliação para customização
- **Extensível**: Sistema de plugins para executores customizados
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

# Configuração interativa
tetrad config

# Inicia o servidor MCP
tetrad serve

# Avalia código manualmente (sem MCP)
tetrad evaluate -c "fn main() { println!(\"Hello\"); }" -l rust

# Mostra histórico de avaliações
tetrad history --limit 20

# Exporta/importa patterns do ReasoningBank
tetrad export -o patterns.json
tetrad import patterns.json
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
    init              Inicializa configuração no diretório atual
    serve             Inicia o servidor MCP (usado pelo Claude Code)
    status            Mostra status das CLIs (codex, gemini, qwen)
    config            Configura opções interativamente
    doctor            Diagnostica problemas de configuração
    version           Mostra versão
    evaluate          Avalia código manualmente (sem MCP)
    history           Mostra histórico de avaliações do ReasoningBank
    export            Exporta patterns do ReasoningBank
    import            Importa patterns para o ReasoningBank

OPÇÕES:
    -c, --config <FILE>    Arquivo de configuração (default: tetrad.toml)
    -v, --verbose          Modo verbose
    -q, --quiet            Modo silencioso
    -h, --help             Mostra ajuda
```

## Ferramentas MCP

Quando executando como servidor MCP, o Tetrad expõe 6 ferramentas:

| Ferramenta | Descrição |
|------------|-----------|
| `tetrad_review_plan` | Revisa planos de implementação antes de codificar |
| `tetrad_review_code` | Revisa código antes de salvar |
| `tetrad_review_tests` | Revisa testes antes de finalizar |
| `tetrad_confirm` | Confirma acordo com feedback recebido |
| `tetrad_final_check` | Verificação final antes de commit |
| `tetrad_status` | Verifica saúde dos avaliadores |

### Exemplo de Fluxo

```
1. Claude Code gera plano → tetrad_review_plan → Feedback
2. Claude Code implementa → tetrad_review_code → Feedback
3. Claude Code ajusta → tetrad_confirm → Confirmação
4. Claude Code finaliza → tetrad_final_check → Certificado
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
                            ┌─────────────┴─────────────┐
                            ▼                           ▼
                      Cache LRU                  ReasoningBank
                      (resultados)                 (SQLite)
                                          RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
```

### Especialização dos Executores

| Executor | CLI | Especialização |
|----------|-----|----------------|
| **Codex** | `codex` | Sintaxe e convenções de código |
| **Gemini** | `gemini -o json` | Arquitetura e design |
| **Qwen** | `qwen` | Bugs lógicos e correção |

### Regras de Consenso

| Regra | Requisito | Uso |
|-------|-----------|-----|
| **Golden** | Unanimidade (3/3) | Código crítico, segurança |
| **Strong** | 3/3 ou 2/3 com alta confiança | Padrão |
| **Weak** | Maioria simples (2/3) | Prototipação rápida |

## ReasoningBank

O ReasoningBank é um sistema de aprendizado contínuo que armazena e consolida padrões de código:

### Ciclo de Aprendizado

```
RETRIEVE → JUDGE → DISTILL → CONSOLIDATE
    │         │        │           │
    │         │        │           └─ Merge patterns similares
    │         │        └─ Extrai novos patterns
    │         └─ Avalia código com contexto
    └─ Busca patterns relevantes
```

### Tipos de Patterns

- **AntiPattern**: Padrões a evitar (bugs, vulnerabilidades, code smells)
- **GoodPattern**: Padrões a seguir (boas práticas, padrões idiomáticos)
- **Ambiguous**: Padrões com classificação incerta (requer mais dados)

### Comandos do ReasoningBank

```bash
# Ver histórico de avaliações
tetrad history --limit 50

# Exportar patterns para compartilhar
tetrad export -o team-patterns.json

# Importar patterns de outro ReasoningBank
tetrad import team-patterns.json
```

## Cache LRU

O sistema inclui um cache LRU para evitar reavaliações desnecessárias:

- **Capacidade**: Configurável (padrão: 1000 entradas)
- **TTL**: Tempo de vida configurável (padrão: 5 minutos)
- **Chave**: Hash do código + linguagem + tipo de avaliação
- **Invalidação**: Automática por TTL ou manual

## Sistema de Hooks

Hooks permitem customizar o comportamento em pontos específicos:

| Hook | Quando | Uso |
|------|--------|-----|
| `pre_evaluate` | Antes da avaliação | Modificar request, pular avaliação |
| `post_evaluate` | Após avaliação | Logging, métricas, notificações |
| `on_consensus` | Quando há consenso | Ações automáticas em aprovação |
| `on_block` | Quando código bloqueado | Alertas, rollback automático |

### Hooks Builtin

- **LoggingHook**: Registra todas as avaliações
- **MetricsHook**: Coleta estatísticas de uso

## Configuração

O arquivo `tetrad.toml` é criado automaticamente com `tetrad init`:

```toml
[general]
log_level = "info"
timeout_secs = 60

[executors.codex]
enabled = true
command = "codex"
args = []
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

### Configuração Interativa

Use `tetrad config` para configurar interativamente:

```
🔧 Configuração Interativa do Tetrad

O que deseja configurar?
❯ Configurações Gerais
  Executores (Codex, Gemini, Qwen)
  Consenso
  ReasoningBank
  Salvar e Sair
  Sair sem Salvar
```

## Status do Desenvolvimento

### ✅ Fase 1 & 2: Fundação + Executores (Completa)

- [x] Setup projeto Rust com estrutura de crate publicável
- [x] CLI com clap (init, serve, status, config, doctor, version)
- [x] Trait `CliExecutor` com implementações para Codex, Gemini, Qwen
- [x] Sistema de configuração TOML
- [x] Health checks (`is_available()`, `version()`)
- [x] Parsing robusto de JSON

### ✅ Fase 3: Consenso + ReasoningBank (Completa)

- [x] Motor de consenso com 3 regras (Golden, Strong, Weak)
- [x] ReasoningBank com SQLite
- [x] Ciclo completo RETRIEVE→JUDGE→DISTILL→CONSOLIDATE
- [x] Export/Import de patterns
- [x] Comandos history/export/import

### ✅ Fase 4: MCP Server (Completa)

- [x] Protocolo MCP (stdio) - JSON-RPC 2.0 com Content-Length headers
- [x] 6 ferramentas expostas (review_plan/code/tests, confirm, final_check, status)
- [x] Cache LRU com TTL para resultados de avaliação
- [x] Hooks básicos (pre/post_evaluate, on_consensus, on_block)
- [x] Sistema de confirmações integrado (confirm → final_check)

### ✅ Fase 5: Polish (Completa)

- [x] CLI interativo completo (dialoguer)
- [x] Documentação completa (README.md, CLAUDE.md)
- [x] Testes de integração (205 testes passando)
- [x] GitHub Actions CI/CD (build, test, lint, audit, release)

### 🔄 Fase 6: Release (Em Andamento)

- [ ] Publicar no crates.io
- [ ] GitHub Releases com binários
- [ ] Homebrew formula

## Estrutura do Projeto

```
tetrad/
├── Cargo.toml              # Manifesto do crate
├── CLAUDE.md               # Documentação para Claude Code
├── README.md               # Este arquivo
├── Tetrad.md               # Especificação completa
├── src/
│   ├── main.rs             # Entry point (CLI)
│   ├── lib.rs              # Biblioteca exportável
│   ├── cli/
│   │   ├── mod.rs          # Definição CLI com clap
│   │   ├── commands.rs     # Implementação dos comandos
│   │   └── interactive.rs  # Configuração interativa (dialoguer)
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
│   ├── consensus/
│   │   ├── mod.rs          # Exports
│   │   ├── engine.rs       # ConsensusEngine
│   │   └── rules.rs        # Regras de votação
│   ├── reasoning/
│   │   ├── mod.rs          # Exports
│   │   ├── bank.rs         # ReasoningBank
│   │   ├── patterns.rs     # Pattern types
│   │   └── sqlite.rs       # Storage SQLite
│   ├── mcp/
│   │   ├── mod.rs          # Exports
│   │   ├── server.rs       # Servidor MCP
│   │   ├── protocol.rs     # Tipos JSON-RPC
│   │   ├── tools.rs        # Handlers das ferramentas
│   │   └── transport.rs    # Transporte stdio
│   ├── cache/
│   │   ├── mod.rs          # Exports
│   │   └── lru.rs          # Cache LRU
│   └── hooks/
│       ├── mod.rs          # Trait Hook e HookSystem
│       └── builtin.rs      # Hooks padrão
└── tests/
    └── integration/        # Testes de integração
```

## Desenvolvimento

```bash
# Build
cargo build
cargo build --release

# Testes
cargo test
cargo test --lib                    # Unit tests only
cargo test --tests                  # Integration tests only

# Lint
cargo clippy

# Formatação
cargo fmt
cargo fmt --check

# Documentação
cargo doc --open

# Rodar CLI
cargo run -- status
cargo run -- doctor
cargo run -- version
cargo run -- config
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
