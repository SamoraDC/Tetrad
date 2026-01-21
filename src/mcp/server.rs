//! Servidor MCP do Tetrad.
//!
//! Implementa o servidor MCP (Model Context Protocol) que expõe
//! as ferramentas de avaliação do Tetrad para o Claude Code.

use serde_json::json;

use crate::types::config::Config;
use crate::TetradResult;

use super::protocol::{
    CallToolParams, InitializeResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    ListToolsResult,
};
use super::tools::ToolHandler;
use super::transport::StdioTransport;

/// Servidor MCP do Tetrad.
pub struct McpServer {
    transport: StdioTransport,
    tools: ToolHandler,
    initialized: bool,
}

impl McpServer {
    /// Cria um novo servidor MCP.
    pub fn new(config: Config) -> TetradResult<Self> {
        let tools = ToolHandler::new(config)?;

        Ok(Self {
            transport: StdioTransport::new(),
            tools,
            initialized: false,
        })
    }

    /// Inicia o servidor (loop principal).
    ///
    /// Este método bloqueia e processa mensagens indefinidamente.
    pub async fn run(&mut self) -> TetradResult<()> {
        tracing::info!("Tetrad MCP Server starting...");

        loop {
            // Lê a próxima mensagem
            let request = match self.transport.read_message() {
                Ok(req) => req,
                Err(e) => {
                    // EOF ou erro de leitura - cliente desconectou
                    if e.to_string().contains("EOF") || e.to_string().contains("empty") {
                        tracing::info!("Client disconnected");
                        break;
                    }
                    tracing::error!(error = %e, "Failed to read message");
                    continue;
                }
            };

            // Notificações (sem ID) não devem receber resposta segundo JSON-RPC 2.0
            let is_notification = request.id.is_none();

            // Processa a request
            let response = self.handle_request(request).await;

            // Envia resposta apenas se não for notificação
            if !is_notification {
                if let Err(e) = self.transport.write_response(&response) {
                    tracing::error!(error = %e, "Failed to write response");
                }
            }
        }

        tracing::info!("Tetrad MCP Server stopped");
        Ok(())
    }

    /// Processa uma requisição JSON-RPC.
    async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::debug!(method = %request.method, "Handling request");

        match request.method.as_str() {
            // Lifecycle
            "initialize" => self.handle_initialize(request),
            "initialized" => self.handle_initialized(request),
            "shutdown" => self.handle_shutdown(request),

            // Tools
            "tools/list" => self.handle_tools_list(request),
            "tools/call" => self.handle_tools_call(request).await,

            // Método desconhecido
            _ => {
                JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(&request.method))
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Handlers de lifecycle
    // ═══════════════════════════════════════════════════════════════════════

    /// Handler para initialize.
    fn handle_initialize(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::info!("Client initializing connection");

        let result = InitializeResult::default();

        self.initialized = true;

        JsonRpcResponse::success(
            request.id,
            serde_json::to_value(result).unwrap_or(json!({})),
        )
    }

    /// Handler para initialized (notificação).
    fn handle_initialized(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::info!("Client initialization complete");

        // initialized é uma notificação, não deve ter resposta
        // Mas retornamos uma resposta vazia caso tenha ID
        JsonRpcResponse::success(request.id, json!({}))
    }

    /// Handler para shutdown.
    fn handle_shutdown(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::info!("Client requested shutdown");

        self.initialized = false;

        JsonRpcResponse::success(request.id, json!(null))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Handlers de tools
    // ═══════════════════════════════════════════════════════════════════════

    /// Handler para tools/list.
    fn handle_tools_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tools = ToolHandler::list_tools();

        let result = ListToolsResult { tools };

        JsonRpcResponse::success(
            request.id,
            serde_json::to_value(result).unwrap_or(json!({"tools": []})),
        )
    }

    /// Handler para tools/call.
    async fn handle_tools_call(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params: CallToolParams = match request.params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        JsonRpcError::invalid_params(format!("Invalid params: {}", e)),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError::invalid_params("Missing params"),
                );
            }
        };

        tracing::info!(tool = %params.name, "Calling tool");

        let result = self
            .tools
            .handle_tool_call(&params.name, params.arguments)
            .await;

        // Converte ToolResult para Value
        let result_value = serde_json::to_value(&result).unwrap_or_else(|_| {
            json!({
                "content": [{"type": "text", "text": "Internal error"}],
                "isError": true
            })
        });

        JsonRpcResponse::success(request.id, result_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::protocol::JsonRpcId;
    use serde_json::Value;

    fn create_test_request(method: &str, params: Option<Value>) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(JsonRpcId::Number(1)),
            method: method.to_string(),
            params,
        }
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let config = Config::default();
        let mut server = McpServer::new(config).unwrap();

        let request = create_test_request("initialize", Some(json!({})));
        let response = server.handle_request(request).await;

        assert!(!response.is_error());
        assert!(server.initialized);

        let result = response.result.unwrap();
        assert!(result["protocolVersion"].is_string());
        assert!(result["serverInfo"]["name"].as_str() == Some("tetrad"));
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let config = Config::default();
        let mut server = McpServer::new(config).unwrap();

        let request = create_test_request("tools/list", None);
        let response = server.handle_request(request).await;

        assert!(!response.is_error());

        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 6);

        // Verifica que todos os tools esperados estão presentes
        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(tool_names.contains(&"tetrad_review_code"));
        assert!(tool_names.contains(&"tetrad_status"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_status() {
        let config = Config::default();
        let mut server = McpServer::new(config).unwrap();

        let request = create_test_request(
            "tools/call",
            Some(json!({
                "name": "tetrad_status",
                "arguments": {}
            })),
        );

        let response = server.handle_request(request).await;

        assert!(!response.is_error());
        // tetrad_status deve retornar informações sobre os executores
        let result = response.result.unwrap();
        // isError não é serializado quando é false (skip_serializing_if)
        // então se não existe, é considerado false (sucesso)
        assert!(!result["isError"].as_bool().unwrap_or(false));
    }

    #[tokio::test]
    async fn test_handle_tools_call_confirm() {
        let config = Config::default();
        let mut server = McpServer::new(config).unwrap();

        let request = create_test_request(
            "tools/call",
            Some(json!({
                "name": "tetrad_confirm",
                "arguments": {
                    "request_id": "test-123",
                    "agreed": true
                }
            })),
        );

        let response = server.handle_request(request).await;

        assert!(!response.is_error());
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let config = Config::default();
        let mut server = McpServer::new(config).unwrap();

        let request = create_test_request("unknown/method", None);
        let response = server.handle_request(request).await;

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, super::super::protocol::METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn test_handle_tools_call_invalid_params() {
        let config = Config::default();
        let mut server = McpServer::new(config).unwrap();

        // Params inválidos (falta 'name')
        let request = create_test_request(
            "tools/call",
            Some(json!({
                "arguments": {}
            })),
        );

        let response = server.handle_request(request).await;

        assert!(response.is_error());
        let error = response.error.unwrap();
        assert_eq!(error.code, super::super::protocol::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_handle_shutdown() {
        let config = Config::default();
        let mut server = McpServer::new(config).unwrap();

        // Initialize primeiro
        server.initialized = true;

        let request = create_test_request("shutdown", None);
        let response = server.handle_request(request).await;

        assert!(!response.is_error());
        assert!(!server.initialized);
    }
}
