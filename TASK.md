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
- Plain Markdown persistence under `~/.neopad/notes/*.md`, with archived
  NeoPad notes stored under `~/.neopad/archive/`.
- Multiple tabs backed by local metadata, including close-without-delete and
  recent-document workflows.
- Autosave.
- Edit, split, and preview editor modes with a persisted Default Mode setting
  and fixed `F4` cycling shortcut; all three modes remain available in
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
  `F5` toggles the list, list filters can narrow reminders by status, and list
  actions complete or reopen reminder checkboxes without deleting their
  Markdown lines.
- Native Save As for exporting the active note to Markdown or all notes to a
  ZIP archive with one Markdown file per tab.
- `Ctrl+O` opens external Markdown files in place and autosaves changes back
  to their original paths. Access requires native-picker approval and writes
  use content revisions to detect out-of-band edits. External files can be
  copied into the NeoPad archive without changing the originals.
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
- Windows MSI packaging with branded installer assets.

## Data Rules

- User note bodies are stored only as Markdown files under `notes/`.
- Metadata belongs in `config.json`, `meta/tabs.json`, and
  `meta/reminders.json`. Reminder content must remain in `notes/*.md`; the
  reminder metadata file stores notification delivery state only.
- Delete operations move notes into `trash/`.
- Core filesystem access must go through `neopad-core`.
- Paths must be validated so callers cannot escape the configured workspace.
- Atomic writes must be preserved.
- Navigation and content-replacing actions must stop when the save barrier
  cannot persist pending edits.
- Full-page updates must keep `expectedUpdatedAt` conflict protection, and
  external file changes must also be detected through content revisions.

## MCP Rules

- `neopad-mcp` communicates over local Streamable HTTP at `/mcp`.
- The service is off by default and binds to `127.0.0.1` by default.
- Bearer token authentication is required for HTTP requests.
- Desktop-managed bearer tokens must be passed through the child environment,
  not command-line arguments.
- Browser-originated requests must pass local Origin validation.
- stderr is used for diagnostics.
- When enabled, local agents with the token can read and write notes.
- The MCP server must not read the system clipboard.
- The MCP server must not access files outside the workspace.
- Installed builds must include `neopad-mcp.exe` as a sidecar.

## Packaging Rules

Supported distributable targets are Windows MSI, macOS ARM64 DMG, and Linux
DEB plus AppImage. Bundle types are selected explicitly; NSIS is not enabled.

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
