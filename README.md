# NeoPad

NeoPad is a lightweight, local-first desktop note pad for fast capture, quick
recall, and agent-friendly access to Markdown notes.

The app is inspired by FlashPad-style workflows: a small desktop window, tray
presence, global shortcuts, multiple note tabs, autosave, full-text search,
clipboard capture, and Markdown preview. Notes are stored as plain `.md` files
under the user's local NeoPad workspace.

## Current Status

The repository currently contains a working Windows-focused MVP:

- Tauri 2 desktop app with Vue 3, TypeScript, Vite, CodeMirror 6, and
  `markdown-it`.
- Shared Rust core crate for workspace initialization, note CRUD, atomic writes
  with startup recovery for a preserved failed note write, trash moves, search,
  config, tabs, and path safety.
- System tray menu, close-to-hide behavior, global shortcuts, and clipboard
  append.
- Flicker-free single-instance startup, first-launch placement at the screen's bottom-right,
  restoration of the last window position on later launches, and an option to
  keep the window hidden after any launch.
- Configurable global shortcuts for window visibility and clipboard capture,
  including reliable restore from the minimized state, plus `Alt+Enter`
  maximize/restore.
- Clipboard captures use readable local timestamps in compact separator lines.
- Edit, split, and preview editor modes selectable from the View menu, status
  bar, and fixed `F5` shortcut. The Default Mode selector in Settings > General
  takes effect immediately and is restored on later launches; new installations
  start in Edit mode.
- Optional Vim key bindings for the CodeMirror editor, with persisted settings
  in the Vim tab, configurable Insert exit sequence, and a visible Normal,
  Insert, or Visual mode indicator. NeoPad Ctrl shortcuts can optionally take
  priority over conflicting Vim mappings.
- Persistent light and dark themes available from the Format menu and status
  bar.
- Markdown preview appearance settings with well-known theme presets, including
  One Dark, Nord, Solarized, Monokai, GitHub Light, and Dracula, plus preview
  typography controls.
- Markdown source and fenced-code syntax highlighting, KaTeX math formulas,
  and asynchronously rendered Mermaid diagrams in split and preview modes.
- A compact editor font dialog with preset font choices, font-size control, and
  live preview.
- Lightweight Markdown reminders using
  `- [ ] @remind YYYY-MM-DD HH:mm content`, with a compact `Ctrl+E` editor,
  sortable and filterable reminder list, source-line navigation, and native
  notifications while NeoPad is running in the tray.
- `F1` opens shortcut help, `F2` and `F3` switch to the previous and next tab,
  and `F6` opens or closes the reminder list. "Mark Completed", "Mark Unfinished",
  and "Clear Due" update reminder checkboxes directly without deleting note
  content.
- `F4` opens the compact note library for active, archived, and trashed notes;
  it keeps everyday note actions close to the editor without turning NeoPad
  into a knowledge-base suite.
- Right-click a tab or a single note in the note library to reveal its Markdown
  file in the system file manager. The tab menu can also copy the absolute
  Markdown file path for use with terminals and local agents.
- The reorganized Settings center groups General, Editor, Preview, Vim,
  Shortcuts, AI, and MCP controls in a focused dialog, including configurable
  visibility and clipboard-capture shortcuts.
- Optional AI collaboration provides a compact `Ctrl+K` note chat, reusable
  Markdown prompts, bounded all-note context search, and independent `//`
  commands for continue, polish, summarize, and Chinese-English translation.
  A selected `//` command applies its result directly as one undoable edit;
  right-clicking an exact editor selection exposes direct AI actions, while
  chat results still require an explicit copy or insert action. The Help menu
  includes structured Software, shortcut, Markdown, expression, and AI guides.
  Provider credentials stay in the operating system credential manager.
- `Ctrl+W` closes the current non-pinned tab without changing its file. The tab
  context menu also provides explicit archive and trash actions, while recent
  documents provide a quick way to reopen closed or archived notes.
- Archived notes move to `~/.neopad/archive/`, remain available to full-text
  search, and can be listed and restored from the note browser without using
  the trash. File > Open Archive Folder opens the archive directory directly.
- If a non-default note file is removed outside NeoPad, its stale tab and recent
  document record are removed on the next refresh; NeoPad does not recreate the
  missing file.
- Help > About NeoPad displays the version embedded in the running application
  build.
- Global search groups matching lines by note, shows a per-note match count,
  and lets you expand repeated matches only when needed.
- `Ctrl+O`, dragging `.md` or `.markdown` files onto the window, or opening an
  associated Markdown document from the operating system opens it in place.
  Changes autosave back to the original path with a SHA-256 content-revision
  conflict check. External files can be copied into the NeoPad archive without
  moving or deleting their originals. Native Save As dialogs export the active
  note as Markdown, PNG, or multi-page PDF, or export all notes as a ZIP archive
  containing one Markdown file per tab. PNG and PDF actions are available from
  both the File menu and the tab context menu.
- Standalone local HTTP `neopad-mcp` service managed from Settings, with bearer
  token access for local agents. The dedicated MCP settings page can start or
  stop the service, show the local URL and token, regenerate the token, and copy
  an agent configuration snippet.
- Save barriers prevent navigation and content-replacing actions from
  discarding pending edits. Note metadata is reconciled under a cross-process
  lock, and interrupted archive/trash state is recovered from Markdown files.
- Windows MSI packaging with app icon, branded WiX installer images, desktop and
  start-menu shortcut icons, and `.md` / `.markdown` file-association support.
  The MSI includes `neopad-mcp.exe` as a bundled sidecar binary.

## Workspace Layout

```text
app/                  Tauri desktop app and Vue frontend
app/src-tauri/        Rust shell for Tauri commands, tray, hotkeys, packaging
crates/neopad-core/   Shared Rust core library
mcp-server/           Standalone neopad-mcp binary
docs/                 Architecture, development, MCP, and task notes
```

## User Data

By default NeoPad stores all user data in:

```text
~/.neopad/
```

Important files and directories:

```text
~/.neopad/
  notes/              Markdown note bodies
  archive/            Archived notes, included in full-text search
  meta/tabs.json      Tab metadata
  meta/reminders.json Notification delivery state
  config.json         App settings
  trash/              Deleted notes are moved here
```

Active note content lives in `notes/*.md`. Emptying NeoPad's Trash moves its
Markdown files to the operating system's Recycle Bin or Trash before removing
their entries from NeoPad's library. Files restored to NeoPad's `trash/`
directory by the operating system are rediscovered on refresh. Metadata files
do not contain note bodies.

## Development

Install dependencies:

```powershell
pnpm install
```

Run the app in development mode:

```powershell
pnpm tauri:dev
```

Build the frontend:

```powershell
pnpm build
```

Run Rust tests:

```powershell
cargo test
```

Run Rust and frontend unit/integration tests:

```powershell
pnpm test:all
```

Run the real Windows desktop interaction suite:

```powershell
pnpm test:e2e
```

Desktop tests use an isolated workspace under `target/e2e-workspace`.

Build all Rust crates:

```powershell
cargo build
```

Build the desktop app and platform bundle:

```powershell
pnpm tauri:build
```

This command builds the release MCP server first, prepares the Tauri sidecar
name, and then builds an explicit platform-native bundle set (`.msi` on
Windows, `.dmg` on macOS, `.deb` plus `.AppImage` on Linux). It does not build
NSIS.

The Windows MSI is written to:

```text
target/release/bundle/msi/NeoPad_0.5.2_x64_en-US.msi
```

Cross-platform release builds also run automatically in GitHub Actions when a
`v*` tag is pushed, producing Windows, macOS (ARM64), and Linux installers.
Before a release is created or published, run the
[release smoke checklist](docs/release-smoke-checklist.md) against the packaged
Windows build.

## MCP Server

Build the MCP server:

```powershell
cargo build -p neopad-mcp --release
```

Run the HTTP server directly without exposing the token in the process command
line:

```powershell
$env:NEOPAD_MCP_TOKEN = '<local-token>'
target\release\neopad-mcp.exe --workspace ~/.neopad serve --host 127.0.0.1 --port 8765
```

NeoPad can also start and stop this process from Settings. The copied agent
configuration uses the local URL and bearer token:

```json
{
  "mcpServers": {
    "neopad": {
      "url": "http://127.0.0.1:8765/mcp",
      "headers": {
        "Authorization": "Bearer <local-token>"
      }
    }
  }
}
```

The service is off by default. When enabled, local agents with the token can
read and write NeoPad notes. The HTTP server rejects non-loopback bind
addresses. The desktop app also passes its managed token through the child
environment rather than command-line arguments.

## Keyboard Shortcuts

Common local shortcuts:

```text
F1              Shortcut help
F2              Switch to the previous tab
F3              Switch to the next tab
F4              Show or hide the note browser
F5              Cycle edit, split, and preview modes
F6              Open or close the reminder list
F7              Toggle window on top
F8              Rename current page and its untouched default heading
F9              Toggle day/night mode
F10             Toggle preview theme
Ctrl+,          Open settings
F11             Toggle immersive fullscreen
F12             Archive the current page
Alt+Enter       Maximize or restore the main window
Esc             Close overlays, exit fullscreen, or hide the window
Ctrl+O          Open an external Markdown file
Ctrl+W          Close the current tab
Alt+Del         Move the current page to Trash
Ctrl+E          Insert a Markdown reminder
Ctrl+Tab        Next tab
Ctrl+Shift+Tab  Previous tab
```

Configurable global shortcuts:

```text
Alt+Z           Show or hide NeoPad
Ctrl+Shift+V    Append current clipboard text to clipboard.md
```

## More Documentation

- [中文说明](README_CN.md)
- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [Markdown preview and export](docs/export.md)
- [MCP](docs/mcp.md)
- [AI collaboration](docs/ai.md)
- [Tasks](docs/tasks.md)
- [Changelog](CHANGELOG.md)
- [Agent guide](AGENTS.md)
