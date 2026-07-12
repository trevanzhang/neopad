pub mod atomic_write;
pub mod config;
mod lock;
pub mod note;
pub mod path;
pub mod reminder;
pub mod search;
pub mod tab;
pub mod workspace;

pub use config::{load_config, save_config, AppConfig, McpConfig, PreviewMode, Theme, UiConfig};
pub use lock::{lock_workspace_for_write, WorkspaceWriteLock};
pub use note::{
    append_to_clipboard_note, append_to_note, archive_note, clear_trash, close_note, create_note,
    delete_note_to_trash, list_archived_notes, list_notes, list_open_notes, list_recent_notes,
    list_searchable_notes, list_trashed_notes, open_note, read_note, reconcile_note_metadata,
    rename_note, restore_note_from_trash, set_note_color, unarchive_note, write_note_atomic,
    write_note_atomic_checked, NoteContent,
};
pub use reminder::{
    claim_due_reminders, complete_due_reminders, complete_reminder, list_reminders,
    parse_reminder_line, reopen_reminder, Reminder, ReminderStatus,
};
pub use search::{search_notes, SearchResult};
pub use tab::{NoteTab, TabsState};
pub use workspace::{default_workspace_dir, ensure_workspace_layout, init_workspace, Workspace};

use anyhow::{bail, Result};
use std::path::Path;

/// Writes note content to a path explicitly selected by the user outside the
/// NeoPad workspace. Workspace-relative paths remain subject to the stricter
/// note path validation in `path`.
pub fn export_note_file(path: &Path, contents: &str) -> Result<()> {
    if !path.is_absolute() {
        bail!("export path must be absolute");
    }
    if path.file_name().is_none() {
        bail!("export path must include a file name");
    }

    atomic_write::write_atomic(path, contents)
}

#[cfg(test)]
mod export_tests {
    use super::*;

    #[test]
    fn export_note_file_writes_selected_absolute_path() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("exported.md");

        export_note_file(&path, "# Exported\n").expect("export note");

        assert_eq!(
            std::fs::read_to_string(path).expect("read export"),
            "# Exported\n"
        );
    }

    #[test]
    fn export_note_file_rejects_relative_paths() {
        let error = export_note_file(Path::new("exported.md"), "content")
            .expect_err("relative export path must fail");

        assert!(error.to_string().contains("must be absolute"));
    }
}
