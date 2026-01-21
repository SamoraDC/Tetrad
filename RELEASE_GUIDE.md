# Guia de Release do Tetrad

## Opções de Distribuição

### Opção A: Publicar no crates.io (Recomendado para Rust)

```bash
# 1. Criar conta em crates.io e obter token
# Acesse: https://crates.io/settings/tokens

# 2. Login no cargo
cargo login <seu-token>

# 3. Verificar se o pacote está pronto
cargo publish --dry-run

# 4. Publicar
cargo publish

# Usuários instalam com:
cargo install tetrad
```

### Opção B: GitHub Releases (Binários pré-compilados)

```bash
# 1. Criar tag de versão
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# 2. Compilar para diferentes plataformas
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# macOS x86_64
cargo build --release --target x86_64-apple-darwin

# macOS ARM (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-msvc

# 3. Criar arquivos compactados
cd target/release
tar -czvf tetrad-v0.1.0-linux-x86_64.tar.gz tetrad
# ... para cada plataforma

# 4. Criar release no GitHub
gh release create v0.1.0 \
  --title "Tetrad v0.1.0" \
  --notes "Release inicial do Tetrad MCP" \
  tetrad-v0.1.0-linux-x86_64.tar.gz \
  tetrad-v0.1.0-macos-x86_64.tar.gz \
  tetrad-v0.1.0-macos-arm64.tar.gz \
  tetrad-v0.1.0-windows-x86_64.zip
```

### Opção C: Homebrew (macOS/Linux)

Criar um tap do Homebrew em `homebrew-tetrad`:

```ruby
# Formula/tetrad.rb
class Tetrad < Formula
  desc "MCP de Consenso Quádruplo para Claude Code"
  homepage "https://github.com/SamoraDC/tetrad"
  url "https://github.com/SamoraDC/tetrad/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "..." # SHA256 do arquivo
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/tetrad", "--version"
  end
end
```

Usuários instalam com:
```bash
brew tap SamoraDC/tetrad
brew install tetrad
```

---

## Pré-requisitos para Usuários

O Tetrad requer que as CLIs externas estejam instaladas:

### 1. Codex CLI (OpenAI)
```bash
# Via npm
npm install -g @anthropic/codex

# Ou via pip
pip install codex-cli

# Configurar API key
export OPENAI_API_KEY="sua-chave"
```

### 2. Gemini CLI (Google)
```bash
# Via npm
npm install -g @anthropic/gemini-cli

# Configurar
gcloud auth application-default login
# Ou
export GOOGLE_API_KEY="sua-chave"
```

### 3. Qwen CLI
```bash
# Via pip
pip install qwen-cli

# Configurar
export DASHSCOPE_API_KEY="sua-chave"
```

---

## Como Usar com Claude Code

### Passo 1: Instalar o Tetrad

```bash
# Opção 1: Via cargo
cargo install tetrad

# Opção 2: Download binário
curl -L https://github.com/SamoraDC/tetrad/releases/latest/download/tetrad-linux-x86_64.tar.gz | tar xz
sudo mv tetrad /usr/local/bin/

# Opção 3: Homebrew
brew install SamoraDC/tetrad/tetrad
```

### Passo 2: Verificar Instalação

```bash
# Verificar versão
tetrad version

# Verificar CLIs externas
tetrad status

# Diagnosticar problemas
tetrad doctor
```

### Passo 3: Configurar Claude Code

Adicionar o Tetrad como servidor MCP no arquivo de configuração do Claude Code:

**Localização do arquivo:**
- Linux: `~/.config/claude-code/config.json`
- macOS: `~/Library/Application Support/claude-code/config.json`
- Windows: `%APPDATA%\claude-code\config.json`

**Conteúdo:**
```json
{
  "mcpServers": {
    "tetrad": {
      "command": "tetrad",
      "args": ["serve"],
      "env": {}
    }
  }
}
```

### Passo 4: Reiniciar Claude Code

Após configurar, reinicie o Claude Code para carregar o servidor MCP.

---

## Fluxo de Uso no Claude Code

### Cenário: Usuário pede para criar uma função

```
Usuário: "Crie uma função em Rust que calcula fatorial"
```

### O que acontece internamente:

1. **Claude Code escreve o código:**
```rust
fn factorial(n: u64) -> u64 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
```

2. **Antes de salvar, Claude Code chama `tetrad_review_code`:**
```json
{
  "code": "fn factorial(n: u64) -> u64 {...}",
  "language": "rust",
  "file_path": "src/math.rs"
}
```

3. **Tetrad executa os 3 avaliadores em paralelo:**
   - Codex analisa sintaxe e convenções
   - Gemini analisa arquitetura
   - Qwen analisa lógica

4. **Tetrad retorna feedback consolidado:**
```json
{
  "decision": "REVISE",
  "score": 72,
  "feedback": "Considere adicionar tratamento de overflow...",
  "findings": [
    {"issue": "Risco de stack overflow para n grande", "severity": "Warning"}
  ]
}
```

5. **Claude Code mostra o feedback e ajusta:**
```rust
fn factorial(n: u64) -> Option<u64> {
    match n {
        0 | 1 => Some(1),
        _ => n.checked_mul(factorial(n - 1)?)
    }
}
```

6. **Claude Code chama `tetrad_final_check`:**
```json
{
  "decision": "PASS",
  "certified": true,
  "certificate_id": "TETRAD-abc123..."
}
```

7. **Código salvo com certificação!**

---

## Ferramentas MCP Disponíveis

| Ferramenta | Quando Usar | Exemplo |
|------------|-------------|---------|
| `tetrad_review_plan` | Antes de implementar | "Vou criar um sistema de auth..." |
| `tetrad_review_code` | Antes de salvar código | Qualquer código novo |
| `tetrad_review_tests` | Antes de finalizar testes | Testes unitários |
| `tetrad_confirm` | Após receber feedback | "Corrigi os problemas" |
| `tetrad_final_check` | Antes de commit | Código final |
| `tetrad_status` | Diagnóstico | Ver status das CLIs |

---

## Configuração Avançada

### Arquivo tetrad.toml

```toml
[general]
log_level = "info"  # trace, debug, info, warn, error

[executors.codex]
enabled = true
command = "codex"
args = ["exec", "--json"]
timeout_secs = 60

[executors.gemini]
enabled = true
command = "gemini"
args = ["-o", "json"]
timeout_secs = 60

[executors.qwen]
enabled = true
command = "qwen"
args = []
timeout_secs = 60

[consensus]
default_rule = "strong"  # golden (3/3), strong (3/3 ou 2/3 alto), weak (2/3)
min_score = 70
max_loops = 3

[reasoning]
enabled = true
db_path = "tetrad.db"

[cache]
enabled = true
capacity = 1000
ttl_secs = 300
```

---

## Troubleshooting

### "CLI não encontrada"
```bash
# Verificar se está no PATH
which codex
which gemini
which qwen

# Verificar configuração
tetrad doctor
```

### "stdin is not a terminal" (Codex)
```bash
# Verificar configuração
cat tetrad.toml | grep -A3 codex
# Deve ter: args = ["exec", "--json"]
```

### "Resposta não contém JSON válido" (Gemini)
```bash
# Verificar configuração
cat tetrad.toml | grep -A3 gemini
# Deve ter: args = ["-o", "json"]
```

---

## Links Úteis

- Repositório: https://github.com/SamoraDC/tetrad
- Documentação: https://docs.rs/tetrad
- Issues: https://github.com/SamoraDC/tetrad/issues
