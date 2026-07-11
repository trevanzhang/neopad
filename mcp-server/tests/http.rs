use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

struct McpProcess {
    child: Child,
    url: String,
    token: String,
}

impl McpProcess {
    fn start(workspace: &std::path::Path) -> Self {
        let token = "integration-secret".to_owned();
        let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
        let port = listener.local_addr().expect("local addr").port();
        drop(listener);

        let mut child = Command::new(env!("CARGO_BIN_EXE_neopad-mcp"))
            .arg("--workspace")
            .arg(workspace)
            .arg("serve")
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .env("NEOPAD_MCP_TOKEN", &token)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start neopad-mcp");

        let stderr = child.stderr.take().expect("stderr");
        thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                line.clear();
            }
        });

        let process = Self {
            child,
            url: format!("http://127.0.0.1:{port}/mcp"),
            token,
        };
        process.wait_until_ready();
        process
    }

    fn request(&self, request: Value) -> ureq::Response {
        ureq::post(&self.url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Content-Type", "application/json")
            .send_string(&request.to_string())
            .expect("MCP request")
    }

    fn json_request(&self, request: Value) -> Value {
        self.request(request).into_json().expect("json response")
    }

    fn wait_until_ready(&self) {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(5) {
            let result = ureq::post(&self.url)
                .set("Authorization", &format!("Bearer {}", self.token))
                .set("Content-Type", "application/json")
                .send_string(r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#);
            if result.is_ok() {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("MCP HTTP server did not become ready");
    }
}

impl Drop for McpProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn http_process_exposes_read_and_write_tools() {
    let temp = tempfile::tempdir().expect("temp dir");
    let process = McpProcess::start(&temp.path().join("workspace"));
    let response = process.json_request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    }));
    let tools = response["result"]["tools"].as_array().expect("tools");
    assert!(tools.iter().any(|tool| tool["name"] == "read_page"));
    assert!(tools.iter().any(|tool| tool["name"] == "create_page"));
    assert!(tools.iter().any(|tool| tool["name"] == "trash_page"));
    assert!(!tools.iter().any(|tool| tool["name"] == "delete_page"));
}

#[test]
fn http_process_persists_content_across_requests() {
    let temp = tempfile::tempdir().expect("temp dir");
    let process = McpProcess::start(&temp.path().join("workspace"));
    let created = process.json_request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "create_page",
            "arguments": { "title": "Integration", "content": "persisted" }
        }
    }));
    assert_eq!(created["result"]["isError"], false);

    let listed = process.json_request(json!({
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
fn http_process_rejects_missing_token() {
    let temp = tempfile::tempdir().expect("temp dir");
    let process = McpProcess::start(&temp.path().join("workspace"));
    let error = ureq::post(&process.url)
        .set("Content-Type", "application/json")
        .send_string(r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#)
        .expect_err("missing token should fail");

    match error {
        ureq::Error::Status(status, _) => assert_eq!(status, 401),
        other => panic!("unexpected error: {other}"),
    }
}
