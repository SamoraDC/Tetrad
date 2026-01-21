//! Transporte stdio para comunicação MCP.
//!
//! Implementa o protocolo de transporte MCP sobre stdin/stdout,
//! usando o formato de mensagens com header Content-Length.

use std::io::{BufRead, BufReader, BufWriter, Read, Stdin, Stdout, Write};

use crate::TetradResult;

use super::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// Transporte stdio para comunicação com o cliente MCP.
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
    /// O formato esperado é:
    /// ```text
    /// Content-Length: <bytes>\r\n
    /// \r\n
    /// <json body>
    /// ```
    pub fn read_message(&mut self) -> TetradResult<JsonRpcRequest> {
        // Lê o header Content-Length
        let content_length = self.read_content_length()?;

        // Lê o body JSON
        let mut body = vec![0u8; content_length];
        self.reader
            .read_exact(&mut body)
            .map_err(crate::types::errors::TetradError::Io)?;

        // Parse do JSON
        let request: JsonRpcRequest =
            serde_json::from_slice(&body).map_err(crate::types::errors::TetradError::Json)?;

        tracing::debug!(
            method = %request.method,
            id = ?request.id,
            "Received request"
        );

        Ok(request)
    }

    /// Lê o header Content-Length e retorna o tamanho do body.
    fn read_content_length(&mut self) -> TetradResult<usize> {
        let mut content_length: Option<usize> = None;
        let mut first_line = true;

        loop {
            let mut line = String::new();
            let bytes_read = self
                .reader
                .read_line(&mut line)
                .map_err(crate::types::errors::TetradError::Io)?;

            // EOF detectado (0 bytes lidos)
            if bytes_read == 0 {
                return Err(crate::types::errors::TetradError::config("EOF"));
            }

            // Remove \r\n
            let trimmed = line.trim();

            // Linha vazia indica fim dos headers
            // Mas se for a primeira linha vazia, é EOF disfarçado
            if trimmed.is_empty() {
                if first_line {
                    return Err(crate::types::errors::TetradError::config("EOF"));
                }
                break;
            }

            first_line = false;

            // Parse do header Content-Length
            if let Some(value) = trimmed.strip_prefix("Content-Length:") {
                content_length = Some(value.trim().parse().map_err(|_| {
                    crate::types::errors::TetradError::config("Invalid Content-Length header")
                })?);
            }
        }

        content_length.ok_or_else(|| {
            crate::types::errors::TetradError::config("Missing Content-Length header")
        })
    }

    /// Escreve uma resposta JSON-RPC para stdout.
    pub fn write_response(&mut self, response: &JsonRpcResponse) -> TetradResult<()> {
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

    /// Escreve uma mensagem com o formato MCP (Content-Length header).
    fn write_message(&mut self, body: &str) -> TetradResult<()> {
        let message = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);

        self.writer
            .write_all(message.as_bytes())
            .map_err(crate::types::errors::TetradError::Io)?;

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
#[cfg(test)]
pub struct StringTransport {
    input: std::io::Cursor<Vec<u8>>,
    output: Vec<u8>,
}

#[cfg(test)]
impl StringTransport {
    /// Cria um transporte com input pré-definido.
    pub fn new(input: &str) -> Self {
        Self {
            input: std::io::Cursor::new(input.as_bytes().to_vec()),
            output: Vec::new(),
        }
    }

    /// Lê uma mensagem JSON-RPC.
    pub fn read_message(&mut self) -> TetradResult<JsonRpcRequest> {
        // Lê headers
        let mut content_length: Option<usize> = None;
        let mut line = String::new();

        loop {
            line.clear();
            use std::io::BufRead;
            self.input
                .read_line(&mut line)
                .map_err(crate::types::errors::TetradError::Io)?;

            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }

            if let Some(value) = trimmed.strip_prefix("Content-Length:") {
                content_length = Some(value.trim().parse().map_err(|_| {
                    crate::types::errors::TetradError::config("Invalid Content-Length")
                })?);
            }
        }

        let length = content_length
            .ok_or_else(|| crate::types::errors::TetradError::config("Missing Content-Length"))?;

        let mut body = vec![0u8; length];
        std::io::Read::read_exact(&mut self.input, &mut body)
            .map_err(crate::types::errors::TetradError::Io)?;

        serde_json::from_slice(&body).map_err(crate::types::errors::TetradError::Json)
    }

    /// Escreve uma resposta.
    pub fn write_response(&mut self, response: &JsonRpcResponse) -> TetradResult<()> {
        let body =
            serde_json::to_string(response).map_err(crate::types::errors::TetradError::Json)?;

        let message = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        self.output.extend_from_slice(message.as_bytes());
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

    fn create_message(body: &str) -> String {
        format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
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
        assert!(output.contains("Content-Length:"));
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
    fn test_multiple_headers() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
        let input = format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        let mut transport = StringTransport::new(&input);
        let request = transport.read_message().unwrap();

        assert_eq!(request.method, "test");
    }

    #[test]
    fn test_missing_content_length() {
        let input = "Content-Type: application/json\r\n\r\n{}";
        let mut transport = StringTransport::new(input);

        let result = transport.read_message();
        assert!(result.is_err());
    }
}
