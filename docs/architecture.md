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

neopad-cli
  cli/                  Standalone native CLI
```

Both the desktop app and CLI use `neopad-core` for workspace access. Do
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
- `useSearchState`, `useReminderState`, `useArchiveState`, and `useAgentTools`
  own their respective async state and failure boundaries.
- `useNoteExport` resolves the requested tab safely, enforces the active-note
  save barrier, and coordinates rendered PNG/PDF generation and native saving.
- `useDialogs` serializes input and confirmation requests.
- `keyboard-shortcuts.ts` is a pure priority router with explicit state getters
  and application actions; characterization tests protect modal blocking,
  Escape precedence, tab cycling, editor undo/redo routing, and native window
  fallback.
- `useWindowLifecycle` owns Tauri event listeners and save-before-hide/quit
  behavior.
- `text-transform.ts`, `help-content.ts`, and the document utility modules keep
  pure text behavior and static presentation data outside the composition root.
- `EditorPane.vue` owns the live CodeMirror instance and command surface only.
  The custom search panel, editor themes, and line calculator live in focused
  `editor-*` modules; pure match-count and expression behavior has regression
  coverage independent of the desktop runtime. A loaded document starts with a
  fresh CodeMirror state so undo/redo history cannot cross note or
  external-document boundaries; loading content is never an undoable edit.

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
  -> ~/.neopad/notes/*.md + projects/**/*.md

Agent process
  -> neopad-cli
  -> neopad-core
  -> ~/.neopad/notes/*.md + projects/**/*.md
```

## Local Workspace

The default workspace is `~/.neopad/`.

```text
~/.neopad/
  notes/
    inbox.md
    clipboard.md
    page-*.md
  archive/
    category/
      page-*.md
  projects/
    project/
      note.md
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

The app presents capture and project notes as ordinary tabs. Capture stays in
flat `notes/`; project Markdown is nested under `projects/`. `tabs.json` stores
stable IDs, optional project/archive paths, and exact trash filenames. All note
operations resolve the same paths in the shared core. Archive a project into
`archive/projects/<name>/`; restoring a note recreates its original project path.

Directory operations reject unsafe paths, links, conflicting destinations, and
unmanaged non-Markdown files. Legacy `prompts/` and `trash/prompts/` migrate into
ordinary notes under `projects/原提示词/` and unified Trash. Migration journals
metadata before each file move, resumes after interruption, and selects a fresh
project name when a destination exists. Existing legacy metadata is retained.

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
  controls live under Vim settings, while AI provider and Agent tools controls
  have their own tabs.
- Compact AI chat composer opened by `Ctrl+K` and independent one-shot editing
  actions opened by CodeMirror `//` completion. Selecting a quick action
  authorizes one direct, undoable editor transaction.
- Capture and project tabs use the ordinary note API; external Markdown uses
  its approved external-document API. There is no prompt document kind.
- Autosave and status reporting.
- Compact reminder creation and list surfaces. New reminder lines use
  `- [ ] @remind YYYY-MM-DD HH:mm content`; checking the Markdown task
  completes the reminder. Legacy `@提醒` lines continue to parse.
- Local keyboard shortcuts:
  - `F1`: open shortcut help.
  - `F2` / `F3`: switch to the previous / next tab.
  - `F4`: show or hide the note and project file browser.
  - `F5`: cycle edit, split, and preview editor modes.
  - `F6`: toggle the reminder list.
  - `F7`: toggle window on top.
  - `F8`: rename the current page.
  - `F10`: cycle the Markdown preview theme.
  - `Ctrl+,`: open settings.
  - `Ctrl+K`: open the current note's AI chat.
  - `Ctrl+Z`: undo an edit in the current document.
  - `Ctrl+Y` / `Ctrl+Shift+Z`: redo an edit in the current document.
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
- Tray integration: a left click restores and focuses the main window; a right
  click opens the show, hide, new note, save clipboard, settings, and quit menu.
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
context scope, and result actions. CodeMirror owns `//`
completion. Quick editing commands use no chat history or
workspace context. A mapped inline widget shows progress at the captured cursor,
then the result is applied directly as one undoable transaction. A replacement
first checks that its mapped source text still matches the captured text. Note
switches and `Esc` cancel pending commands.

Inline AI plans keep the editable target separate from request context. The
target remains a selection, paragraph, or note range used for guarded writeback.
Continue, polish, and translate additionally send a bounded current-note
reference: full text for ordinary notes, or the note opening plus a target-local
window for long notes. This improves tone and terminology consistency without
broadening the range that AI may replace.

The Rust shell validates the configured URL, refuses remote plain HTTP,
disables redirects, bounds responses, and calls the configured
`/chat/completions` endpoint. Non-secret provider settings are stored in
`config.json`; the API key is stored through the platform credential manager.
AI is off by default and conversations are not persisted. There is no reusable
prompt attachment picker. Whole-workspace scope
uses core text relevance search and sends a bounded set of excerpts with source
metadata; it does not concatenate the workspace or maintain a vector index.

This client is independent of `neopad-cli`. AI collaboration sends an explicit
editor request to a model service, while the CLI exposes workspace tools to
external agents.

## CLI Responsibilities

`neopad-cli` parses commands and UTF-8 input, calls the shared core, and emits
JSON with exit codes for argument, operation, and conflict errors. Note commands
hold the workspace write lock through reconciliation and the operation. Full
replacement requires an expected timestamp. Input is bounded to 2 MiB.

The desktop resolves the bundled binary beside its executable and asks the core
to install it into `.agents/bin/`. SHA-256 comparison detects missing, outdated,
or modified binaries. Deployment is atomic and rejects symlinks or Windows
junctions in the managed path. Settings can check and repair this installation.
This controls the CLI interface; it is not an operating-system sandbox for an
Agent that already has direct filesystem access.

The frontend checks for note changes every two seconds. It reloads a note only
when no edits are pending, and rejects an asynchronous result if typing, saving,
or switching tabs occurred. External Markdown tabs keep their own
lifecycle.

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
- An English application-language selector with one bilingual Chinese-default
  option, persisted for the app's first launch.
- English standard installer pages, with a locale code page that also preserves
  the Chinese option text.
- Installation of `neopad-cli.exe` as the desktop app's native CLI sidecar.

Installer bitmap assets live in:

```text
app/src-tauri/icons/wix-banner.bmp
app/src-tauri/icons/wix-dialog.bmp
```
