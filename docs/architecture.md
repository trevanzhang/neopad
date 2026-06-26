# Architecture

NeoPad is a local-first desktop notes application split into three Rust
workspace members plus a Vue frontend.

## Components

```text
neopad-app
  app/src/              Vue 3 frontend
  app/src-tauri/        Tauri 2 desktop shell

neopad-core
  crates/neopad-core/   Shared local data model and filesystem logic

neopad-mcp
  mcp-server/           Standalone MCP stdio server
```

Both the desktop app and MCP server use `neopad-core` for workspace access. Do
not duplicate note, search, path, or write logic outside the core crate.

## Data Flow

```text
Vue UI
  -> Tauri command
  -> neopad-core
  -> ~/.neopad/notes/*.md

MCP client
  -> neopad-mcp over stdio
  -> neopad-core
  -> ~/.neopad/notes/*.md
```

## Local Workspace

The default workspace is `~/.neopad/`.

```text
~/.neopad/
  notes/
    inbox.md
    clipboard.md
    page-*.md
  meta/
    tabs.json
  config.json
  trash/
  backups/
```

The app presents notes as tabs. The durable source of note content is always a
Markdown file in `notes/`. `tabs.json` stores tab metadata such as title, file
name, timestamps, pinned state, and active tab.

## Core Responsibilities

`neopad-core` owns:

- Workspace path resolution and first-run initialization.
- Safe note path construction.
- Note list, read, create, write, rename, append, and trash operations.
- Atomic writes through temporary files and replace/rename behavior.
- mtime-checked writes for MCP update conflict protection.
- Full-text search over Markdown note files.
- Config and tab metadata defaults.

Path safety is mandatory. Public operations must not accept paths that escape
the configured workspace.

## Desktop App Responsibilities

`neopad-app` owns:

- Tauri command bindings to core operations.
- Vue app shell and note editing workflow.
- CodeMirror markdown editor.
- Markdown preview and split mode.
- Autosave and status reporting.
- Tray menu: show, hide, new note, save clipboard, settings, quit.
- Global shortcuts:
  - `Alt+Z`: toggle window.
  - `Ctrl+Shift+V`: append current text clipboard to `clipboard.md`.
  - `Escape`: hide window when focused.
- Close-to-hide behavior.
- Windows runtime icon and release GUI subsystem.

## MCP Responsibilities

`neopad-mcp` owns the MCP protocol surface only. It must keep stdout reserved
for JSON-RPC responses and write diagnostics to stderr.

The server is read-only by default. Write tools are available only when started
with `--allow-write`.

## Packaging

The Windows target currently builds an MSI package only. The Tauri config points
to:

```text
app/src-tauri/wix/main.wxs
```

This custom WiX template exists so the installer can control:

- Branded banner and dialog bitmaps.
- Product icon.
- Start-menu shortcut icon.
- Desktop shortcut icon.
- AppUserModel.ID on shortcuts.

Installer bitmap assets live in:

```text
app/src-tauri/icons/wix-banner.bmp
app/src-tauri/icons/wix-dialog.bmp
```
