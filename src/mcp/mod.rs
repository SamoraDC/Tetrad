//! Servidor MCP do Tetrad.
//!
//! Este módulo implementa o servidor MCP (Model Context Protocol) que permite
//! ao Claude Code usar as ferramentas de avaliação do Tetrad.
//!
//! ## Ferramentas Expostas
//!
//! - `tetrad_review_plan` - Revisa planos de implementação
//! - `tetrad_review_code` - Revisa código antes de salvar
//! - `tetrad_review_tests` - Revisa testes
//! - `tetrad_confirm` - Confirma acordo com feedback
//! - `tetrad_final_check` - Verificação final antes de commit
//! - `tetrad_status` - Status dos avaliadores
//!
//! ## Exemplo de Uso
//!
//! ```ignore
//! use tetrad::mcp::McpServer;
//! use tetrad::types::config::Config;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = Config::load_or_default();
//!     let mut server = McpServer::new(config).unwrap();
//!     server.run().await.unwrap();
//! }
//! ```

mod protocol;
mod server;
mod tools;
mod transport;

pub use protocol::{
    CallToolParams, InitializeResult, JsonRpcError, JsonRpcId, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse, ListToolsResult, ServerCapabilities, ServerInfo, ToolContent, ToolDescription,
    ToolResult, ToolsCapability, INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND,
    PARSE_ERROR,
};

pub use server::McpServer;
pub use tools::ToolHandler;
pub use transport::StdioTransport;
