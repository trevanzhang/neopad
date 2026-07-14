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
  mcp-server/           Standalone MCP HTTP server
```

Both the desktop app and MCP server use `neopad-core` for workspace access. Do
not duplicate note, search, path, or write logic outside the core crate.

## Frontend Composition

`App.vue` is the composition root rather than the owner of every application
state machine. Domain behavior is split into focused Vue composables:

- `useDocumentSession` owns loading, autosave, stale-load generations, and
  external-document revisions.
- `useNoteLifecycle` owns tab selection, create/rename/close/trash/archive,
  clipboard capture, and the mandatory save barrier before navigation.
- `usePreferenceState` and `useNativeSettings` separate browser persistence
  from Tauri-native settings synchronization.
- `useSearchState`, `useReminderState`, `useArchiveState`, and `useMcpService`
  own their respective async state and failure boundaries.
- `useNoteExport` resolves the requested tab safely, enforces the active-note
  save barrier, and coordinates rendered PNG/PDF generation and native saving.
- `useDialogs` serializes input and confirmation requests.
- `keyboard-shortcuts.ts` is a pure priority router with explicit state getters
  and application actions; characterization tests protect modal blocking,
  Escape precedence, tab cycling, and native window fallback.
- `useWindowLifecycle` owns Tauri event listeners and save-before-hide/quit
  behavior.
- `text-transform.ts`, `help-content.ts`, and the document utility modules keep
  pure text behavior and static presentation data outside the composition root.
- `EditorPane.vue` owns the live CodeMirror instance and command surface only.
  The custom search panel, editor themes, and line calculator live in focused
  `editor-*` modules; pure match-count and expression behavior has regression
  coverage independent of the desktop runtime.

Keep dependencies explicit when adding a composable. Do not introduce a global
store merely to reduce prop or line counts. New global shortcuts must extend
the router tests whenever they interact with modal or overlay precedence.
CodeMirror adapters should remain separate from pure editor behavior so the
latter can be exercised in Vitest without WebView or DOM setup.

## Data Flow

```text
Vue UI
  -> Tauri command
  -> neopad-core
  -> ~/.neopad/notes/*.md

MCP client
  -> neopad-mcp over local HTTP
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
```

Deleting a note moves its Markdown file to `trash/`. Emptying the visible Trash
moves those files to the operating system's Recycle Bin or Trash before
deleted-tab metadata is removed. If an operating-system Trash operation fails,
the affected file and its metadata remain in NeoPad's Trash. Metadata
reconciliation also recreates deleted-tab entries for NeoPad-formatted files
that the operating system restores to `trash/`.

The app presents notes as tabs. The durable source of note content is always a
Markdown file in `notes/`. `tabs.json` stores tab metadata such as title, file
name, timestamps, pinned state, active tab, and optional tab color.
`reminders.json` stores only notification delivery signatures so reminder text
continues to live in ordinary Markdown note lines.

Desktop preferences are persisted in `config.json`. Browser `localStorage`
supports frontend startup, while native `config.json` is authoritative in the
desktop runtime. Config loading uses defaults for older files so missing fields
from previous versions do not block startup.

## Core Responsibilities

`neopad-core` owns:

- Workspace path resolution and first-run initialization.
- Safe note path construction.
- Note list, read, create, write, rename, append, close, archive, restore, and
  trash operations.
- Atomic writes through temporary files and replace/rename behavior.
- Strictly monotonic internal revisions and content-hash revisions for
  external-file conflict protection.
- Full-text search over active and archived Markdown note files.
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
- A shared safe Markdown renderer with raw HTML disabled, highlighted fenced
  code, KaTeX formulas, and strict-mode Mermaid diagrams. Export uses the same
  renderer so preview and saved output do not drift.
- Content-focused PNG and A4 PDF export. Screenshot and PDF engines are loaded
  on demand, and binary files are saved atomically through `neopad-core`.
- Persistent light and dark themes.
- Native Windows whole-window opacity controlled through a Tauri command and
  `SetLayeredWindowAttributes`; CSS opacity is not used for the app shell.
- Tab context actions for rename, trash, persistent color selection, file
  manager reveal, absolute-path copying, and PNG/PDF export.
- Compact settings dialog with independently scrollable content. Optional Vim
  controls live under Vim settings, while AI provider and MCP service controls
  have their own tabs.
- Compact AI chat composer opened by `Ctrl+K` and AI actions opened by
  CodeMirror Slash completion. Responses apply only after an explicit insert
  or replace action.
- Autosave and status reporting.
- Compact reminder creation and list surfaces. New reminder lines use
  `- [ ] @remind YYYY-MM-DD HH:mm content`; checking the Markdown task
  completes the reminder. Legacy `@提醒` lines continue to parse.
- Local keyboard shortcuts:
  - `F1`: open shortcut help.
  - `F2` / `F3`: switch to the previous / next tab.
  - `F4`: show or hide the note browser.
  - `F5`: cycle edit, split, and preview editor modes.
  - `F6`: toggle the reminder list.
  - `F7`: toggle window on top.
  - `F8`: rename the current page.
  - `F10`: cycle the Markdown preview theme.
  - `Ctrl+,`: open settings.
  - `Ctrl+K`: open the current note's AI chat.
  - `F9`: toggle the light or dark theme.
  - `F11`: toggle immersive fullscreen.
- Single and bulk overdue completion operations atomically change `[ ]` to
  `[x]` while preserving the reminder lines.
- Native reminder notifications while the desktop process remains running.
- On startup, NeoPad offers to restore any preserved note write left by a
  failed atomic replacement.
- New installations start in Edit mode; later launches restore the Default Mode
  chosen in Settings > General. `F5` is the fixed local shortcut for cycling
  edit, split, and preview modes.
- `Ctrl+O`, native file drag and drop, and operating-system file associations
  open external Markdown files in place. Their canonical paths are persisted,
  and writes use atomic replacement with a SHA-256 content-revision conflict
  check. The file remains outside NeoPad lifecycle operations unless copied
  into archive.
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

## AI Collaboration Responsibilities

The Vue layer owns one volatile conversation per note, the compact composer,
prompt selection, context scope, and result actions. CodeMirror owns Slash
completion and applies accepted text as one undoable transaction. A replacement
first checks that the captured text still matches the editor.

The Rust shell validates the configured URL, refuses remote plain HTTP,
disables redirects, bounds responses, and calls the configured
`/chat/completions` endpoint. Non-secret provider settings are stored in
`config.json`; the API key is stored through the platform credential manager.
AI is off by default and conversations are not persisted. User prompt files are
read from `~/.neopad/prompts/*.md` through `neopad-core`. Whole-workspace scope
uses core text relevance search and sends a bounded set of excerpts with source
metadata; it does not concatenate the workspace or maintain a vector index.

This client is independent of `neopad-mcp`. AI collaboration sends an explicit
editor request to a model service, while MCP exposes workspace tools to
external agents.

## MCP Responsibilities

`neopad-mcp` owns the local HTTP MCP protocol surface. It validates bearer
tokens before parsing request bodies, validates local browser origins, caps
request bodies at 2 MiB, then delegates note operations to `neopad-core`.

The service is off by default in the desktop app. When enabled, local agents
with the token can read and write notes. The desktop process owns lifecycle:
start on launch when enabled, stop when disabled, keep running while hidden to
tray, and stop on full app exit. On Windows the sidecar is spawned without a
console window.

## Packaging

The Windows target builds an MSI package only. Cross-platform build scripts
select DMG on macOS and DEB plus AppImage on Linux. The Tauri config points to:

```text
app/src-tauri/wix/main.wxs
```

This custom WiX template exists so the installer can control:

- Branded banner and dialog bitmaps.
- Product icon.
- Start-menu shortcut icon.
- Desktop shortcut icon.
- AppUserModel.ID on shortcuts.
- Installation of `neopad-mcp.exe` as the desktop app's MCP sidecar.

Installer bitmap assets live in:

```text
app/src-tauri/icons/wix-banner.bmp
app/src-tauri/icons/wix-dialog.bmp
```
