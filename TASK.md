# NeoPad Task Brief

This file is the clean project task brief for the current MVP. It replaces an
older corrupted-encoding draft.

## Product Direction

NeoPad is a lightweight, local-first desktop note pad for fast temporary notes.
It should feel like a quick capture utility rather than a full knowledge-base
application.

The public product name is `NeoPad`. Engineering identifiers stay lowercase:

- Repository: `neopad`
- Package names: `neopad-app`, `neopad-core`, `neopad-mcp`
- MCP server binary: `neopad-mcp`
- Default workspace: `~/.neopad`
- Bundle identifier: `com.neopad.desktop`

## MVP Scope

Implemented MVP capabilities:

- Tauri 2 desktop app.
- Vue 3 + TypeScript frontend.
- CodeMirror 6 Markdown editor.
- Plain Markdown persistence under `~/.neopad/notes/*.md`.
- Multiple tabs backed by local metadata.
- Autosave.
- Markdown preview and split mode.
- Full-text search.
- Manual clipboard capture into `clipboard.md`.
- Tray menu.
- Global shortcuts.
- Close-to-hide behavior.
- Standalone MCP server.
- Windows MSI packaging with branded installer assets.

## Data Rules

- User note bodies are stored only as Markdown files under `notes/`.
- Metadata belongs in `config.json` and `meta/tabs.json`.
- Delete operations move notes into `trash/`.
- Core filesystem access must go through `neopad-core`.
- Paths must be validated so callers cannot escape the configured workspace.
- Atomic writes must be preserved.
- MCP full-page updates must keep timestamp conflict protection.

## MCP Rules

- `neopad-mcp` communicates over stdio.
- stdout is reserved for JSON-RPC messages.
- stderr is used for diagnostics.
- Read-only tools are available by default.
- Write tools require explicit `--allow-write`.
- The MCP server must not read the system clipboard.
- The MCP server must not access files outside the workspace.

## Packaging Rules

The current Windows packaging target is MSI only.

Installer-related files:

```text
app/src-tauri/tauri.conf.json
app/src-tauri/wix/main.wxs
app/src-tauri/icons/icon.ico
app/src-tauri/icons/wix-banner.bmp
app/src-tauri/icons/wix-dialog.bmp
```

The WiX dialog bitmap is used as a full background on welcome and finish pages.
Keep the right side visually quiet so default WiX text remains readable.

## Verification

Baseline verification:

```powershell
cargo test
pnpm build
pnpm tauri:build
```

For MCP-specific changes:

```powershell
cargo build -p neopad-mcp --release
```

## Non-Goals for the MVP

- Cloud sync.
- Accounts.
- AI chat UI.
- RAG.
- Vector database.
- Backlinks.
- Graph view.
- Plugin system.
- Automatic clipboard history capture.
- Physical deletion of notes.
