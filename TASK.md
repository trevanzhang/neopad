# NeoPad Task Brief

This file is the clean project task brief for the current MVP. It replaces an
older corrupted-encoding draft.

## Product Direction

Version 0.7.5 is the current local test distribution target. It adds bond
holding-period returns and dated cash-flow annual return / cost (XIRR), and
changes the calculator shortcut to Shift+F1. F1 help and F6 reminders remain.
Local packaging and GitHub draft/public release steps are documented in
[Release preparation](docs/release.md); a successful CI run alone does not
publish installers.

Version 0.7.0 replaces the existing MCP integration with the native CLI.
This is an incompatible Agent interface change; local note storage is preserved.

NeoPad is a lightweight, local-first desktop note pad for fast temporary notes.
It should feel like a quick capture utility rather than a full knowledge-base
application.

The public product name is `NeoPad`. Engineering identifiers stay lowercase:

- Repository: `neopad`
- Package names: `neopad-app`, `neopad-core`, `neopad-cli`
- Native CLI binary: `neopad-cli`
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
  `~/.neopad/notes/*.md`, project notes under `~/.neopad/projects/**/*.md`, and categorized archived notes under
  `~/.neopad/archive/**/*.md`.
- Multiple tabs backed by local metadata, including close-without-delete and
  recent-document workflows.
- Tab and single-file browser context menus can reveal note or
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
- Exact editor selections can be opened directly as the body of a new note;
  this operation does not overwrite the system clipboard.
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
- A lazily loaded offline financial calculator (`Shift+F1`) with interest,
  rates, time value and general bond price/YTM/duration/convexity/DV01,
  personal loans, financing / lending bond repos, bond holding-period returns
  and effective annual return / cost (XIRR). Holding returns separate price
  gains, received coupons and actual fees / financing costs. XIRR supports
  2-200 dated receipts / payments, sorting and same-day netting; the initial
  scope requires one sign change and a 365-day year. Loans solve payment,
  principal or nominal annual rate and include all repayments. Repos use
  actual funding days or settlement dates, explicit fees and signed cash flows.
  Left navigation and grouped forms default to bond yield. Regular schedules
  are automatic; maturity-interest and discount bonds use explicit simple
  annual investment yields. Negative yields and three annual payments are supported.
  The shared Rust core owns calculation rules; the UI handles explicit units
  and cash-flow signs. Inputs stay in session memory. Full records can be
  copied or safely inserted without replacing selected note text. Domestic
  Wind / ChinaBond conventions await verified specifications and reference
  cases; the general mode must never be labelled as domestic valuation.
- PNG and multi-page PDF export for the current tab, including
  rendered code highlighting, KaTeX formulas, and Mermaid diagrams. PNG output
  can be saved to a file or copied directly to the system clipboard, including
  a narrower high-density mobile-sharing layout. Output follows the current
  preview theme and typography, and PDF pages use the theme background across
  the complete A4 page. PNG and PDF output ends with a `Powered by NeoPad`
  footer. Export rendering stays content-focused and loads its heavier engines
  on demand.
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
- Native CLI bundled with the app and deployed to `.agents/bin/`, with an
  Agent tools settings page for status, repair, copying its path, and editing
  the workspace Agent guide.
- Optional AI collaboration with a compact `Ctrl+K` note chat and independent
  `//` continue, polish, summarize, and Chinese-English translation commands.
  Selecting a `//` command applies generated Markdown as one undoable edit;
  chat results remain explicit insert, replace, or copy actions. Right-clicking
  an editor selection exposes direct polish, summarize, and translate actions.
  The Help menu documents setup, all three entry points, projects, and privacy.
  Ordinary note tabs also provide an explicit AI rename action that generates
  a single-line title and atomically synchronizes the tab title with the first
  Markdown level-one heading.
- Projects organize ordinary Markdown notes under `projects/<project>/`.
  The tab context menu can move a note to an existing project or subdirectory
  after saving edits, without closing the tab or opening the file browser.
  The compact F4 browser supports folders, notes, drag-and-drop movement,
  a saved draggable width, and Notes / Projects / Archive / Trash ordering,
  rename, whole-project archive, and recoverable deletion. Project notes share
  ordinary note APIs, revision checks, search, reminders, export, and CLI.
  Legacy prompt files and their trash records migrate to project notes with
  original bodies and categories preserved. The reusable prompt picker and
  prompt-only document/API types are removed.
- Structured in-app Software Help that introduces NeoPad's product direction,
  core note workflows, editor and search tools, capture and export paths,
  reminders, AI and CLI boundaries, and local-first data guarantees.
- Grouped in-app Markdown, Vim, and expression references that document
  supported rendering syntax, Vim editing and NeoPad-specific tab mappings,
  NeoPad reminders, calculation operators, examples, and unsupported
  expression behavior.
- Windows MSI packaging with branded installer assets.

## Data Rules

- Capture Markdown stays in flat `notes/`; active project Markdown lives in
  `projects/`; archived notes may be categorized under `archive/`.
- Metadata belongs in `config.json`, `meta/tabs.json`, and `meta/reminders.json`.
  Reminder content remains in note Markdown; reminder metadata stores delivery
  state only. `tabs.json` owns stable note IDs, project paths, and exact trash
  filenames. Legacy `meta/prompts.json` remains only as migration history.
- Delete operations move managed Markdown files into `trash/`, never unlink
  user content. Restoring a project note recreates its original directory.
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

## CLI Rules

- `neopad-cli` directly uses `neopad-core`; no HTTP service is required.
- Desktop startup deploys the bundled binary to `~/.neopad/.agents/bin/`.
- Deployment verifies binary contents and refuses linked deployment paths.
- Settings exposes status, repair, the CLI path, and a guide editor entry.
- Commands return JSON and documented exit codes; help and version are text.
- Reads, writes, and metadata reconciliation share the workspace process lock.
- Full note replacement requires the `updatedAt` returned by a fresh read.
- Preserve atomic writes and conflict detection. Move deleted notes to trash.
- Use IDs from the CLI; do not accept arbitrary paths for note operations.
- Project Markdown shares normal note commands, conflict checks, and search.
- `append-clipboard` appends supplied text, without reading the system clipboard.
- Refresh saved desktop notes after CLI changes without replacing unsaved edits.
- Preserve existing user-authored `.agents/` files. Create a short root
  `AGENTS.md` only when absent; never overwrite an existing guide, including
  empty files or linked entries. Keep it outside managed notes and projects.
- Read-only diagnostics expose guide path and availability without initializing
  the workspace. Skills are not automatically installed.
- Old MCP configuration is ignored; it is removed when config is next saved.

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
- AI has no reusable prompt library or attachment picker. Projects hold ordinary notes.
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

The Windows MSI uses English throughout its installer pages. Its application-
language selection page keeps only the Chinese-default option bilingual.
Chinese is selected by default, English is available, and the choice seeds only the first launch of a new
default workspace without replacing existing user preferences.

## Verification

Baseline verification:

```powershell
cargo test
pnpm build
pnpm tauri:build
```

For CLI-specific changes:

```powershell
cargo build -p neopad-cli --release
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
