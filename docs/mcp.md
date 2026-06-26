# MCP

`neopad-mcp` is a standalone Rust MCP server that exposes controlled access to
the local NeoPad workspace over stdio.

## Build

```powershell
cargo build -p neopad-mcp --release
```

The release binary is:

```text
target/release/neopad-mcp.exe
```

## Run

Read-only mode:

```powershell
target\release\neopad-mcp.exe --workspace ~/.neopad
```

Write-enabled mode:

```powershell
target\release\neopad-mcp.exe --workspace ~/.neopad --allow-write
```

If `--workspace` is omitted, the server uses the default workspace from
`neopad-core`.

## Client Configuration

Read-only example:

```json
{
  "mcpServers": {
    "neopad": {
      "command": "D:\\TrevanCode\\neopad\\target\\release\\neopad-mcp.exe",
      "args": ["--workspace", "~/.neopad"]
    }
  }
}
```

Write-enabled example:

```json
{
  "mcpServers": {
    "neopad": {
      "command": "D:\\TrevanCode\\neopad\\target\\release\\neopad-mcp.exe",
      "args": ["--workspace", "~/.neopad", "--allow-write"]
    }
  }
}
```

## Tools

Read-only tools are always exposed:

- `list_pages`: list note metadata and file sizes.
- `read_page`: read one page as Markdown.
- `search_pages`: search all Markdown pages.

Write tools are exposed only with `--allow-write`:

- `create_page`: create a new page, optionally with initial content.
- `append_to_page`: append Markdown to an existing page.
- `append_to_clipboard_page`: append text to `clipboard.md` with a timestamp
  separator.
- `update_page`: replace a page only when `expectedUpdatedAt` matches the
  current note timestamp.
- `delete_page`: move a page to trash.

## Safety Rules

- Read-only is the default.
- Write tools require explicit `--allow-write`.
- All data access goes through `neopad-core`.
- Note paths must stay inside the configured workspace.
- Delete means move to `trash/`, not physical deletion.
- stdout is reserved for MCP JSON-RPC messages.
- Diagnostics and startup errors go to stderr.
- The server does not read the system clipboard.
