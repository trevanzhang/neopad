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
    reminders.json
  config.json
  trash/
  backups/
```

The app presents notes as tabs. The durable source of note content is always a
Markdown file in `notes/`. `tabs.json` stores tab metadata such as title, file
name, timestamps, pinned state, active tab, and optional tab color.
`reminders.json` stores only notification delivery signatures so reminder text
continues to live in ordinary Markdown note lines.

Desktop preferences are persisted in `config.json`. Browser `localStorage`
supports frontend startup, while native `config.json` is authoritative in the
desktop runtime. The experimental project does not maintain migrations for old
configuration shapes.

## Core Responsibilities

`neopad-core` owns:

- Workspace path resolution and first-run initialization.
- Safe note path construction.
- Note list, read, create, write, rename, append, and trash operations.
- Atomic writes through temporary files and replace/rename behavior.
- mtime-checked writes for MCP update conflict protection.
- Full-text search over Markdown note files.
- Reminder parsing, ordering, and notification delivery-state persistence.
- Config and tab metadata defaults.

Path safety is mandatory. Public operations must not accept paths that escape
the configured workspace.

## Desktop App Responsibilities

`neopad-app` owns:

- Tauri command bindings to core operations.
- Vue app shell and note editing workflow.
- CodeMirror markdown editor with optional Vim key bindings, configurable
  Insert-mode exit sequence, visible mode status, and an option to preserve
  NeoPad Ctrl shortcuts when Vim mappings conflict.
- Markdown preview and split mode.
- Persistent light and dark themes.
- Native Windows whole-window opacity controlled through a Tauri command and
  `SetLayeredWindowAttributes`; CSS opacity is not used for the app shell.
- Tab context actions for rename, trash, and persistent color selection.
- Compact settings dialog with independently scrollable content and a dedicated
  Advanced tab for optional editor features.
- Autosave and status reporting.
- Compact reminder creation and list surfaces. Reminder lines use
  `- [ ] @提醒 YYYY-MM-DD HH:mm content`; checking the Markdown task completes
  the reminder.
- `F5` toggles the reminder list. Single and bulk overdue completion operations
  atomically change `[ ]` to `[x]` while preserving the reminder lines.
- Native reminder notifications while the desktop process remains running.
- Tray menu: show, hide, new note, save clipboard, settings, quit.
- Global shortcuts:
  - `Alt+Z`: toggle window.
  - `Ctrl+Shift+V`: append current text clipboard to `clipboard.md`.
  - `Alt+Enter`: maximize or restore the focused main window.
  - `Escape`: leave Vim Insert or Visual mode first; hide the window from Vim
    Normal mode or the regular editor.

The two global shortcuts are configurable and re-registered at runtime. NeoPad
rejects duplicate window and clipboard shortcut combinations.
- Close-to-hide behavior.
- Single-instance enforcement and persisted main-window position; the initial
  position is the active screen's bottom-right work area.
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
