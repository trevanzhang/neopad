# MCP

`neopad-mcp` is a standalone Rust MCP server that exposes controlled local HTTP
access to the NeoPad workspace.

The desktop app can start and stop the service from Settings. The server is not
embedded in the desktop process.

## Desktop Usage

Open `Settings -> MCP` to manage the local service.

The page shows:

- A start or stop button.
- Current service status.
- Local endpoint URL.
- Bearer token.
- Buttons to copy client configuration or regenerate the token.
- A short client installation example.

The service is off by default. If enabled, NeoPad starts the sidecar process on
app launch, keeps it running while the app is hidden to tray, and stops it on
full app exit. On Windows the sidecar is started without opening a console
window.

## Build

```powershell
cargo build -p neopad-mcp --release
```

The release binary is:

```text
target/release/neopad-mcp.exe
```

## Run

```powershell
target\release\neopad-mcp.exe --workspace ~/.neopad serve --host 127.0.0.1 --port 8765 --token <local-token>
```

If `--workspace` is omitted, the server uses the default workspace from
`neopad-core`.

The default endpoint is:

```text
http://127.0.0.1:8765/mcp
```

## Client Configuration

Use the MCP settings page to copy the current URL and bearer token:

```json
{
  "mcpServers": {
    "neopad": {
      "url": "http://127.0.0.1:8765/mcp",
      "headers": {
        "Authorization": "Bearer <local-token>"
      }
    }
  }
}
```

## Tools

The HTTP service is read-write when enabled:

- `append_to_inbox`: append Markdown to the pinned Inbox page.
- `append_to_clipboard_page`: append provided text to `clipboard.md` with a
  timestamp separator.
- `create_page`: create a new page, optionally with initial content.
- `append_to_page`: append Markdown to an existing page.
- `update_page`: replace a page only when `expectedUpdatedAt` matches the
  current note timestamp.
- `list_pages`: list note metadata and file sizes.
- `read_page`: read one page as Markdown.
- `search_pages`: search all Markdown pages.
- `trash_page`: move a page to trash.

`delete_page` is intentionally not supported. Use `trash_page` so user notes are
moved to NeoPad's trash directory instead of being physically deleted.

## Safety Rules

- The service is off by default.
- The default bind address is `127.0.0.1`.
- Non-loopback bind addresses are rejected.
- HTTP requests require `Authorization: Bearer <local-token>`.
- Browser-originated requests must use a local origin.
- CLI and agent requests without an `Origin` header are allowed when the bearer
  token is valid.
- All data access goes through `neopad-core`.
- Note paths must stay inside the configured workspace.
- Trash means move to `trash/`, not physical deletion.
- Diagnostics and startup errors go to stderr.
- The server does not read the system clipboard.

## Packaging

The desktop MSI includes `neopad-mcp.exe` as an external sidecar binary. The
root build command prepares the sidecar name expected by Tauri:

```powershell
pnpm tauri:build
```

Manual packaging steps, if needed:

```powershell
cargo build -p neopad-mcp --release
node scripts/prepare-mcp-sidecar.mjs
node scripts/build-tauri-bundle.mjs
```

Do not reintroduce stdio mode for the desktop-managed server. Local agents
should connect to the Streamable HTTP endpoint at `/mcp`.
