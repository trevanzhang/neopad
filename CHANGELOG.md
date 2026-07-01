# Changelog

NeoPad follows Semantic Versioning. While the project remains below `1.0.0`,
minor versions may include user-visible behavior changes and new capabilities;
patch versions are reserved for compatible fixes.

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
