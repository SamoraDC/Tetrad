# Relatório de Debugging - Tetrad v0.1.0

**Data:** 2026-01-21
**Status do Projeto:** Phase 5 (Polish) em progresso

---

## Resumo Executivo

O projeto Tetrad está funcionalmente estável com **205 testes passando** e compilação bem-sucedida. No entanto, foram identificados problemas de integração com os executores CLI externos (Codex, Gemini, Qwen) e alguns warnings de código que devem ser corrigidos antes do release.

---

## 1. Resultados da Análise

### 1.1 Compilação
| Métrica | Status |
|---------|--------|
| `cargo build` | ✅ Sucesso |
| `cargo build --release` | ✅ Sucesso |
| Warnings de compilação | ⚠️ 1 warning (clippy) |

### 1.2 Testes
| Suite | Resultado |
|-------|-----------|
| Unit tests (lib) | ✅ 127 passando |
| Unit tests (bin) | ✅ 0 (esperado) |
| Integration tests | ✅ 78 passando |
| Doc tests | ⚠️ 2 ignorados |
| **Total** | **205 testes OK** |

### 1.3 Análise Estática
| Ferramenta | Status |
|------------|--------|
| `cargo clippy` | ⚠️ 1 warning |
| `cargo fmt --check` | ❌ Falhas de formatação |

---

## 2. Problemas Identificados

### 2.1 CRÍTICO - Executores CLI Falham em Produção

**Arquivo:** `src/executors/codex.rs`, `src/executors/gemini.rs`
**Severidade:** 🔴 Alta

#### Problema 1: Codex - "stdin is not a terminal"

Ao executar `tetrad evaluate`, o executor Codex falha com:
```
Executor 'Codex' falhou: Error: stdin is not a terminal
```

**Causa:** O Codex CLI requer um terminal TTY para funcionar corretamente. Quando executado via `tokio::process::Command`, não há TTY disponível.

**Impacto:** Codex não pode ser usado para avaliações automatizadas.

**Solução Proposta:**
1. Usar pseudo-terminal (PTY) para executar Codex
2. Ou usar flag `--non-interactive` se disponível
3. Ou passar input via stdin com pipes configurados corretamente

#### Problema 2: Gemini - "Resposta não contém JSON válido"

```
Executor 'Gemini' falhou: Resposta não contém JSON válido
```

**Causa:** O Gemini CLI não está retornando output no formato JSON esperado pelo parser em `src/executors/base.rs:116-134`.

**Impacto:** Gemini não pode ser usado para avaliações.

**Solução Proposta:**
1. Verificar argumentos corretos para forçar output JSON (`-o json` ou `--output-format json`)
2. Revisar parser para lidar com diferentes formatos de resposta
3. Adicionar fallback para parsing de texto livre

---

### 2.2 MÉDIO - Warning do Clippy: Arc com tipo não Send/Sync

**Arquivo:** `src/mcp/tools.rs:152`
**Severidade:** 🟡 Média

```rust
warning: usage of an `Arc` that is not `Send` and `Sync`
  --> src/mcp/tools.rs:152:29
   |
152 |             reasoning_bank: Arc::new(RwLock::new(reasoning_bank)),
   |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

**Causa:** `ReasoningBank` contém `rusqlite::Connection` que não implementa `Send` nem `Sync`. Usar `std::sync::RwLock` com tipo não-Sync causa este warning.

**Impacto:** Potenciais problemas em código multi-thread. O código funciona atualmente porque o acesso é serializado, mas não é thread-safe de forma garantida.

**Solução Proposta:**
```rust
// Opção 1: Usar tokio::sync::RwLock + Mutex interno
reasoning_bank: Arc<tokio::sync::Mutex<Option<ReasoningBank>>>

// Opção 2: Mover para thread dedicada com channel
// Opção 3: Usar r2d2 connection pool com rusqlite
```

---

### 2.3 BAIXO - Formatação de Código Inconsistente

**Severidade:** 🟢 Baixa

Múltiplos arquivos não estão formatados de acordo com `rustfmt`:

| Arquivo | Problema |
|---------|----------|
| `src/cache/lru.rs` | Linhas muito longas |
| `src/cli/commands.rs` | Box::new em múltiplas linhas |
| `src/cli/interactive.rs` | Cadeias de métodos não formatadas |
| `src/mcp/tools.rs` | Formatação inconsistente |
| `tests/*.rs` | Imports não ordenados |

**Solução:** Executar `cargo fmt` antes do release.

---

### 2.4 BAIXO - Doc Tests Ignorados

**Arquivos:**
- `src/consensus/mod.rs` (linha 15)
- `src/mcp/mod.rs` (linha 17)

**Severidade:** 🟢 Baixa

Os exemplos de documentação estão marcados como ignorados, provavelmente porque dependem de setup complexo.

**Solução:** Converter para testes unitários ou adicionar setup adequado nos doctests.

---

## 3. Análise de Funcionalidades

### 3.1 Comandos CLI Testados

| Comando | Status | Observações |
|---------|--------|-------------|
| `tetrad --help` | ✅ OK | |
| `tetrad version` | ✅ OK | |
| `tetrad status` | ✅ OK | Mostra CLIs disponíveis |
| `tetrad doctor` | ✅ OK | Diagnóstico funciona |
| `tetrad init` | ✅ OK | Cria tetrad.toml |
| `tetrad evaluate` | ⚠️ Parcial | Codex e Gemini falham |
| `tetrad serve` | ✅ OK | Servidor MCP inicia |
| `tetrad history` | ✅ OK | |
| `tetrad export` | ✅ OK | |
| `tetrad config` | ✅ OK | Interativo funciona |

### 3.2 MCP Tools

| Ferramenta | Implementação | Testada |
|------------|--------------|---------|
| `tetrad_review_plan` | ✅ Completa | Via unit tests |
| `tetrad_review_code` | ✅ Completa | Via unit tests |
| `tetrad_review_tests` | ✅ Completa | Via unit tests |
| `tetrad_confirm` | ✅ Completa | Via unit tests |
| `tetrad_final_check` | ✅ Completa | Via unit tests |
| `tetrad_status` | ✅ Completa | Testado manualmente |

---

## 4. Análise de Código

### 4.1 Estrutura do Projeto
```
Módulo          | Linhas | Cobertura Estimada
----------------|--------|-------------------
cli/            | ~600   | Alta (testes CLI)
consensus/      | ~400   | Alta (28 testes)
executors/      | ~450   | Média (9 testes)
hooks/          | ~300   | Alta (18 testes)
mcp/            | ~800   | Alta (27 testes)
reasoning/      | ~900   | Alta (14 testes)
cache/          | ~250   | Alta (12 testes)
types/          | ~350   | Alta (via usage)
```

### 4.2 Potenciais Melhorias de Código

#### 4.2.1 Error Handling nos Executores

**Localização:** `src/executors/*.rs:83-111`

O tratamento de erros poderia ser mais granular:

```rust
// Atual: erros genéricos
Err(TetradError::ExecutorFailed(self.name().to_string(), stderr.to_string()))

// Sugerido: erros específicos
Err(TetradError::ExecutorNoTerminal(self.name().to_string()))
Err(TetradError::ExecutorInvalidJson(self.name().to_string(), output))
Err(TetradError::ExecutorTimeout(self.name().to_string()))
```

#### 4.2.2 Argumentos Hardcoded

**Localização:** `src/types/config.rs:90-96`

Os argumentos padrão dos executores estão hardcoded:

```rust
// Gemini usa -o json, mas poderia não ser correto
gemini: ExecutorConfig::new("gemini", &["-o", "json"]),
```

**Sugestão:** Verificar dinamicamente a versão/help de cada CLI para determinar os argumentos corretos.

#### 4.2.3 Prompt Template

**Localização:** `src/executors/base.rs:66-96`

O template de prompt está em português e pode não ser ideal para todas as CLIs:

```rust
let mut prompt = format!(
    "Avalie o seguinte código {} para {}.\n\n",
    language, eval_type
);
```

**Sugestão:** Permitir templates configuráveis por executor ou usar inglês por padrão.

---

## 5. Recomendações de Ação

### 5.1 Prioridade Alta (Antes do Release)

1. **Corrigir integração com Codex CLI**
   - Investigar flags para modo não-interativo
   - Implementar pseudo-terminal se necessário
   - Adicionar testes de integração reais

2. **Corrigir integração com Gemini CLI**
   - Verificar formato de output correto
   - Melhorar parser de respostas
   - Adicionar fallback para diferentes formatos

3. **Resolver warning do Clippy**
   - Migrar para `tokio::sync::Mutex` ou
   - Implementar wrapper thread-safe para ReasoningBank

### 5.2 Prioridade Média

4. **Executar `cargo fmt`**
   ```bash
   cargo fmt
   ```

5. **Adicionar testes de integração para executores**
   - Mock das CLIs externas
   - Testes com responses simuladas

6. **Melhorar mensagens de erro**
   - Erros mais específicos
   - Sugestões de correção para usuários

### 5.3 Prioridade Baixa

7. **Documentação**
   - Corrigir ou remover doctests ignorados
   - Adicionar mais exemplos

8. **Performance**
   - Considerar connection pool para SQLite
   - Avaliar caching de assinaturas de código

---

## 6. Métricas do Projeto

```
Linguagem:        Rust 2021 Edition
Linhas de código: ~5.000 (estimado)
Dependências:     17 diretas
Testes:           205 (100% passando)
Warnings:         1 (clippy)
Erros:            0
```

---

## 7. Ambiente de Teste

```
SO:           Linux 6.14.0-37-generic
Rust:         1.92.0 (estimado)
Cargo:        1.92.0 (estimado)
Plataforma:   linux x86_64
```

---

## 8. Conclusão

O Tetrad está em bom estado de desenvolvimento com uma arquitetura sólida e boa cobertura de testes. Os principais bloqueadores para o release são:

1. **Integração com CLIs externas** - Os executores Codex e Gemini não funcionam corretamente em produção, limitando a funcionalidade de consenso quádruplo.

2. **Thread-safety do ReasoningBank** - O warning do clippy indica um potencial problema de concorrência que deve ser resolvido.

Uma vez resolvidos esses problemas, o projeto estará pronto para release em crates.io e outros canais de distribuição.

---

*Relatório gerado por debugging automatizado.*
