//! Transporte stdio para comunicação MCP.
//!
//! Implementa o protocolo de transporte MCP sobre stdin/stdout,
//! usando o formato de mensagens newline-delimited JSON conforme
//! a especificação MCP oficial.
//!
//! ## Formato de Mensagens
//!
//! Segundo a [especificação MCP](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports):
//! - Mensagens são delimitadas por newlines (`\n`)
//! - Mensagens NÃO DEVEM conter newlines embutidos
//! - Cada mensagem é um objeto JSON-RPC 2.0 completo em uma única linha
//!
//! ## Exemplo
//!
//! ```text
//! {"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}\n
//! {"jsonrpc":"2.0","id":1,"result":{...}}\n
//! ```

use std::io::{BufRead, BufReader, BufWriter, Stdin, Stdout, Write};

use crate::TetradResult;

use super::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// Transporte stdio para comunicação com o cliente MCP.
///
/// Implementa o protocolo MCP usando newline-delimited JSON sobre stdin/stdout.
pub struct StdioTransport {
    reader: BufReader<Stdin>,
    writer: BufWriter<Stdout>,
}

impl StdioTransport {
    /// Cria um novo transporte stdio.
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(std::io::stdin()),
            writer: BufWriter::new(std::io::stdout()),
        }
    }

    /// Lê uma mensagem JSON-RPC de stdin.
    ///
    /// O formato esperado é newline-delimited JSON:
    /// ```text
    /// {"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}\n
    /// ```
    ///
    /// Esta função bloqueia até receber uma linha completa.
    pub fn read_message(&mut self) -> TetradResult<JsonRpcRequest> {
        let mut line = String::new();

        // Lê uma linha completa de stdin
        let bytes_read = self
            .reader
            .read_line(&mut line)
            .map_err(crate::types::errors::TetradError::Io)?;

        // EOF detectado (0 bytes lidos)
        if bytes_read == 0 {
            return Err(crate::types::errors::TetradError::config("EOF"));
        }

        // Remove whitespace (incluindo \n e \r\n)
        let trimmed = line.trim();

        // Linha vazia = EOF ou mensagem inválida
        if trimmed.is_empty() {
            return Err(crate::types::errors::TetradError::config(
                "Empty message received",
            ));
        }

        // Parse do JSON
        let request: JsonRpcRequest =
            serde_json::from_str(trimmed).map_err(crate::types::errors::TetradError::Json)?;

        tracing::debug!(
            method = %request.method,
            id = ?request.id,
            "Received request"
        );

        Ok(request)
    }

    /// Escreve uma resposta JSON-RPC para stdout.
    ///
    /// A resposta é serializada como JSON compacto (sem newlines embutidos)
    /// seguido de um caractere newline (`\n`).
    pub fn write_response(&mut self, response: &JsonRpcResponse) -> TetradResult<()> {
        // Serializa como JSON compacto (sem pretty print para evitar newlines)
        let body =
            serde_json::to_string(response).map_err(crate::types::errors::TetradError::Json)?;

        self.write_message(&body)?;

        tracing::debug!(
            id = ?response.id,
            is_error = response.is_error(),
            "Sent response"
        );

        Ok(())
    }

    /// Envia uma notificação (mensagem sem ID que não espera resposta).
    pub fn send_notification(&mut self, notification: &JsonRpcNotification) -> TetradResult<()> {
        let body =
            serde_json::to_string(notification).map_err(crate::types::errors::TetradError::Json)?;

        self.write_message(&body)?;

        tracing::debug!(
            method = %notification.method,
            "Sent notification"
        );

        Ok(())
    }

    /// Escreve uma mensagem no formato MCP (newline-delimited JSON).
    ///
    /// Formato: `<json>\n`
    fn write_message(&mut self, body: &str) -> TetradResult<()> {
        // Escreve o JSON seguido de newline
        self.writer
            .write_all(body.as_bytes())
            .map_err(crate::types::errors::TetradError::Io)?;

        self.writer
            .write_all(b"\n")
            .map_err(crate::types::errors::TetradError::Io)?;

        // Flush é crítico para garantir que a mensagem seja enviada imediatamente
        self.writer
            .flush()
            .map_err(crate::types::errors::TetradError::Io)?;

        Ok(())
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

/// Transporte baseado em strings para testes.
///
/// Usa o mesmo formato newline-delimited JSON do StdioTransport.
#[cfg(test)]
pub struct StringTransport {
    input: std::io::Cursor<Vec<u8>>,
    output: Vec<u8>,
}

#[cfg(test)]
impl StringTransport {
    /// Cria um transporte com input pré-definido (newline-delimited JSON).
    pub fn new(input: &str) -> Self {
        Self {
            input: std::io::Cursor::new(input.as_bytes().to_vec()),
            output: Vec::new(),
        }
    }

    /// Lê uma mensagem JSON-RPC (newline-delimited).
    pub fn read_message(&mut self) -> TetradResult<JsonRpcRequest> {
        let mut line = String::new();

        use std::io::BufRead;
        let bytes_read = self
            .input
            .read_line(&mut line)
            .map_err(crate::types::errors::TetradError::Io)?;

        if bytes_read == 0 {
            return Err(crate::types::errors::TetradError::config("EOF"));
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err(crate::types::errors::TetradError::config("Empty message"));
        }

        serde_json::from_str(trimmed).map_err(crate::types::errors::TetradError::Json)
    }

    /// Escreve uma resposta (newline-delimited JSON).
    pub fn write_response(&mut self, response: &JsonRpcResponse) -> TetradResult<()> {
        let body =
            serde_json::to_string(response).map_err(crate::types::errors::TetradError::Json)?;

        self.output.extend_from_slice(body.as_bytes());
        self.output.push(b'\n');
        Ok(())
    }

    /// Retorna o output acumulado.
    pub fn get_output(&self) -> String {
        String::from_utf8_lossy(&self.output).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Cria uma mensagem no formato newline-delimited JSON.
    fn create_message(body: &str) -> String {
        format!("{}\n", body)
    }

    #[test]
    fn test_read_message() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let input = create_message(body);

        let mut transport = StringTransport::new(&input);
        let request = transport.read_message().unwrap();

        assert_eq!(request.method, "initialize");
        assert_eq!(
            request.id,
            Some(super::super::protocol::JsonRpcId::Number(1))
        );
    }

    #[test]
    fn test_write_response() {
        let mut transport = StringTransport::new("");

        let response = JsonRpcResponse::success(Some(1.into()), json!({"status": "ok"}));
        transport.write_response(&response).unwrap();

        let output = transport.get_output();
        // Verifica que a saída termina com newline
        assert!(output.ends_with('\n'));
        // Verifica que não há Content-Length header
        assert!(!output.contains("Content-Length"));
        // Verifica o conteúdo JSON
        assert!(output.contains("\"result\""));
        assert!(output.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_roundtrip() {
        // Cria uma request
        let original = JsonRpcRequest::new("test/method", Some(42.into()))
            .with_params(json!({"key": "value"}));

        let body = serde_json::to_string(&original).unwrap();
        let message = create_message(&body);

        // Lê a request
        let mut transport = StringTransport::new(&message);
        let parsed = transport.read_message().unwrap();

        assert_eq!(original.method, parsed.method);
        assert_eq!(original.id, parsed.id);
    }

    #[test]
    fn test_multiple_messages() {
        let messages = concat!(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            "\n",
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
            "\n"
        );

        let mut transport = StringTransport::new(messages);

        // Lê primeira mensagem
        let request1 = transport.read_message().unwrap();
        assert_eq!(request1.method, "initialize");
        assert_eq!(
            request1.id,
            Some(super::super::protocol::JsonRpcId::Number(1))
        );

        // Lê segunda mensagem
        let request2 = transport.read_message().unwrap();
        assert_eq!(request2.method, "tools/list");
        assert_eq!(
            request2.id,
            Some(super::super::protocol::JsonRpcId::Number(2))
        );
    }

    #[test]
    fn test_empty_input() {
        let mut transport = StringTransport::new("");
        let result = transport.read_message();
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_line() {
        let mut transport = StringTransport::new("\n");
        let result = transport.read_message();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json() {
        let mut transport = StringTransport::new("not valid json\n");
        let result = transport.read_message();
        assert!(result.is_err());
    }

    #[test]
    fn test_notification_without_id() {
        let body = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let input = create_message(body);

        let mut transport = StringTransport::new(&input);
        let request = transport.read_message().unwrap();

        assert_eq!(request.method, "notifications/initialized");
        assert!(request.id.is_none());
    }

    #[test]
    fn test_output_format() {
        let mut transport = StringTransport::new("");

        let response = JsonRpcResponse::success(
            Some(1.into()),
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "tetrad", "version": "0.1.0"}
            }),
        );
        transport.write_response(&response).unwrap();

        let output = transport.get_output();

        // Verifica formato newline-delimited (uma linha JSON + newline)
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1);

        // Verifica que o JSON é válido
        let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 1);
        assert!(parsed["result"].is_object());
    }
}
