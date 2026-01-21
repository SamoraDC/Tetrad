# Correções de Integração com CLIs Externas

**Data:** 2026-01-21
**Status:** Concluído com Sucesso

---

## Resumo

As integrações com Codex CLI e Gemini CLI foram corrigidas e testadas rigorosamente. O MCP Tetrad agora funciona corretamente com todos os três executores.

---

## 1. Correção do Codex CLI

### Problema
O Codex CLI estava falhando com o erro "stdin is not a terminal" porque o comando `codex <prompt>` inicia o modo interativo que requer um terminal TTY.

### Solução
Alterado para usar `codex exec --json` que é o modo não-interativo projetado para automação.

### Arquivos Modificados

**`src/executors/codex.rs`:**
- Adicionado parser `parse_codex_events()` para eventos NDJSON
- Adicionado `analyze_text_response()` para fallback semântico
- Alterado args padrão para `["exec", "--json"]`

**`src/types/config.rs`:**
- Atualizado args padrão: `ExecutorConfig::new("codex", &["exec", "--json"])`

**`tetrad.toml`:**
- Atualizado `args = ["exec", "--json"]`

### Formato de Saída do Codex exec --json
```json
{"type":"thread.started","thread_id":"..."}
{"type":"turn.started"}
{"type":"item.completed","item":{"type":"agent_message","text":"Resposta aqui"}}
{"type":"turn.completed","usage":{"input_tokens":100,"output_tokens":50}}
```

---

## 2. Correção do Gemini CLI

### Problema
O Gemini CLI com `-o json` retorna um JSON wrapper ao invés do JSON estruturado esperado:
```json
{
  "session_id": "...",
  "response": "texto da resposta",
  "stats": {...}
}
```

### Solução
Implementado parser de duas fases que extrai o conteúdo do campo `response` do wrapper.

### Arquivos Modificados

**`src/executors/gemini.rs`:**
- Adicionada struct `GeminiWrapper` para deserializar o wrapper
- Adicionado método `parse_gemini_output()` para parser de duas fases
- Adicionado `analyze_text_response()` para fallback semântico
- Lógica: tenta extrair JSON estruturado do `response`, senão analisa texto semanticamente

---

## 3. Testes Realizados

### Testes Unitários
- 219 testes passando (141 unit + 78 integration)
- 0 warnings do clippy

### Testes de Integração MCP
| Teste | Resultado |
|-------|-----------|
| Initialize (Handshake) | ✓ PASS |
| tetrad_status | ✓ Codex 0.87.0, Gemini 0.25.0, Qwen 0.7.2 |
| tetrad_review_code (Rust) | ✓ PASS (score 91) |
| tetrad_review_code (com bug) | ✓ BLOCK (score 26) - problema detectado |
| tetrad_review_code (Python) | ✓ REVISE (score 64) |
| tetrad_review_code (JavaScript) | ✓ REVISE (score 67) |
| tetrad_review_tests | ✓ REVISE (score 80) |
| tetrad_confirm | ✓ PASS |
| tetrad_final_check | ✓ CERTIFICADO |
| tetrad_review_plan | ✓ REVISE (score 72) |

**Taxa de sucesso: 100%**

---

## 4. Validação de Consenso

O sistema de consenso está funcionando corretamente:

- **Código bom** → PASS com consenso (3/3 votos positivos)
- **Código com bug crítico** → BLOCK com consenso (3/3 votos negativos)
- **Código com melhorias sugeridas** → REVISE (sem consenso unânime)

Exemplo de votação bem-sucedida:
```
Votos:
  - Codex: Pass (score: 95)
  - Gemini: Pass (score: 90)
  - Qwen: Pass (score: 90)
Decision: PASS
Score: 91
Consensus: True
```

---

## 5. Certificação

O sistema de certificação está funcionando:
```
Certificate ID: TETRAD-2369cea0-06e2-4401-8179-3eb8f50f53e8
```

---

## 6. Arquivos de Teste

Scripts de teste criados (podem ser removidos):
- `test_single.py` - Teste rápido de uma revisão
- `test_complete.py` - Suite completa de testes
- `test_mcp_full.sh` - Script bash (problemas com escape)
- `test_mcp_rigorous.py` - Versão anterior

---

## 7. Próximos Passos

1. ✅ Implementação concluída
2. ✅ Testes unitários passando
3. ✅ Testes de integração passando
4. Considerar adicionar mais testes edge-case
5. Documentar no README as configurações necessárias das CLIs

---

*Relatório gerado após correções bem-sucedidas das integrações CLI.*
