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
- Note CRUD, close-without-delete, archive, and trash move behavior.
- Durable atomic writes, monotonic update timestamps, and content-revision
  checks for out-of-band edits.
- Cross-process write locking for desktop and write-enabled MCP operations.
- Tauri command bridge.
- Vue app shell with tabs, editor, preview, search, settings, and status bar.
- Edit, split, and preview modes with a compact View menu, status indicator,
  fixed `F8` cycling shortcut, and a Default Mode selector that takes effect
  immediately and is restored at startup. New installations start in Edit mode.
- Page rename and trash actions with UI and core protection for default pages.
- Tab context menu with rename, archive, trash, persistent color selection,
  and an external-file marker.
- Localized system page labels that preserve stable storage IDs and user titles.
- CodeMirror 6 Markdown editor.
- Optional persisted Vim key bindings with Normal, Insert, and Visual status,
  plus a configurable Insert exit sequence in Vim settings.
- Persistent light and dark themes with a destination-state status-bar toggle.
- Viewport-constrained settings dialog with fixed navigation and actions.
- Autosave from the UI.
- Serialized autosave and tray/global-command handling that preserves pending
  edits, with deterministic frontend tests.
- Tray menu and close-to-hide behavior.
- Global shortcuts for toggle window and clipboard capture.
- Full-text search with grouped, collapsible results sorted by title match and
  per-note match count.
- Markdown-native reminder creation, reminder aggregation, source-line
  navigation, status filtering, completion reopening, delivery deduplication,
  and native due notifications.
- ZIP export for active and archived NeoPad notes, preserving each note as its
  own Markdown file.
- Native picker, drag-and-drop, and operating-system file-association opening
  with in-place autosave for external Markdown files; content-revision
  conflicts, persistent recent-document list, and copy-to-archive behavior.
- Manual clipboard capture to `clipboard.md`.
- Standalone Streamable HTTP `neopad-mcp` read and write tools.
- Dedicated MCP settings page with service start/stop, status, token display,
  token regeneration, and copyable client configuration.
- MCP HTTP child-process integration tests.
- Windows Tauri WebDriver smoke tests for the critical note workflow.
- Windows icon, tray/taskbar icon, MSI branding, desktop shortcut icon, bundled
  MCP sidecar, and hidden release console windows.
- Synchronized application versions with a CI consistency check and
  release changelog.
- Frontend composition split into focused document, note lifecycle,
  preferences, native settings, search, reminder, archive, MCP, dialog, window,
  and editor-support modules with deterministic unit coverage.
- Audited cross-platform CI and draft release workflows with pinned actions,
  production dependency gates, exact artifact validation, and SHA-256 manifests.

## Current Build Targets

- Windows x64 MSI.
- macOS ARM64 DMG.
- Linux x64 DEB and AppImage.
- Bundled standalone `neopad-mcp` sidecar.

## Recommended Next Tasks

1. Expand frontend coverage for settings validation and DOM interaction in the
   custom editor search panel.
2. Add Windows-native automation for tray menus, global shortcuts, and MSI
   install/uninstall behavior.
3. Review the custom WiX template for portability before moving the repository
   path or building on CI.
4. Add a signed release workflow before publishing installers.

## Non-Goals for the MVP

- Cloud sync.
- Accounts.
- AI chat UI.
- RAG or vector database.
- Backlinks or graph view.
- Plugin system.
- Automatic clipboard history capture.
- Physical deletion of user notes.
