# Changelog

NeoPad follows Semantic Versioning. While the project remains below `1.0.0`,
minor versions may include user-visible behavior changes and new capabilities;
patch versions are reserved for compatible fixes.

## Unreleased

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
