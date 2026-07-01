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
- Optional Vim key bindings with an explicit mode indicator. Vim support stays
  an editor input mode rather than a plugin or Vim runtime system; its small
  set of options lives under Advanced settings, including whether NeoPad Ctrl
  shortcuts take priority over conflicting Vim mappings.
- Plain Markdown persistence under `~/.neopad/notes/*.md`.
- Multiple tabs backed by local metadata.
- Autosave.
- Edit, hybrid, and preview editor modes with a persisted default and shortcut.
- Page creation, rename, and trash actions with protected default pages.
- Persistent optional tab colors managed from the tab context menu.
- English and Chinese display names for system-managed page titles.
- Persistent day and night themes across the complete desktop interface.
- Native whole-window opacity controlled from an in-app slider.
- Compact settings that remain usable at the minimum supported window size.
- Full-text search.
- Native Save As for exporting the active note or a combined all-notes document
  to a user-selected Markdown file.
- Manual clipboard capture into `clipboard.md`.
- Tray menu.
- Single-instance startup and persistent main-window position, with a
  bottom-right default on first launch.
- Global shortcuts.
- Configurable global window and clipboard shortcuts, plus local `Alt+Enter`
  maximize/restore.
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
Keep the right side visually quiet so default WiX text remains readable. The
WiX banner must remain free of branding and text because native installer copy
is drawn over its left side.

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
