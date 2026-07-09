use crate::tools::{tool_result, Tools};
use neopad_core::Workspace;
use serde_json::{json, Value};

#[derive(Debug)]
pub struct Server {
    tools: Tools,
}

impl Server {
    pub fn new(workspace: Workspace) -> Self {
        Self {
            tools: Tools::new(workspace),
        }
    }

    pub fn handle_request(&self, request: Value) -> Value {
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or_else(|| json!({}));

        match method {
            "notifications/initialized" => Value::Null,
            "initialize" => jsonrpc_result(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "neopad-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            ),
            "tools/list" => jsonrpc_result(id, json!({ "tools": self.tools.list() })),
            "tools/call" => jsonrpc_result(id, tool_result(self.tools.call(params))),
            _ => jsonrpc_error(id, -32601, "method not found"),
        }
    }
}

fn jsonrpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn jsonrpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use neopad_core::init_workspace;

    fn server() -> (tempfile::TempDir, Server) {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        (temp, Server::new(workspace))
    }

    #[test]
    fn tools_list_exposes_read_and_write_tools() {
        let (_temp, server) = server();
        let response = server.handle_request(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }));
        let tools = response["result"]["tools"].as_array().expect("tools");
        assert!(tools.iter().any(|tool| tool["name"] == "read_page"));
        assert!(tools.iter().any(|tool| tool["name"] == "update_page"));
        assert!(tools.iter().any(|tool| tool["name"] == "trash_page"));
    }
}
