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
- Edit, hybrid, and preview editor modes selectable from the View menu, status
  bar, configurable shortcut, and persisted default setting, including while
  using immersive fullscreen.
- Optional Vim key bindings for the CodeMirror editor, with persisted settings
  in the Advanced tab, configurable Insert exit sequence, and a visible Normal,
  Insert, or Visual mode indicator. NeoPad Ctrl shortcuts can optionally take
  priority over conflicting Vim mappings.
- Persistent light and dark themes available from the Format menu and status
  bar.
- Tab context menu with rename, trash, and persistent color choices.
- Native Save As dialogs for exporting the active note or all notes as Markdown.
- Standalone `neopad-mcp` stdio server with read-only tools by default and
  opt-in write tools via `--allow-write`.
- Windows MSI packaging with app icon, branded WiX installer images, and desktop
  and start-menu shortcut icons.

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
  meta/tabs.json      Tab metadata
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

Build the Windows MSI:

```powershell
pnpm tauri:build
```

The MSI is written to:

```text
target/release/bundle/msi/NeoPad_0.2.0_x64_en-US.msi
```

## MCP Server

Build the MCP server:

```powershell
cargo build -p neopad-mcp --release
```

Read-only configuration example:

```json
{
  "mcpServers": {
    "neopad": {
      "command": "D:\\TrevanCode\\neopad\\target\\release\\neopad-mcp.exe",
      "args": ["--workspace", "~/.neopad"]
    }
  }
}
```

Write-enabled configuration example:

```json
{
  "mcpServers": {
    "neopad": {
      "command": "D:\\TrevanCode\\neopad\\target\\release\\neopad-mcp.exe",
      "args": ["--workspace", "~/.neopad", "--allow-write"]
    }
  }
}
```

Write tools are intentionally hidden unless `--allow-write` is provided.

## More Documentation

- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [MCP](docs/mcp.md)
- [Tasks](docs/tasks.md)
- [Changelog](CHANGELOG.md)
- [Agent guide](AGENTS.md)
