use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct McpProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl McpProcess {
    fn start(workspace: &std::path::Path, allow_write: bool) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_neopad-mcp"));
        command
            .arg("--workspace")
            .arg(workspace)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if allow_write {
            command.arg("--allow-write");
        }
        let mut child = command.spawn().expect("start neopad-mcp");
        let stdin = child.stdin.take().expect("child stdin");
        let stdout = BufReader::new(child.stdout.take().expect("child stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn request(&mut self, request: Value) -> Value {
        writeln!(self.stdin, "{request}").expect("write request");
        self.stdin.flush().expect("flush request");
        let mut line = String::new();
        self.stdout.read_line(&mut line).expect("read response");
        serde_json::from_str(&line).expect("stdout must contain one JSON-RPC response per line")
    }
}

impl Drop for McpProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn read_only_process_exposes_only_read_tools() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut process = McpProcess::start(&temp.path().join("workspace"), false);
    let response = process.request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    }));
    let tools = response["result"]["tools"].as_array().expect("tools");
    assert!(tools.iter().any(|tool| tool["name"] == "read_page"));
    assert!(!tools.iter().any(|tool| tool["name"] == "create_page"));
}

#[test]
fn write_enabled_process_persists_content_across_requests() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut process = McpProcess::start(&temp.path().join("workspace"), true);
    let created = process.request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "create_page",
            "arguments": { "title": "Integration", "content": "persisted" }
        }
    }));
    assert_eq!(created["result"]["isError"], false);

    let listed = process.request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": { "name": "list_pages", "arguments": {} }
    }));
    let text = listed["result"]["content"][0]["text"]
        .as_str()
        .expect("tool text");
    let output: Value = serde_json::from_str(text).expect("tool JSON");
    assert!(output["pages"]
        .as_array()
        .expect("pages")
        .iter()
        .any(|page| page["title"] == "Integration"));
}

#[test]
fn malformed_input_returns_json_rpc_parse_error() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut process = McpProcess::start(&temp.path().join("workspace"), false);
    writeln!(process.stdin, "not-json").expect("write malformed request");
    process.stdin.flush().expect("flush malformed request");
    let mut line = String::new();
    process.stdout.read_line(&mut line).expect("read response");
    let response: Value = serde_json::from_str(&line).expect("JSON-RPC response");
    assert_eq!(response["error"]["code"], -32700);
}
