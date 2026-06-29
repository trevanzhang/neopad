use anyhow::{bail, Context, Result};
use neopad_core::{
    append_to_clipboard_note, append_to_note, create_note, delete_note_to_trash, init_workspace,
    list_notes, lock_workspace_for_write, path::note_file_path, read_note, search_notes,
    write_note_atomic, write_note_atomic_checked, Workspace,
};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct Args {
    workspace: Option<PathBuf>,
    allow_write: bool,
}

#[derive(Debug)]
struct Server {
    workspace: Workspace,
    allow_write: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("neopad-mcp error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let workspace = init_workspace(args.workspace)?;
    let server = Server {
        workspace,
        allow_write: args.allow_write,
    };

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.context("failed to read stdin")?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => server.handle_request(request),
            Err(error) => jsonrpc_error(Value::Null, -32700, &format!("parse error: {error}")),
        };

        if !response.is_null() {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }

    Ok(())
}

impl Server {
    fn handle_request(&self, request: Value) -> Value {
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
            "tools/list" => jsonrpc_result(id, json!({ "tools": self.tools() })),
            "tools/call" => match self.call_tool(params) {
                Ok(output) => jsonrpc_result(
                    id,
                    json!({
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_owned())
                            }
                        ],
                        "isError": false
                    }),
                ),
                Err(error) => jsonrpc_result(
                    id,
                    json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("{error:#}")
                            }
                        ],
                        "isError": true
                    }),
                ),
            },
            _ => jsonrpc_error(id, -32601, "method not found"),
        }
    }

    fn tools(&self) -> Vec<Value> {
        let mut tools = vec![
            tool(
                "list_pages",
                "List NeoPad pages.",
                json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
            ),
            tool(
                "read_page",
                "Read a NeoPad page as Markdown.",
                json!({
                    "type": "object",
                    "properties": {
                        "pageId": { "type": "string" }
                    },
                    "required": ["pageId"],
                    "additionalProperties": false
                }),
            ),
            tool(
                "search_pages",
                "Search all NeoPad Markdown pages.",
                json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 100 }
                    },
                    "required": ["query"],
                    "additionalProperties": false
                }),
            ),
        ];

        if self.allow_write {
            tools.extend([
                tool(
                    "create_page",
                    "Create a NeoPad page.",
                    json!({
                        "type": "object",
                        "properties": {
                            "title": { "type": "string" },
                            "content": { "type": "string" }
                        },
                        "additionalProperties": false
                    }),
                ),
                tool(
                    "append_to_page",
                    "Append Markdown content to a page.",
                    json!({
                        "type": "object",
                        "properties": {
                            "pageId": { "type": "string" },
                            "content": { "type": "string" }
                        },
                        "required": ["pageId", "content"],
                        "additionalProperties": false
                    }),
                ),
                tool(
                    "append_to_clipboard_page",
                    "Append content to clipboard.md with a timestamp separator.",
                    json!({
                        "type": "object",
                        "properties": {
                            "content": { "type": "string" }
                        },
                        "required": ["content"],
                        "additionalProperties": false
                    }),
                ),
                tool(
                    "update_page",
                    "Replace a page if expectedUpdatedAt matches.",
                    json!({
                        "type": "object",
                        "properties": {
                            "pageId": { "type": "string" },
                            "content": { "type": "string" },
                            "expectedUpdatedAt": { "type": "integer" }
                        },
                        "required": ["pageId", "content", "expectedUpdatedAt"],
                        "additionalProperties": false
                    }),
                ),
                tool(
                    "delete_page",
                    "Move a page to trash.",
                    json!({
                        "type": "object",
                        "properties": {
                            "pageId": { "type": "string" }
                        },
                        "required": ["pageId"],
                        "additionalProperties": false
                    }),
                ),
            ]);
        }

        tools
    }

    fn call_tool(&self, params: Value) -> Result<Value> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .context("missing tool name")?;
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));

        match name {
            "list_pages" => self.list_pages(),
            "read_page" => self.read_page(required_string(&arguments, "pageId")?),
            "search_pages" => self.search_pages(
                required_string(&arguments, "query")?,
                optional_usize(&arguments, "limit").unwrap_or(20),
            ),
            "create_page" => self.require_write(|| {
                let title = optional_string(&arguments, "title");
                let content = optional_string(&arguments, "content").unwrap_or_default();
                let note = create_note(&self.workspace, title)?;
                if content.is_empty() {
                    Ok(json!(note))
                } else {
                    Ok(json!(write_note_atomic(
                        &self.workspace,
                        &note.id,
                        &content
                    )?))
                }
            }),
            "append_to_page" => self.require_write(|| {
                Ok(json!(append_to_note(
                    &self.workspace,
                    &required_string(&arguments, "pageId")?,
                    &required_string(&arguments, "content")?
                )?))
            }),
            "append_to_clipboard_page" => self.require_write(|| {
                Ok(json!(append_to_clipboard_note(
                    &self.workspace,
                    &required_string(&arguments, "content")?
                )?))
            }),
            "update_page" => self.require_write(|| {
                Ok(json!(write_note_atomic_checked(
                    &self.workspace,
                    &required_string(&arguments, "pageId")?,
                    &required_string(&arguments, "content")?,
                    required_i64(&arguments, "expectedUpdatedAt")?
                )?))
            }),
            "delete_page" => self.require_write(|| {
                Ok(json!(delete_note_to_trash(
                    &self.workspace,
                    &required_string(&arguments, "pageId")?
                )?))
            }),
            _ => bail!("unknown tool: {name}"),
        }
    }

    fn list_pages(&self) -> Result<Value> {
        let pages = list_notes(&self.workspace)?
            .into_iter()
            .map(|tab| {
                let size = note_file_path(&self.workspace, &tab.file_name)
                    .ok()
                    .and_then(|path| path.metadata().ok())
                    .map(|metadata| metadata.len())
                    .unwrap_or(0);
                json!({
                    "id": tab.id,
                    "title": tab.title,
                    "fileName": tab.file_name,
                    "updatedAt": tab.updated_at,
                    "size": size
                })
            })
            .collect::<Vec<_>>();

        Ok(json!({ "pages": pages }))
    }

    fn read_page(&self, page_id: String) -> Result<Value> {
        let note = read_note(&self.workspace, &page_id)?;
        Ok(json!({
            "id": note.id,
            "title": note.title,
            "fileName": note.file_name,
            "content": note.content,
            "updatedAt": note.updated_at
        }))
    }

    fn search_pages(&self, query: String, limit: usize) -> Result<Value> {
        let results = search_notes(&self.workspace, &query, limit)?;
        Ok(json!({ "results": results }))
    }

    fn require_write<F>(&self, operation: F) -> Result<Value>
    where
        F: FnOnce() -> Result<Value>,
    {
        if !self.allow_write {
            bail!("write tools require --allow-write");
        }
        let _lock = lock_workspace_for_write(&self.workspace)?;
        operation()
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args> {
    let mut workspace = None;
    let mut allow_write = false;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => {
                let value = args.next().context("--workspace requires a path")?;
                workspace = Some(expand_home_path(&value));
            }
            "--allow-write" => {
                allow_write = true;
            }
            "--help" | "-h" => {
                eprintln!("Usage: neopad-mcp [--workspace <path>] [--allow-write]");
                std::process::exit(0);
            }
            _ => bail!("unknown argument: {arg}"),
        }
    }

    Ok(Args {
        workspace,
        allow_write,
    })
}

fn expand_home_path(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
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

fn required_string(arguments: &Value, key: &str) -> Result<String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .with_context(|| format!("missing string argument: {key}"))
}

fn optional_string(arguments: &Value, key: &str) -> Option<String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn optional_usize(arguments: &Value, key: &str) -> Option<usize> {
    arguments
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
}

fn required_i64(arguments: &Value, key: &str) -> Result<i64> {
    arguments
        .get(key)
        .and_then(Value::as_i64)
        .with_context(|| format!("missing integer argument: {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(allow_write: bool) -> (tempfile::TempDir, Server) {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        (
            temp,
            Server {
                workspace,
                allow_write,
            },
        )
    }

    #[test]
    fn tools_list_hides_write_tools_by_default() {
        let (_temp, server) = server(false);
        let response = server.handle_request(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }));
        let tools = response["result"]["tools"].as_array().expect("tools");
        assert!(tools.iter().any(|tool| tool["name"] == "read_page"));
        assert!(!tools.iter().any(|tool| tool["name"] == "update_page"));
    }

    #[test]
    fn write_enabled_tool_call_round_trips_page_content() {
        let (_temp, server) = server(true);
        let created = server.handle_request(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "create_page",
                "arguments": { "title": "MCP", "content": "hello" }
            }
        }));
        assert_eq!(created["result"]["isError"], false);

        let pages = server.list_pages().expect("list pages");
        let page = pages["pages"]
            .as_array()
            .expect("pages")
            .iter()
            .find(|page| page["title"] == "MCP")
            .expect("created page");
        let read = server
            .read_page(page["id"].as_str().expect("page id").to_owned())
            .expect("read page");
        assert_eq!(read["content"], "hello");
    }
}
