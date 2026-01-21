//! Tipos do protocolo MCP (Model Context Protocol).
//!
//! O MCP usa JSON-RPC 2.0 como protocolo de transporte.
//! Este módulo define todos os tipos necessários para comunicação.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ═══════════════════════════════════════════════════════════════════════════
// Códigos de erro JSON-RPC padrão
// ═══════════════════════════════════════════════════════════════════════════

/// Erro de parse - JSON inválido.
pub const PARSE_ERROR: i32 = -32700;

/// Request inválida - JSON-RPC malformado.
pub const INVALID_REQUEST: i32 = -32600;

/// Método não encontrado.
pub const METHOD_NOT_FOUND: i32 = -32601;

/// Parâmetros inválidos.
pub const INVALID_PARAMS: i32 = -32602;

/// Erro interno do servidor.
pub const INTERNAL_ERROR: i32 = -32603;

// ═══════════════════════════════════════════════════════════════════════════
// Tipos básicos JSON-RPC
// ═══════════════════════════════════════════════════════════════════════════

/// ID de uma request JSON-RPC (pode ser número ou string).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum JsonRpcId {
    Number(i64),
    String(String),
}

impl From<i64> for JsonRpcId {
    fn from(n: i64) -> Self {
        JsonRpcId::Number(n)
    }
}

impl From<String> for JsonRpcId {
    fn from(s: String) -> Self {
        JsonRpcId::String(s)
    }
}

impl From<&str> for JsonRpcId {
    fn from(s: &str) -> Self {
        JsonRpcId::String(s.to_string())
    }
}

/// Request JSON-RPC 2.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Versão do protocolo (sempre "2.0").
    pub jsonrpc: String,

    /// ID da request (opcional para notificações).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,

    /// Nome do método a ser chamado.
    pub method: String,

    /// Parâmetros do método (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Cria uma nova request.
    pub fn new(method: impl Into<String>, id: Option<JsonRpcId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params: None,
        }
    }

    /// Adiciona parâmetros à request.
    pub fn with_params(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }

    /// Verifica se é uma notificação (sem ID).
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// Response JSON-RPC 2.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Versão do protocolo (sempre "2.0").
    pub jsonrpc: String,

    /// ID da request original.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,

    /// Resultado em caso de sucesso.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Erro em caso de falha.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Cria uma response de sucesso.
    pub fn success(id: Option<JsonRpcId>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Cria uma response de erro.
    pub fn error(id: Option<JsonRpcId>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Verifica se a response é um erro.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// Erro JSON-RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Código do erro.
    pub code: i32,

    /// Mensagem de erro.
    pub message: String,

    /// Dados adicionais (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Cria um novo erro.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Adiciona dados ao erro.
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Erro de parse.
    pub fn parse_error() -> Self {
        Self::new(PARSE_ERROR, "Parse error")
    }

    /// Request inválida.
    pub fn invalid_request() -> Self {
        Self::new(INVALID_REQUEST, "Invalid Request")
    }

    /// Método não encontrado.
    pub fn method_not_found(method: &str) -> Self {
        Self::new(METHOD_NOT_FOUND, format!("Method not found: {}", method))
    }

    /// Parâmetros inválidos.
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(INVALID_PARAMS, message)
    }

    /// Erro interno.
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(INTERNAL_ERROR, message)
    }
}

/// Notificação JSON-RPC (request sem ID, não espera resposta).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// Versão do protocolo.
    pub jsonrpc: String,

    /// Método.
    pub method: String,

    /// Parâmetros.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    /// Cria uma nova notificação.
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: None,
        }
    }

    /// Adiciona parâmetros.
    pub fn with_params(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tipos MCP específicos
// ═══════════════════════════════════════════════════════════════════════════

/// Informações do servidor MCP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// Nome do servidor.
    pub name: String,

    /// Versão do servidor.
    pub version: String,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: "tetrad".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Capacidades do servidor.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    /// Capacidades de ferramentas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

/// Capacidade de ferramentas.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    /// Suporta listagem de ferramentas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resultado da inicialização.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// Versão do protocolo suportada.
    pub protocol_version: String,

    /// Capacidades do servidor.
    pub capabilities: ServerCapabilities,

    /// Informações do servidor.
    pub server_info: ServerInfo,
}

impl Default for InitializeResult {
    fn default() -> Self {
        Self {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability::default()),
            },
            server_info: ServerInfo::default(),
        }
    }
}

/// Descrição de uma ferramenta MCP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDescription {
    /// Nome da ferramenta.
    pub name: String,

    /// Descrição da ferramenta.
    pub description: String,

    /// Schema de entrada (JSON Schema).
    pub input_schema: Value,
}

impl ToolDescription {
    /// Cria uma nova descrição de ferramenta.
    pub fn new(name: impl Into<String>, description: impl Into<String>, input_schema: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// Resultado da listagem de ferramentas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// Lista de ferramentas disponíveis.
    pub tools: Vec<ToolDescription>,
}

/// Parâmetros para chamada de ferramenta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    /// Nome da ferramenta.
    pub name: String,

    /// Argumentos da ferramenta.
    #[serde(default)]
    pub arguments: Value,
}

/// Conteúdo retornado por uma ferramenta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolContent {
    /// Conteúdo de texto.
    Text {
        text: String,
    },
}

impl ToolContent {
    /// Cria conteúdo de texto.
    pub fn text(text: impl Into<String>) -> Self {
        ToolContent::Text { text: text.into() }
    }
}

/// Resultado de chamada de ferramenta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    /// Conteúdo retornado.
    pub content: Vec<ToolContent>,

    /// Se a chamada resultou em erro.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

impl ToolResult {
    /// Cria um resultado de sucesso com texto.
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(text)],
            is_error: false,
        }
    }

    /// Cria um resultado de sucesso com JSON.
    pub fn success_json(value: &Value) -> Self {
        Self {
            content: vec![ToolContent::text(serde_json::to_string_pretty(value).unwrap_or_default())],
            is_error: false,
        }
    }

    /// Cria um resultado de erro.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(message)],
            is_error: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Testes
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_rpc_id_number() {
        let id: JsonRpcId = 42.into();
        assert_eq!(id, JsonRpcId::Number(42));
    }

    #[test]
    fn test_json_rpc_id_string() {
        let id: JsonRpcId = "test-id".into();
        assert_eq!(id, JsonRpcId::String("test-id".to_string()));
    }

    #[test]
    fn test_json_rpc_request_serialize() {
        let request = JsonRpcRequest::new("test/method", Some(1.into()))
            .with_params(json!({"key": "value"}));

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test/method\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_json_rpc_request_deserialize() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "initialize");
        assert_eq!(request.id, Some(JsonRpcId::Number(1)));
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(Some(1.into()), json!({"status": "ok"}));

        assert!(!response.is_error());
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::error(
            Some(1.into()),
            JsonRpcError::method_not_found("unknown"),
        );

        assert!(response.is_error());
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_json_rpc_error_codes() {
        let parse_err = JsonRpcError::parse_error();
        assert_eq!(parse_err.code, PARSE_ERROR);

        let invalid_req = JsonRpcError::invalid_request();
        assert_eq!(invalid_req.code, INVALID_REQUEST);

        let method_err = JsonRpcError::method_not_found("test");
        assert_eq!(method_err.code, METHOD_NOT_FOUND);
    }

    #[test]
    fn test_tool_description() {
        let tool = ToolDescription::new(
            "test_tool",
            "A test tool",
            json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                }
            }),
        );

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("Operation completed");
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Something went wrong");
        assert!(result.is_error);
    }

    #[test]
    fn test_initialize_result_default() {
        let result = InitializeResult::default();
        assert_eq!(result.server_info.name, "tetrad");
        assert!(result.capabilities.tools.is_some());
    }

    #[test]
    fn test_notification() {
        let notif = JsonRpcNotification::new("initialized");
        assert_eq!(notif.method, "initialized");
        assert!(notif.params.is_none());
    }
}
