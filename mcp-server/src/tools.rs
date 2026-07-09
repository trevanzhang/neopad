use anyhow::{bail, Context, Result};
use neopad_core::{
    append_to_clipboard_note, append_to_note, create_note, delete_note_to_trash, list_notes,
    lock_workspace_for_write, path::note_file_path, read_note, search_notes, write_note_atomic,
    write_note_atomic_checked, Workspace,
};
use serde_json::{json, Value};

#[derive(Debug)]
pub struct Tools {
    workspace: Workspace,
}

impl Tools {
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }

    pub fn list(&self) -> Vec<Value> {
        vec![
            tool(
                "append_to_inbox",
                "Append Markdown content to the pinned Inbox page.",
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
                "append_to_clipboard_page",
                "Append provided content to clipboard.md with a timestamp separator.",
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
            tool(
                "trash_page",
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
        ]
    }

    pub fn call(&self, params: Value) -> Result<Value> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .context("missing tool name")?;
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));

        match name {
            "append_to_inbox" => self.write(|| {
                Ok(json!(append_to_note(
                    &self.workspace,
                    "inbox",
                    &required_string(&arguments, "content")?
                )?))
            }),
            "append_to_clipboard_page" => self.write(|| {
                Ok(json!(append_to_clipboard_note(
                    &self.workspace,
                    &required_string(&arguments, "content")?
                )?))
            }),
            "create_page" => self.write(|| {
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
            "append_to_page" => self.write(|| {
                Ok(json!(append_to_note(
                    &self.workspace,
                    &required_string(&arguments, "pageId")?,
                    &required_string(&arguments, "content")?
                )?))
            }),
            "update_page" => self.write(|| {
                Ok(json!(write_note_atomic_checked(
                    &self.workspace,
                    &required_string(&arguments, "pageId")?,
                    &required_string(&arguments, "content")?,
                    required_i64(&arguments, "expectedUpdatedAt")?
                )?))
            }),
            "list_pages" => self.list_pages(),
            "read_page" => self.read_page(required_string(&arguments, "pageId")?),
            "search_pages" => self.search_pages(
                required_string(&arguments, "query")?,
                optional_usize(&arguments, "limit").unwrap_or(20),
            ),
            "trash_page" => self.write(|| {
                Ok(json!(delete_note_to_trash(
                    &self.workspace,
                    &required_string(&arguments, "pageId")?
                )?))
            }),
            "delete_page" => bail!("delete_page has been replaced by trash_page"),
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

    fn write<F>(&self, operation: F) -> Result<Value>
    where
        F: FnOnce() -> Result<Value>,
    {
        let _lock = lock_workspace_for_write(&self.workspace)?;
        operation()
    }
}

pub fn tool_result(output: Result<Value>) -> Value {
    match output {
        Ok(output) => json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_owned())
                }
            ],
            "isError": false
        }),
        Err(error) => json!({
            "content": [
                {
                    "type": "text",
                    "text": format!("{error:#}")
                }
            ],
            "isError": true
        }),
    }
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
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
    use neopad_core::init_workspace;

    fn tools() -> (tempfile::TempDir, Tools) {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        (temp, Tools::new(workspace))
    }

    #[test]
    fn append_to_inbox_persists_content() {
        let (_temp, tools) = tools();
        tools
            .call(json!({
                "name": "append_to_inbox",
                "arguments": { "content": "captured" }
            }))
            .expect("append");
        let inbox = tools
            .call(json!({
                "name": "read_page",
                "arguments": { "pageId": "inbox" }
            }))
            .expect("read");
        assert!(inbox["content"]
            .as_str()
            .expect("content")
            .contains("captured"));
    }

    #[test]
    fn update_page_rejects_stale_timestamp() {
        let (_temp, tools) = tools();
        let created = tools
            .call(json!({
                "name": "create_page",
                "arguments": { "title": "Checked", "content": "one" }
            }))
            .expect("create");
        let error = tools
            .call(json!({
                "name": "update_page",
                "arguments": {
                    "pageId": created["id"].as_str().expect("id"),
                    "content": "two",
                    "expectedUpdatedAt": created["updatedAt"].as_i64().expect("updated") - 1
                }
            }))
            .expect_err("stale write should fail");
        assert!(format!("{error:#}").contains("note was modified"));
    }

    #[test]
    fn trash_page_moves_note_out_of_page_list() {
        let (_temp, tools) = tools();
        let created = tools
            .call(json!({
                "name": "create_page",
                "arguments": { "title": "Trash me" }
            }))
            .expect("create");
        tools
            .call(json!({
                "name": "trash_page",
                "arguments": { "pageId": created["id"].as_str().expect("id") }
            }))
            .expect("trash");
        let listed = tools.call(json!({ "name": "list_pages" })).expect("list");
        assert!(listed["pages"]
            .as_array()
            .expect("pages")
            .iter()
            .all(|page| page["id"] != created["id"]));
    }
}
