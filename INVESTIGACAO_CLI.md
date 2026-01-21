# Investigacao: Integracao com Codex CLI e Gemini CLI

**Data:** 2026-01-21
**Status:** Investigacao Completa

---

## Resumo

A investigacao identificou as causas raiz dos problemas de integracao com Codex CLI e Gemini CLI, e propos solucoes especificas para cada caso.

---

## 1. Problema: Codex CLI - "stdin is not a terminal"

### Causa Raiz

O Tetrad esta usando o comando `codex <prompt>` que inicia o **modo interativo** do Codex, que requer um terminal TTY (pseudo-terminal). Quando executado via `tokio::process::Command`, nao ha TTY disponivel.

### Solucao: Usar `codex exec`

O Codex CLI possui um modo nao-interativo chamado `codex exec` projetado especificamente para automacao:

```bash
# Modo correto para uso programatico
codex exec --json "seu prompt aqui"

# Ou via stdin
echo "seu prompt" | codex exec --json -
```

### Output do `codex exec --json`

O comando retorna **JSON Lines** (NDJSON) com eventos:

```json
{"type":"thread.started","thread_id":"..."}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"item_0","type":"reasoning","text":"..."}}
{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"Resposta final aqui"}}
{"type":"turn.completed","usage":{"input_tokens":3450,"output_tokens":125}}
```

### Alteracoes Necessarias em `src/executors/codex.rs`

```rust
// ANTES (modo interativo - NAO FUNCIONA)
let mut cmd = Command::new(&self.command_name);
cmd.arg(&prompt);

// DEPOIS (modo nao-interativo)
let mut cmd = Command::new(&self.command_name);
cmd.arg("exec");
cmd.arg("--json");
cmd.arg(&prompt);
```

### Parser para Codex Events

Criar funcao para extrair a mensagem do agente dos eventos:

```rust
fn parse_codex_events(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
            if event["type"] == "item.completed"
                && event["item"]["type"] == "agent_message" {
                return event["item"]["text"].as_str().map(String::from);
            }
        }
    }
    None
}
```

### Referencias
- [Codex Non-Interactive Mode](https://developers.openai.com/codex/noninteractive/)
- [Codex CLI Reference](https://developers.openai.com/codex/cli/reference/)

---

## 2. Problema: Gemini CLI - "Resposta nao contem JSON valido"

### Causa Raiz

O Gemini CLI com `-o json` retorna um JSON **wrapper**, nao o JSON estruturado esperado pelo Tetrad:

```json
{
  "session_id": "978acc88-2486-4942-bbcd-0a1e011615e3",
  "response": "A funcao `add` fornecida e uma implementacao idiomatica...",
  "stats": { ... }
}
```

O campo `response` contem **texto livre**, nao JSON estruturado. O parser do Tetrad procura por `{"vote": ..., "score": ...}` mas encontra apenas o wrapper.

### Bug Conhecido

Ha um bug reportado onde `--output-format json` retorna markdown dentro do JSON quando streaming esta ativo:
- [Issue #11184](https://github.com/google-gemini/gemini-cli/issues/11184)
- [Issue #9009](https://github.com/google-gemini/gemini-cli/issues/9009)

### Solucao: Parser de Duas Fases

1. **Fase 1:** Parsear o JSON wrapper do Gemini
2. **Fase 2:** Extrair e parsear JSON do campo `response` (se houver)

### Alteracoes Necessarias em `src/executors/gemini.rs`

```rust
/// Estrutura do wrapper JSON do Gemini CLI
#[derive(Deserialize)]
struct GeminiWrapper {
    session_id: String,
    response: String,
    #[serde(default)]
    stats: serde_json::Value,
}

fn parse_gemini_output(output: &str) -> TetradResult<ExecutorResponse> {
    // Fase 1: Parsear o wrapper
    let wrapper: GeminiWrapper = serde_json::from_str(output)
        .map_err(|e| TetradError::ExecutorFailed("Gemini".into(), e.to_string()))?;

    // Fase 2: Tentar extrair JSON do campo response
    if let Some(json_str) = ExecutorResponse::find_balanced_json(&wrapper.response) {
        if let Ok(response) = serde_json::from_str::<ExecutorResponse>(json_str) {
            return Ok(response);
        }
    }

    // Fallback: Analisar o texto da resposta semanticamente
    analyze_text_response(&wrapper.response)
}
```

### Analise Semantica como Fallback

Se o modelo nao retornar JSON estruturado, analisar o texto:

```rust
fn analyze_text_response(text: &str) -> TetradResult<ExecutorResponse> {
    let lower = text.to_lowercase();

    // Heuristicas simples baseadas em palavras-chave
    let vote = if lower.contains("erro") || lower.contains("bug") || lower.contains("falha") {
        "FAIL"
    } else if lower.contains("atencao") || lower.contains("considere") || lower.contains("sugestao") {
        "WARN"
    } else {
        "PASS"
    };

    // Score baseado em sentimento geral
    let score = if vote == "PASS" { 85 } else if vote == "WARN" { 65 } else { 35 };

    Ok(ExecutorResponse {
        vote: vote.to_string(),
        score,
        reasoning: text.chars().take(500).collect(),
        issues: vec![],
        suggestions: vec![],
    })
}
```

### Referencias
- [Gemini CLI GitHub](https://github.com/google-gemini/gemini-cli)
- [Gemini CLI Configuration](https://geminicli.com/docs/get-started/configuration/)

---

## 3. Configuracao Recomendada

### tetrad.toml

```toml
[executors.codex]
enabled = true
command = "codex"
args = ["exec", "--json"]  # IMPORTANTE: usar exec --json
timeout_secs = 60

[executors.gemini]
enabled = true
command = "gemini"
args = ["-o", "json"]  # OK, mas requer parser especial
timeout_secs = 60

[executors.qwen]
enabled = true
command = "qwen"
args = ["-p"]
timeout_secs = 60
```

---

## 4. Resumo das Alteracoes Necessarias

| Arquivo | Alteracao |
|---------|-----------|
| `src/executors/codex.rs` | Usar `codex exec --json` e parsear eventos NDJSON |
| `src/executors/gemini.rs` | Parsear wrapper JSON e extrair do campo `response` |
| `src/executors/base.rs` | Adicionar funcoes auxiliares de parsing |
| `src/types/config.rs` | Atualizar args padrao do Codex para `["exec", "--json"]` |

---

## 5. Testes de Validacao

### Codex (funciona)

```bash
$ echo 'Avalie: fn add(a: i32, b: i32) -> i32 { a + b }' | codex exec --json -

{"type":"thread.started","thread_id":"..."}
{"type":"item.completed","item":{"type":"agent_message","text":"Nenhum problema..."}}
{"type":"turn.completed","usage":{...}}
```

### Gemini (funciona)

```bash
$ echo 'Avalie: fn add(a: i32, b: i32) -> i32 { a + b }' | gemini -o json

{
  "session_id": "...",
  "response": "A funcao `add` fornecida e uma implementacao idiomatica...",
  "stats": {...}
}
```

---

## 6. Proximos Passos

1. **Implementar** alteracoes nos executores Codex e Gemini
2. **Testar** com diferentes tipos de codigo
3. **Adicionar** testes unitarios para os novos parsers
4. **Documentar** as mudancas no CHANGELOG

---

## 7. Alternativas Consideradas

### Usar `--output-schema` (Codex)

O Codex suporta `--output-schema <path>` para forcar resposta JSON estruturada. Isso poderia ser uma alternativa mais robusta:

```bash
codex exec --output-schema schema.json "Avalie este codigo..."
```

Onde `schema.json`:
```json
{
  "type": "object",
  "properties": {
    "vote": { "type": "string", "enum": ["PASS", "WARN", "FAIL"] },
    "score": { "type": "integer", "minimum": 0, "maximum": 100 },
    "reasoning": { "type": "string" },
    "issues": { "type": "array", "items": { "type": "string" } },
    "suggestions": { "type": "array", "items": { "type": "string" } }
  },
  "required": ["vote", "score", "reasoning"]
}
```

### Usar API diretamente (ambos)

Ambos Codex e Gemini possuem APIs REST/SDK que poderiam ser usadas diretamente, evitando as limitacoes das CLIs. Isso seria mais robusto mas aumentaria a complexidade do codigo.

---

*Relatorio gerado apos investigacao de compatibilidade das CLIs externas.*
