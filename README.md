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
- Shared Rust core crate for workspace initialization, note CRUD, atomic writes,
  trash moves, search, config, tabs, and path safety.
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
  bar, and fixed `F4` shortcut. The Default Mode selector in Settings > General
  takes effect immediately and is restored on later launches; new installations
  start in Edit mode.
- Optional Vim key bindings for the CodeMirror editor, with persisted settings
  in the Advanced tab, configurable Insert exit sequence, and a visible Normal,
  Insert, or Visual mode indicator. NeoPad Ctrl shortcuts can optionally take
  priority over conflicting Vim mappings.
- Persistent light and dark themes available from the Format menu and status
  bar.
- Markdown preview appearance settings with well-known theme presets, including
  One Dark, Nord, Solarized, Monokai, GitHub Light, and Dracula, plus preview
  typography controls.
- A compact editor font dialog with preset font choices, font-size control, and
  live preview.
- Lightweight Markdown reminders using
  `- [ ] @remind YYYY-MM-DD HH:mm content`, with a compact `Ctrl+E` editor,
  sortable and filterable reminder list, source-line navigation, and native
  notifications while NeoPad is running in the tray.
- `F1` opens shortcut help, `F2` renames the current page, and `F5` opens or
  closes the reminder list. "Mark Completed", "Mark Unfinished",
  and "Clear Due" update reminder checkboxes directly without deleting note
  content.
- `Ctrl+W` closes the current non-pinned tab without changing its file. The tab
  context menu also provides explicit archive and trash actions, while recent
  documents provide a quick way to reopen closed or archived notes.
- Archived notes move to `~/.neopad/archive/`, remain available to full-text
  search, and can be listed and restored from File > View Archive without using
  the trash.
- If a non-default note file is removed outside NeoPad, its stale tab and recent
  document record are removed on the next refresh; NeoPad does not recreate the
  missing file.
- Help > About NeoPad displays the version embedded in the running application
  build.
- Global search groups matching lines by note, shows a per-note match count,
  and lets you expand repeated matches only when needed.
- `Ctrl+O` opens an external Markdown file in place through the native file
  picker. Only picker-approved paths can be reopened, and changes autosave back
  to the original path with a SHA-256 content-revision conflict check. External
  files can be copied into the NeoPad archive without moving or deleting their
  originals. Native Save As dialogs export the active note as Markdown or all
  notes as a ZIP archive containing one Markdown file per tab.
- Standalone local HTTP `neopad-mcp` service managed from Settings, with bearer
  token access for local agents. The dedicated MCP settings page can start or
  stop the service, show the local URL and token, regenerate the token, and copy
  an agent configuration snippet.
- Save barriers prevent navigation and content-replacing actions from
  discarding pending edits. Note metadata is reconciled under a cross-process
  lock, and interrupted archive/trash state is recovered from Markdown files.
- Windows MSI packaging with app icon, branded WiX installer images, and desktop
  and start-menu shortcut icons. The MSI includes `neopad-mcp.exe` as a bundled
  sidecar binary.

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
  backups/            Reserved for future backup support
```

Note content lives in `notes/*.md`. Metadata files do not contain note bodies.

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
target/release/bundle/msi/NeoPad_0.4.6_x64_en-US.msi
```

Cross-platform release builds also run automatically in GitHub Actions when a
`v*` tag is pushed, producing Windows, macOS (ARM64), and Linux installers.

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
F2              Rename current page and its untouched default heading
F4              Cycle edit, split, and preview modes
F5              Open or close the reminder list
F7              Cycle preview theme
F8              Open settings
F9              Toggle day/night mode
F10             Show or hide the note library
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
- [MCP](docs/mcp.md)
- [Tasks](docs/tasks.md)
- [Changelog](CHANGELOG.md)
- [Agent guide](AGENTS.md)
