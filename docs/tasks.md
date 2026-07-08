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
- Cross-process write locking for desktop and write-enabled MCP operations.
- Tauri command bridge.
- Vue app shell with tabs, editor, preview, search, settings, and status bar.
- Edit, hybrid, and preview modes with a compact View menu, status indicator,
  configurable cycling shortcut, and persisted default.
- Page rename and trash actions with UI and core protection for default pages.
- Tab context menu with rename, trash, and persistent color selection.
- Localized system page labels that preserve stable storage IDs and user titles.
- CodeMirror 6 Markdown editor.
- Optional persisted Vim key bindings with Normal, Insert, and Visual status,
  plus a configurable Insert exit sequence in Advanced settings.
- Persistent light and dark themes with a destination-state status-bar toggle.
- Viewport-constrained settings dialog with fixed navigation and actions.
- Autosave from the UI.
- Serialized autosave and tray/global-command handling that preserves pending
  edits, with deterministic frontend tests.
- Tray menu and close-to-hide behavior.
- Global shortcuts for toggle window and clipboard capture.
- Full-text search.
- Markdown-native reminder creation, reminder aggregation, source-line
  navigation, delivery deduplication, and native due notifications.
- Manual clipboard capture to `clipboard.md`.
- Standalone `neopad-mcp` read and write tools.
- MCP stdio child-process integration tests.
- Windows Tauri WebDriver smoke tests for the critical note workflow.
- Windows icon, tray/taskbar icon, MSI branding, desktop shortcut icon, and
  hidden release console window.
- Synchronized application versions with a CI consistency check and
  release changelog.

## Current Build Targets

- Windows MSI package.
- Standalone `neopad-mcp.exe`.

## Recommended Next Tasks

1. Expand frontend coverage to search, settings validation, and editor
   transformations.
2. Add Windows-native automation for tray menus, global shortcuts, and MSI
   install/uninstall behavior.
3. Review the custom WiX template for portability before moving the repository
   path or building on CI.
4. Decide whether `neopad-mcp` should be bundled as a sidecar or distributed as
   a separate binary.
5. Add a signed release workflow before publishing installers.

## Non-Goals for the MVP

- Cloud sync.
- Accounts.
- AI chat UI.
- RAG or vector database.
- Backlinks or graph view.
- Plugin system.
- Automatic clipboard history capture.
- Physical deletion of user notes.
