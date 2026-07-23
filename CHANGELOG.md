# Changelog

NeoPad follows Semantic Versioning. While the project remains below `1.0.0`,
minor versions may include user-visible behavior changes and new capabilities;
patch versions are reserved for compatible fixes.

## Unreleased

### Changed

- Simplified the current-note export menus to one PNG submenu and one PDF
  command. All PNG and PDF exports now follow the active preview theme and
  typography, with the mobile clipboard layout remaining available.

## 0.6.2 - 2026-07-22

### Added

- Added `Powered by NeoPad` branding to the end of exported PNG images and PDF
  documents without changing the source Markdown.
- Added a high-density mobile PNG layout for copying phone-friendly long images
  directly to the system clipboard.
- Added preview-style PNG and PDF export using the active preview palette,
  typography, and full-page PDF background, while preserving the existing
  white GitHub Light print output.

### Changed

- Labeled export choices as Print and Preview for a shorter, clearer menu.

### Fixed

- Updated the transitive `linkify-it` dependency to the patched `5.0.2`
  release to address a quadratic-complexity denial-of-service advisory.

## 0.6.0 - 2026-07-15

### Added

- Added prompt Markdown management to the `F4` file browser, including create,
  edit in marked main-editor tabs, rename, duplicate, reveal, trash, and restore
  workflows with atomic saves and external-change conflict checks.
- Added nested archive and prompt folders with safe rename, delete, and drag-and-drop
  organization while preserving stable document identities.
- Added a bilingual Windows installer page for choosing Chinese or English as
  NeoPad's first-launch language, defaulting to Chinese.
- Added a Vim quick guide and reorganized the Help menu for easier discovery.
- Added View > Fullscreen with the existing `F11` shortcut.
- Added direct PNG export to the system clipboard alongside file export.

### Changed

- Archived notes can now be opened and edited in place without restoring them.
- The File menu now opens the complete `~/.neopad/` data directory instead of
  exposing an archive-only folder action.
- Reorganized PNG export into file and clipboard submenu actions in both the
  File menu and tab context menu.
- Replaced the retired npm audit endpoint in CI and release preflight with an
  exact production-dependency check against npm's Bulk Advisory API.

### Fixed

- Added the missing Redo command to the Edit menu and documented `Ctrl+Y`.
- Isolated undo and redo history when loading or switching notes so undoing an
  edit cannot restore startup placeholder text or content from another note.
- Made a left click on the system tray icon restore and focus NeoPad while a
  right click continues to open the tray menu.
- Fixed PNG clipboard export on Windows by writing native image data to the
  operating-system clipboard.

## 0.5.3 - 2026-07-14

### Added

- Added a selection-aware editor context menu with standard clipboard actions
  and direct AI polish, summarize, and translate commands.
- Added an in-app AI collaboration guide covering setup, quick commands,
  selection actions, note chat, prompt files, and privacy boundaries.
- Rebuilt the in-app Software Help as a structured product overview covering
  notes, editing, search, reminders, capture, export, AI, MCP, data safety, and
  NeoPad's deliberately lightweight scope.
- Expanded the Markdown and Expression guides into grouped reference tables
  with NeoPad-specific rendering syntax, calculation examples, and clear
  behavior boundaries.
- Added optional inline AI editing with secure OpenAI-compatible provider
  settings, a compact `Ctrl+K` note chat, in-memory per-note sessions, reusable
  Markdown prompts, bounded all-note context search, explicit chat result
  actions, and independent auto-applying `//` commands with cursor-anchored
  progress for continuing, polishing, summarizing, and Chinese-English
  translation.

### Changed

- Reorganized the in-app shortcut reference into grouped tables with clearer
  descriptions and the current shortcut set.
- Slash commands now separate their precise edit target from current-note
  reference context, using bounded target-aware context for long notes.

- Emptying NeoPad's Trash now asks for confirmation and moves Markdown files
  to the operating system's Recycle Bin or Trash instead of maintaining a
  second application backup directory. Files restored by the operating system
  are rediscovered in NeoPad's Trash on refresh.
- Localized the current-note find and replace panel so its controls follow the
  selected application language.

### Fixed

- Prevented Escape from hiding the NeoPad window after closing an open Help
  dialog, including when duplicate same-target listeners exist during hot reload.
- Improved text-selection contrast in both light and dark editor themes.
- Prevented an older global-search request from replacing results returned by
  a newer query.
- Ignored application shortcuts during IME composition and disposed frontend
  listeners and composable state when the application view is unmounted.
- Required MCP bearer authentication before JSON parsing, limited request
  bodies to 2 MiB, and kept malformed-request responses consistent.
- Ensured the desktop-owned MCP sidecar is stopped and reaped when disabled or
  when the desktop application exits.

## 0.5.2 - 2026-07-13

### Added

- Added Markdown source and fenced-code syntax highlighting, KaTeX formulas,
  and Mermaid diagrams to the editor and preview workflow.
- Added current-note PNG and PDF export actions to the File menu and tab
  context menu, with atomic binary writes through `neopad-core`.
- Added a tab action that copies the current Markdown file's absolute path.

### Changed

- Clarified the File menu archive action and made it open the archive directory
  in the system file manager.

## 0.5.1 - 2026-07-13

### Added

- Added contextual Explorer reveal actions for tabs and single note-library selections.
- Added a startup recovery prompt for a note write preserved after a failed
  atomic replacement.

### Changed

- Reorganized high-frequency shortcuts for tab navigation, editor modes,
  reminders, window pinning, renaming, and preview themes, including previous
  and next page entries in the View menu.
- Moved the expression-calculation cursor to the next line after a successful result.
- Retried temporary Windows file locks during atomic note writes and retained
  pending content if replacement remains blocked.

## 0.5.0 - 2026-07-12

### Added

- Added the `F4` note library sidebar for active, archived, and trashed notes,
  including direct restore, archive, rename, and trash actions.
- Added a reorganized Settings center with focused General, Editor, Preview,
  Vim, Shortcuts, and MCP sections, including configurable global shortcuts.
- Added native drag-and-drop opening for `.md` and `.markdown` files, plus
  desktop file-association declarations so NeoPad can be selected as their
  default editor.
- Added cross-platform release builds via GitHub Actions. Pushing a `v*` tag
  now produces Windows `.msi`, macOS `.dmg` (ARM64), and Linux `.deb` +
  `.AppImage` installers automatically.
- Added release preflight checks, exact artifact validation, and SHA-256
  manifests. Automated releases now remain drafts until signature and
  notarization checks are completed.
- Added production npm and RustSec dependency audit gates, plus Linux x64 and
  macOS ARM64 workspace compilation on pull requests.

### Changed

- Modernized the compact desktop UI with unified color tokens, a refreshed
  application chrome, improved tab colors, responsive dialog layouts, clearer
  focus states, and consistent icon-only controls.
- Consolidated font, input, confirmation, reminder, and help dialogs behind a
  shared visual layer, keeping long content within the viewport.
- Refactored the frontend composition root into focused document, note
  lifecycle, preferences, native settings, search, reminder, archive, MCP, and
  dialog modules while preserving existing UI and data behavior.
- Moved keyboard shortcut priority routing behind a pure, characterized module
  while retaining the existing modal, Vim, menu, and Escape precedence.
- Extracted window lifecycle, help content, and text transformations; added
  regression vectors for MD5, localized help, and shortcut routing.
- Reduced the CodeMirror editor component to its live instance and command
  responsibilities by extracting search-panel UI, themes, and line-calculation
  logic, with regression coverage for match navigation and expression parsing.
- External Markdown conflict checks now use content revisions instead of
  millisecond modification times; files opened through the picker, drag and
  drop, or an operating-system association are approved before reopening.
- Desktop-managed MCP tokens are passed through the child environment instead
  of command-line arguments, and the HTTP service only accepts loopback binds.
- Platform builds now request explicit bundle types so Windows builds no
  longer generate an unintended NSIS installer.
- Preview, settings, reminder, archive, and dialog surfaces are loaded on
  demand, reducing the initial JavaScript bundle by about 130 kB.

### Fixed

- Prevented page switches and content-replacing actions when autosave fails,
  preserving the pending editor snapshot for retry.
- Preserved the newest queued edit when an in-flight autosave fails, with a
  regression test covering the explicit retry path.
- Prevented failed native note creation from falling back to an unsavable
  browser-only draft, and synchronized external-file revisions after reload.
- Made note conflict versions strictly monotonic, including rapid writes in
  the same millisecond.
- Removed implicit, unlocked metadata writes from note-list queries and added
  locked reconciliation at desktop and MCP boundaries.
- Added rollback for note content and file moves when the matching metadata
  update fails.
- Restored a clean Clippy `-D warnings` build for the MCP authentication path.
- Upgraded the plist XML parser path to `quick-xml 0.41.0`, removing two
  high-severity RustSec findings from runtime-reachable dependencies.

## 0.4.6 - 2026-07-10

### Added

- Added a dedicated archive browser (File > View Archive) that lists
  archived notes and restores them back into the active workspace.
- Added tab-context keyboard shortcuts: `F2` renames, `Del` moves to
  trash, `F12` archives, and `Ctrl+W` closes the focused tab directly
  from the tab bar or its context menu.
- Added a visible version string to the About status so the running build
  version is shown in-app.

### Changed

- Global search now groups matching lines by note, shows a per-note match
  count, and collapses repeated matches with on-demand expansion. Results
  are sorted with title-match notes first, and a "Load more results" pager
  grows the backend limit in batches.
- Page-action menus and context menus now display their keyboard shortcut
  hints inline.
- Removed tabs whose note files were deleted outside NeoPad are detected
  on the next refresh instead of recreating the missing file.

### Fixed

- Fixed missing-note detection so a stale tab and its recent-document
  record are cleaned up when the underlying file no longer exists.

## 0.4.5 - 2026-07-10

### Added

- Added a managed note lifecycle with close-without-delete, archive, restore,
  and recent-document workflows.
- Added `archive/` storage for NeoPad-managed notes. Archived notes remain
  searchable and are included in all-notes ZIP exports.
- Added native opening and in-place autosave for explicitly selected external
  Markdown files, including modification-time conflict checks.
- Added copy-to-archive for external Markdown files without moving or deleting
  their originals.
- Added a document marker and full-path tooltip for external-file tabs.
- Generated untitled notes now receive incrementing titles for easier tab,
  search, and recent-document identification.

### Changed

- `Ctrl+W` now closes the current non-pinned tab without changing the note
  file. Deletion remains an explicit move-to-trash action.
- Preview content is centered within its configured reading width, including in
  immersive fullscreen mode.

### Fixed

- Fixed Windows startup registration so the Run key stores a valid quoted
  executable command, allowing "Run automatically at system startup" to launch
  NeoPad after sign-in.
- Fixed external Markdown tabs when switching with `Ctrl+Tab` or
  `Ctrl+Shift+Tab`.
- Fixed the current-note find panel so `Esc` and moving focus outside the panel
  reliably close it.
- Fixed archive confirmation copy, action label, and button styling so archive
  is not presented as a destructive delete action.

## 0.4.4 - 2026-07-09

### Added

- Added Markdown preview appearance settings with Light, One Dark, Nord,
  Solarized Light, Solarized Dark, Monokai, GitHub Light, and Dracula presets.
- Added preview typography controls for font family, font size, line height,
  and content width.
- Added a dedicated editor font dialog with preset font choices, font-size
  control, and live preview.
- Added `F7` to cycle preview themes.
- Added `README_CN.md` for Chinese users.

### Changed

- The Format menu now distinguishes preview theme cycling from app day/night
  mode: `F7` cycles preview themes and `F9` toggles day/night mode.
- Editor font selection no longer requires users to type CSS font-family
  strings manually.
- Bumped distributable version to `0.4.4`.

### Fixed

- The editor font size is now persisted and reapplied through the shared UI
  configuration.

## 0.4.2 - 2026-07-09

### Added

- Added a dedicated MCP settings page with service start/stop, status, endpoint,
  token display, token regeneration, and copyable client configuration.
- Added desktop-managed Streamable HTTP MCP support at `/mcp`, protected by a
  bearer token and local browser-origin validation.
- Added MSI packaging for `neopad-mcp.exe` as the app's sidecar binary.

### Changed

- Settings now shows the editor-mode cycling shortcut as fixed `F4` instead of
  asking users to choose or disable it.
- New Markdown reminders now use the language-neutral `@remind` marker while
  keeping compatibility with existing `@提醒` reminder lines.
- NeoPad now launches into edit mode by default while keeping edit, split, and
  preview available through `F4`, the View menu, and the status bar.
- Documentation has been refreshed for the MCP HTTP service, bundled sidecar,
  edit-mode startup behavior, Markdown import behavior, and current packaging
  flow.

### Fixed

- MCP service startup no longer opens a console window on Windows.
- Older sparse `config.json` files now load with defaults for newly added UI
  fields instead of failing on missing settings.
- Native window opacity is reapplied when the window is shown or focused without
  interrupting maximize and restore transitions.

## 0.4.1 - 2026-07-08

### Changed

- Active tabs now use a yellow bottom highlight for stronger visual contrast.
- Editor mode cycling now defaults to `F4` instead of `F7`.

### Added

- `F1` opens shortcut help.
- `F2` renames the current page from the keyboard.

### Fixed

- Rename and other input dialogs now confirm reliably with Enter.
- Shortcut help opens from the editor when `F1` is pressed.

## 0.3.6 - 2026-07-08

### Added

- Reminder list filtering for all reminders, pending reminders, due reminders,
  and completed reminders.
- Completed reminders can now be marked unfinished directly from the reminder
  list.
- Vim Normal-mode `gt` and `gT` now switch to the next and previous NeoPad tab.

### Changed

- Reminder list actions now use clearer labels: "Mark Completed" and
  "Mark Unfinished".
- The all-notes export now writes a `.zip` archive with one Markdown file per
  tab instead of combining every note into a single Markdown document.
- Reminder list controls now share a more consistent compact button style.

### Fixed

- Closing a tab with `Ctrl+W` now handles the active-tab switch before trashing
  the note, avoiding stale UI tabs and failed repeated delete attempts.
- Tab metadata now reconciles notes that were already moved to `trash/`, so
  deleted tabs do not remain visible after restart or refresh.
- Delete confirmation dialogs now focus the delete button by default and support
  keyboard-only confirmation, cancellation, and focus cycling.
- Opening global search from its shortcut now focuses and selects the search
  field immediately.

## 0.3.0 - 2026-07-03

### Added

- Markdown-native reminders in the form
  `- [ ] @提醒 YYYY-MM-DD HH:mm content`, with a compact `Ctrl+E` editor,
  dedicated status-sorted list, source-line navigation, completion through
  standard Markdown checkboxes, and native notifications while NeoPad runs.
- Persisted reminder delivery state prevents due notifications from repeating
  after application restart.
- `F5` toggles the reminder list, which provides per-row completion and a
  "Clear Due" batch action without deleting Markdown reminder lines.

- `Ctrl+Tab` and `Ctrl+Shift+Tab` cycle through note tabs; the tab bar arrow
  buttons now provide the same previous/next actions for mouse users.
- `F9` toggles the light/dark theme, while `F11` enters an immersive native
  fullscreen mode that shows only the editor; Escape exits immersion first.
- `Alt+Enter` toggles the main window between maximized and restored states.
- The global clipboard capture shortcut is now configurable in Settings,
  defaulting to `Ctrl+Shift+V`.
- Persistent light and dark themes switchable from the Format menu and status
  bar.
- Custom tab context menu with rename, trash, and persistent tab colors.
- Optional Vim key bindings for the Markdown editor with persisted settings,
  mode status, and Vim-first shortcut handling while the editor is focused.
- Configurable Insert-mode exit sequence, defaulting to `jj`, while retaining
  the standard `Esc` binding.
- A dedicated Advanced settings tab for optional Vim editing controls.

### Fixed

- Browser-native text prompts have been replaced by a compact, theme-aware
  NeoPad input dialog for tab titles, editor fonts, and custom insert text.
- The web content no longer applies a second rounded mask inside the native
  Windows frame, removing visible gaps at all four window corners.
- Escape now closes settings, menus, search, help, and tab context menus before
  falling back to hiding the main window.
- Menu popovers now size to their command labels and shortcut columns so long
  localized labels cannot overlap shortcut text.
- Edge snapping no longer interrupts `Alt+Enter` while Windows is maximizing
  the main window.
- Vim Insert-mode carets now use the theme-aware cursor color instead of the
  CodeMirror default black caret on dark backgrounds.
- Window opacity now controls the native Windows window through Tauri instead
  of fading only the web content; the Format menu opens the existing slider.
- Vim block cursors remain clearly visible in both light and dark themes.
- Settings remain fully accessible at the default window size by scrolling only
  the settings content while keeping its header, tabs, and confirmation fixed.
- The status-bar theme button now shows the destination action: moon to enter
  dark mode and sun to enter light mode.
- Date-time separators now include trailing dashes and respond to
  `Ctrl+Shift+-`.
- Application menus support Alt mnemonics and keyboard navigation.

## 0.2.0 - 2026-07-01

### Added

- Edit, hybrid, and preview editor modes with View menu selection, status-bar
  switching, persisted defaults, and configurable cycling shortcut.
- Working page rename and trash actions with core protection for the Inbox and
  Clipboard default pages.
- Chinese display names for system pages and generated untitled pages without
  changing their storage IDs or user-defined titles.
- Automated version consistency checking across Cargo, Tauri, and package
  manifests.

### Changed

- Simplified the View menu into editor mode and tab position submenus.
- Shortened utility menu labels and clarified status-bar mode labels.
- Updated the MSI banner to avoid collisions with native WiX text.

## 0.1.0

- Initial Windows-focused MVP.
