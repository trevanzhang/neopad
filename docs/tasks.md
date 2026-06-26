# Tasks

This file tracks the implemented MVP relative to the original task plan.

## Completed

- Project initialization with a Tauri 2 + Vue 3 + TypeScript app and Rust
  workspace.
- `neopad-core` workspace initialization for `~/.neopad`.
- Default notes and metadata:
  - `notes/inbox.md`
  - `notes/clipboard.md`
  - `meta/tabs.json`
  - `config.json`
- Note CRUD and trash move behavior.
- Atomic writes and mtime-checked writes.
- Tauri command bridge.
- Vue app shell with tabs, editor, preview, search, settings, and status bar.
- CodeMirror 6 Markdown editor.
- Autosave from the UI.
- Tray menu and close-to-hide behavior.
- Global shortcuts for toggle window and clipboard capture.
- Full-text search.
- Manual clipboard capture to `clipboard.md`.
- Standalone `neopad-mcp` read and write tools.
- Windows icon, tray/taskbar icon, MSI branding, desktop shortcut icon, and
  hidden release console window.

## Current Build Targets

- Windows MSI package.
- Standalone `neopad-mcp.exe`.

## Recommended Next Tasks

1. Replace the generated or broken-encoding `TASK.md` with a clean UTF-8 product
   specification.
2. Add integration tests for MCP JSON-RPC tool calls.
3. Add frontend tests or smoke tests for autosave, tab switching, and search.
4. Review the custom WiX template for portability before moving the repository
   path or building on CI.
5. Decide whether `neopad-mcp` should be bundled as a sidecar or distributed as
   a separate binary.
6. Add release notes and versioning policy before publishing installers.

## Non-Goals for the MVP

- Cloud sync.
- Accounts.
- AI chat UI.
- RAG or vector database.
- Backlinks or graph view.
- Plugin system.
- Automatic clipboard history capture.
- Physical deletion of user notes.
