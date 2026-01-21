//! Testes de integração para o protocolo MCP do Tetrad.

use serde_json::{json, Value};

/// Helper para criar uma mensagem JSON-RPC.
fn jsonrpc_request(id: u64, method: &str, params: Option<Value>) -> String {
    let mut req = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
    });
    if let Some(p) = params {
        req["params"] = p;
    }
    serde_json::to_string(&req).unwrap()
}

/// Helper para criar uma mensagem MCP com Content-Length header.
fn mcp_message(content: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", content.len(), content)
}

#[test]
fn test_jsonrpc_request_format() {
    let req = jsonrpc_request(1, "initialize", None);
    let parsed: Value = serde_json::from_str(&req).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 1);
    assert_eq!(parsed["method"], "initialize");
}

#[test]
fn test_mcp_message_format() {
    let content = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
    let msg = mcp_message(content);

    assert!(msg.starts_with("Content-Length: "));
    assert!(msg.contains("\r\n\r\n"));
    assert!(msg.ends_with(content));
}

#[test]
fn test_jsonrpc_with_params() {
    let params = json!({
        "code": "fn main() {}",
        "language": "rust"
    });
    let req = jsonrpc_request(42, "tools/call", Some(params.clone()));
    let parsed: Value = serde_json::from_str(&req).unwrap();

    assert_eq!(parsed["id"], 42);
    assert_eq!(parsed["method"], "tools/call");
    assert_eq!(parsed["params"]["code"], "fn main() {}");
    assert_eq!(parsed["params"]["language"], "rust");
}

// Testes do protocolo MCP
mod protocol_tests {
    use tetrad::mcp::{
        JsonRpcId, JsonRpcRequest, JsonRpcResponse, JsonRpcError,
        ToolDescription, ToolResult,
        PARSE_ERROR, INVALID_REQUEST, METHOD_NOT_FOUND, INVALID_PARAMS, INTERNAL_ERROR,
    };
    use serde_json::json;

    #[test]
    fn test_json_rpc_id_number() {
        let id = JsonRpcId::Number(42);
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "42");

        let deserialized: JsonRpcId = serde_json::from_str("42").unwrap();
        assert!(matches!(deserialized, JsonRpcId::Number(42)));
    }

    #[test]
    fn test_json_rpc_id_string() {
        let id = JsonRpcId::String("test-id".to_string());
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "\"test-id\"");

        let deserialized: JsonRpcId = serde_json::from_str("\"test-id\"").unwrap();
        assert!(matches!(deserialized, JsonRpcId::String(s) if s == "test-id"));
    }

    #[test]
    fn test_json_rpc_request_parsing() {
        let json_str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert!(matches!(request.id, Some(JsonRpcId::Number(1))));
        assert_eq!(request.method, "initialize");
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(
            Some(JsonRpcId::Number(1)),
            json!({"status": "ok"})
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::error(
            Some(JsonRpcId::Number(1)),
            JsonRpcError::method_not_found("unknown_method")
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_some());
        assert!(response.result.is_none());

        let error = response.error.unwrap();
        assert_eq!(error.code, METHOD_NOT_FOUND);
    }

    #[test]
    fn test_tool_description() {
        let tool = ToolDescription {
            name: "tetrad_review_code".to_string(),
            description: "Review code before saving".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "code": { "type": "string" },
                    "language": { "type": "string" }
                },
                "required": ["code", "language"]
            }),
        };

        assert_eq!(tool.name, "tetrad_review_code");
        assert!(tool.input_schema["properties"]["code"]["type"] == "string");
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("Hello, World!");
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_tool_result_json() {
        let data = json!({"score": 85, "passed": true});
        let result = ToolResult::success_json(&data);
        assert!(!result.is_error);
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Something went wrong");
        assert!(result.is_error);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(PARSE_ERROR, -32700);
        assert_eq!(INVALID_REQUEST, -32600);
        assert_eq!(METHOD_NOT_FOUND, -32601);
        assert_eq!(INVALID_PARAMS, -32602);
        assert_eq!(INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_json_rpc_error_constructors() {
        let parse_err = JsonRpcError::parse_error();
        assert_eq!(parse_err.code, PARSE_ERROR);

        let invalid_req = JsonRpcError::invalid_request();
        assert_eq!(invalid_req.code, INVALID_REQUEST);

        let not_found = JsonRpcError::method_not_found("unknown");
        assert_eq!(not_found.code, METHOD_NOT_FOUND);

        let invalid_params = JsonRpcError::invalid_params("bad param");
        assert_eq!(invalid_params.code, INVALID_PARAMS);

        let internal = JsonRpcError::internal_error("unexpected");
        assert_eq!(internal.code, INTERNAL_ERROR);
    }
}

// Testes do cache
mod cache_tests {
    use tetrad::cache::EvaluationCache;
    use tetrad::types::responses::EvaluationResult;
    use tetrad::types::requests::EvaluationType;
    use std::time::Duration;

    fn sample_result() -> EvaluationResult {
        EvaluationResult::success("test-123", 85, "Looks good!")
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(300));
        let result = sample_result();

        cache.insert_by_code("fn main() {}", "rust", &EvaluationType::Code, result.clone());

        let cached = cache.get_by_code("fn main() {}", "rust", &EvaluationType::Code);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().score, 85);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(300));

        let cached = cache.get_by_code("fn main() {}", "rust", &EvaluationType::Code);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_different_keys() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(300));
        let result = sample_result();

        cache.insert_by_code("fn main() {}", "rust", &EvaluationType::Code, result.clone());

        // Código diferente
        let cached = cache.get_by_code("fn main() { println!(); }", "rust", &EvaluationType::Code);
        assert!(cached.is_none());

        // Linguagem diferente
        let cached = cache.get_by_code("fn main() {}", "python", &EvaluationType::Code);
        assert!(cached.is_none());

        // Tipo diferente
        let cached = cache.get_by_code("fn main() {}", "rust", &EvaluationType::Tests);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = EvaluationCache::new(2, Duration::from_secs(300));
        let result = sample_result();

        // Insere 3 itens em cache de capacidade 2
        cache.insert_by_code("code1", "rust", &EvaluationType::Code, result.clone());
        cache.insert_by_code("code2", "rust", &EvaluationType::Code, result.clone());
        cache.insert_by_code("code3", "rust", &EvaluationType::Code, result.clone());

        // O primeiro deve ter sido evictado
        let cached1 = cache.get_by_code("code1", "rust", &EvaluationType::Code);
        assert!(cached1.is_none());

        // Verifica que code3 está presente
        let cached3 = cache.get_by_code("code3", "rust", &EvaluationType::Code);
        assert!(cached3.is_some());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = EvaluationCache::new(10, Duration::from_secs(300));
        let result = sample_result();

        cache.insert_by_code("code1", "rust", &EvaluationType::Code, result.clone());
        cache.insert_by_code("code2", "rust", &EvaluationType::Code, result.clone());

        cache.clear();

        assert!(cache.get_by_code("code1", "rust", &EvaluationType::Code).is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = EvaluationCache::new(100, Duration::from_secs(300));
        let stats = cache.stats();

        assert_eq!(stats.capacity, 100);
        assert_eq!(stats.size, 0);
    }
}

// Testes do sistema de hooks
mod hooks_tests {
    use tetrad::hooks::{Hook, HookEvent, HookResult, HookContext, HookSystem, LoggingHook};
    use tetrad::types::requests::EvaluationRequest;
    use tetrad::types::responses::EvaluationResult;

    fn sample_request() -> EvaluationRequest {
        EvaluationRequest::new("fn main() {}", "rust")
    }

    fn sample_result() -> EvaluationResult {
        EvaluationResult::success("test-123", 85, "Looks good!")
    }

    #[test]
    fn test_logging_hook_name() {
        let hook = LoggingHook;
        assert_eq!(hook.name(), "logging");
    }

    #[test]
    fn test_logging_hook_event() {
        let hook = LoggingHook;
        assert!(matches!(hook.event(), HookEvent::PostEvaluate));
    }

    #[tokio::test]
    async fn test_logging_hook_execute() {
        let hook = LoggingHook;
        let request = sample_request();
        let result = sample_result();

        let context = HookContext::PostEvaluate {
            request: &request,
            result: &result,
        };

        let hook_result = hook.execute(&context).await.unwrap();
        assert!(matches!(hook_result, HookResult::Continue));
    }

    #[tokio::test]
    async fn test_hook_system_registration() {
        let mut system = HookSystem::new();
        system.register(Box::new(LoggingHook));

        let request = sample_request();
        let result = sample_result();

        system.run_post_evaluate(&request, &result).await.unwrap();
    }

    #[tokio::test]
    async fn test_hook_system_pre_evaluate() {
        let system = HookSystem::new();
        let request = sample_request();

        let result = system.run_pre_evaluate(&request).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
    }

    #[test]
    fn test_hook_result_variants() {
        let _continue = HookResult::Continue;
        let _skip = HookResult::Skip;
        let _modify = HookResult::ModifyRequest(sample_request());
    }

    #[test]
    fn test_hook_context_variants() {
        let request = sample_request();
        let result = sample_result();

        let _pre = HookContext::PreEvaluate { request: &request };
        let _post = HookContext::PostEvaluate { request: &request, result: &result };
        let _consensus = HookContext::OnConsensus { result: &result };
        let _block = HookContext::OnBlock { result: &result };
    }
}
