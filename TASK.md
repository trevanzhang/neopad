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
- CodeMirror 6 Markdown editor with Markdown and common fenced-code language
  highlighting.
- Optional Vim key bindings with an explicit mode indicator. Vim support stays
  an editor input mode rather than a plugin or Vim runtime system; its small
  set of options lives under Advanced settings, including whether NeoPad Ctrl
  shortcuts take priority over conflicting Vim mappings.
- Plain Markdown persistence under the flat active work area
  `~/.neopad/notes/*.md`, with categorized archived notes stored under
  `~/.neopad/archive/**/*.md`.
- Multiple tabs backed by local metadata, including close-without-delete and
  recent-document workflows.
- Tab and single-file browser context menus can reveal note, prompt, or
  external Markdown files in the system file manager. Tab menus can also copy
  the current document's absolute file path, while the File menu opens the
  local archive directory directly.
- Autosave.
- Per-document undo and redo with `Ctrl+Z`, `Ctrl+Y`, and `Ctrl+Shift+Z`;
  loading or switching documents starts a fresh history boundary so edits
  cannot be undone into another note's content.
- Edit, split, and preview editor modes with a persisted Default Mode setting
  and fixed `F5` cycling shortcut; all three modes remain available in
  immersive fullscreen. New installations start in Edit mode, and later
  launches restore the saved default.
- Page creation with incrementing untitled names, rename, archive, and trash
  actions with protected default pages.
- Persistent optional tab colors managed from the tab context menu.
- English and Chinese display names for system-managed page titles.
- Persistent day and night themes across the complete desktop interface.
- Native whole-window opacity controlled from an in-app slider.
- Compact settings that remain usable at the minimum supported window size.
- Full-text search with grouped, collapsible results, per-note match counts,
  and a paginated load-more pager.
- Lightweight reminders stored as readable Markdown task lines. `Ctrl+E`
  opens a compact reminder editor, the reminder list derives its rows from
  note content, and due reminders use native notifications while NeoPad is
  running.
  `F6` toggles the list, list filters can narrow reminders by status, and list
  actions complete or reopen reminder checkboxes without deleting their
  Markdown lines.
- Native Save As for exporting the active note to Markdown or all notes to a
  ZIP archive with one Markdown file per tab.
- White-background PNG and multi-page PDF export for the current tab, including
  rendered code highlighting, KaTeX formulas, and Mermaid diagrams. PNG output
  can be saved to a file or copied directly to the system clipboard. Export
  rendering stays content-focused and loads its heavier engines on demand.
- `Ctrl+O`, native file drag and drop, and operating-system file associations
  open external Markdown files in place and autosave changes back to their
  original paths. Writes use content revisions to detect out-of-band edits.
  External files can be copied into the NeoPad archive without changing the
  originals.
- Manual clipboard capture into `clipboard.md` with readable local timestamp
  separator lines.
- Tray menu.
- Flicker-free single-instance startup and persistent main-window position, with a
  user-selectable hidden background launch mode and a
  bottom-right default on first launch.
- Global shortcuts.
- Configurable global window and clipboard shortcuts, plus local `Alt+Enter`
  maximize/restore.
- Close-to-hide behavior.
- Standalone Streamable HTTP MCP server managed from a dedicated MCP settings
  page.
- Optional AI collaboration with a compact `Ctrl+K` note chat and independent
  `//` continue, polish, summarize, and Chinese-English translation commands.
  Selecting a `//` command applies generated Markdown as one undoable edit;
  chat results remain explicit insert, replace, or copy actions. Right-clicking
  an editor selection exposes direct polish, summarize, and translate actions.
  The Help menu documents setup, all three entry points, prompts, and privacy.
- Prompt Markdown files are organized under `prompts/**/*.md` and managed from
  the compact `F4` file browser with local folders. Files and complete folder
  trees can be moved by drag and drop; folders can be renamed or deleted, with
  their managed Markdown files retained in NeoPad Trash. Archived notes can
  likewise be organized into folders and opened without restoring them to the
  flat `notes/` work area; restoration is explicit. Prompts open in the main
  editor as marked tabs. They use the normal save barrier and
  content-revision conflict checks, but remain outside note search, archive,
  reminders, recent notes, and MCP note operations.
- Structured in-app Software Help that introduces NeoPad's product direction,
  core note workflows, editor and search tools, capture and export paths,
  reminders, AI and MCP boundaries, and local-first data guarantees.
- Grouped in-app Markdown, Vim, and expression references that document
  supported rendering syntax, Vim editing and NeoPad-specific tab mappings,
  NeoPad reminders, calculation operators, examples, and unsupported
  expression behavior.
- Windows MSI packaging with branded installer assets.

## Data Rules

- Active user note bodies are stored only as Markdown files under the flat
  `notes/` work area; archived note bodies may be categorized under `archive/`.
- Metadata belongs in `config.json`, `meta/tabs.json`, `meta/prompts.json`, and
  `meta/reminders.json`. Reminder content must remain in note Markdown; the
  reminder metadata file stores notification delivery state only.
- User-authored reusable AI prompts live in `prompts/**/*.md` and remain separate
  from note content.
- Delete operations move notes and prompts into `trash/`; prompt trash remains
  distinguishable from note trash and must not be reconciled as note content.
- Emptying Trash moves its Markdown files into the operating system's Recycle
  Bin or Trash before removing deleted-tab metadata; it must not unlink note
  files. Files restored by the operating system must be rediscovered on the
  next metadata reconciliation.
- Core filesystem access must go through `neopad-core`.
- Paths must be validated so callers cannot escape the configured workspace.
- Atomic writes must be preserved. If Windows leaves a failed note-write
  temporary file behind, NeoPad must offer a safe restore on the next startup.
- Navigation and content-replacing actions must stop when the save barrier
  cannot persist pending edits.
- Full-page updates must keep `expectedUpdatedAt` conflict protection, and
  external file changes must also be detected through content revisions.

## MCP Rules

- `neopad-mcp` communicates over local Streamable HTTP at `/mcp`.
- The service is off by default and binds to `127.0.0.1` by default.
- Bearer token authentication is required for HTTP requests.
- Authentication must run before request-body parsing, and request bodies must
  remain bounded.
- Desktop-managed bearer tokens must be passed through the child environment,
  not command-line arguments.
- Browser-originated requests must pass local Origin validation.
- stderr is used for diagnostics.
- When enabled, local agents with the token can read and write notes.
- The MCP server must not read the system clipboard.
- The MCP server must not access files outside the workspace.
- Installed builds must include `neopad-mcp.exe` as a sidecar.

## AI Rules

- AI is off by default and runs only after an explicit user action.
- The desktop shell calls an OpenAI-compatible service configured by the user.
- Remote service URLs require HTTPS; loopback HTTP remains available for local
  model servers.
- The service URL and model belong in `config.json`. API keys must remain in
  platform secure storage and must not be returned to the webview after save.
- AI chat responses must not modify a note automatically. Selecting a `//`
  quick command is the explicit action that authorizes one undoable insert or
  replacement transaction.
- Slash command write ranges remain selection/paragraph scoped, while the
  current note is provided as read-only model context; long note references are
  bounded around the operation target.
- Replacements must verify that their captured source text is still current.
- `Ctrl+K` conversations are kept in memory per note and never persisted.
- Whole-workspace context uses local text relevance search and sends only a
  bounded set of excerpts rather than concatenating every note.
- User-authored reusable prompts live in `~/.neopad/prompts/**/*.md`. The `F4`
  file browser owns prompt creation, rename, duplicate, trash, restore, and
  folder organization actions. Open prompt tabs use atomic writes with
  content-revision conflicts.
- AI request bodies, credentials, and note text must not be written to logs.

## Packaging Rules

Supported distributable targets are Windows MSI, macOS ARM64 DMG, and Linux
DEB plus AppImage. Bundle types are selected explicitly; NSIS is not enabled.

Installer-related files:

```text
app/src-tauri/tauri.conf.json
app/src-tauri/wix/main.wxs
app/src-tauri/wix/locales/en-US.wxl
app/src-tauri/icons/icon.ico
app/src-tauri/icons/wix-banner.bmp
app/src-tauri/icons/wix-dialog.bmp
```

The WiX dialog bitmap is used as a full background on welcome and finish pages.
Keep the right side visually quiet so default WiX text remains readable. The
WiX banner must remain free of branding and text because native installer copy
is drawn over its left side.

The Windows MSI uses English for its standard installer pages and includes one
bilingual application-language selection page. Chinese is selected by default,
English is available, and the choice seeds only the first launch of a new
default workspace without replacing existing user preferences.

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
- Persistent AI chat sidebar or chat history.
- RAG.
- Vector database.
- Backlinks.
- Graph view.
- Plugin system.
- Automatic clipboard history capture.
- Physical deletion of notes.
