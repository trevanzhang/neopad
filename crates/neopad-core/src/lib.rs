pub mod atomic_write;
pub mod config;
pub mod note;
pub mod path;
pub mod search;
pub mod tab;
pub mod workspace;

pub use config::{AppConfig, McpConfig, PreviewMode, Theme};
pub use note::{
    append_to_clipboard_note, append_to_note, create_note, delete_note_to_trash, list_notes,
    read_note, rename_note, write_note_atomic, write_note_atomic_checked, NoteContent,
};
pub use search::{search_notes, SearchResult};
pub use tab::{NoteTab, TabsState};
pub use workspace::{default_workspace_dir, ensure_workspace_layout, init_workspace, Workspace};
