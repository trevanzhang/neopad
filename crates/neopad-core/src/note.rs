use crate::atomic_write::write_atomic;
use crate::path::{note_file_path, trash_file_path};
use crate::{ensure_workspace_layout, NoteTab, TabsState, Workspace};
use anyhow::{bail, Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoteContent {
    pub id: String,
    pub title: String,
    pub file_name: String,
    pub content: String,
    pub updated_at: i64,
}

pub fn list_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    let mut tabs = load_tabs(workspace)?;
    if reconcile_trashed_tabs(workspace, &mut tabs)? {
        save_tabs(workspace, &tabs)?;
    }
    Ok(tabs.tabs.into_iter().filter(|tab| !tab.deleted).collect())
}

pub fn read_note(workspace: &Workspace, note_id: &str) -> Result<NoteContent> {
    let tab = find_note_tab(workspace, note_id)?;
    let path = note_file_path(workspace, &tab.file_name)?;
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read note file {}", path.display()))?;

    Ok(NoteContent {
        id: tab.id,
        title: tab.title,
        file_name: tab.file_name,
        content,
        updated_at: tab.updated_at,
    })
}

pub fn create_note(workspace: &Workspace, title: Option<String>) -> Result<NoteContent> {
    ensure_workspace_layout(workspace)?;
    let mut tabs = load_tabs(workspace)?;
    let now = now_ms()?;
    let id = unique_note_id(&tabs, now);
    let system_title = title.as_ref().is_none_or(|value| value.trim().is_empty());
    let title = normalized_title(title);
    let file_name = format!("{id}.md");
    let content = format!("# {title}\n\n");
    let path = note_file_path(workspace, &file_name)?;

    write_atomic(&path, &content)?;

    tabs.tabs.push(NoteTab {
        id: id.clone(),
        title: title.clone(),
        file_name: file_name.clone(),
        created_at: now,
        updated_at: now,
        pinned: false,
        deleted: false,
        system_title,
        color: None,
    });
    tabs.active_tab_id = id.clone();
    save_tabs(workspace, &tabs)?;

    Ok(NoteContent {
        id,
        title,
        file_name,
        content,
        updated_at: now,
    })
}

pub fn write_note_atomic(
    workspace: &Workspace,
    note_id: &str,
    content: &str,
) -> Result<NoteContent> {
    let mut tabs = load_tabs(workspace)?;
    let tab = find_note_tab_mut(&mut tabs, note_id)?;
    let path = note_file_path(workspace, &tab.file_name)?;
    let now = now_ms()?;

    write_atomic(&path, content)?;
    tab.updated_at = now;

    let note = NoteContent {
        id: tab.id.clone(),
        title: tab.title.clone(),
        file_name: tab.file_name.clone(),
        content: content.to_owned(),
        updated_at: now,
    };
    save_tabs(workspace, &tabs)?;

    Ok(note)
}

pub fn write_note_atomic_checked(
    workspace: &Workspace,
    note_id: &str,
    content: &str,
    expected_updated_at: i64,
) -> Result<NoteContent> {
    let current = read_note(workspace, note_id)?;
    if current.updated_at != expected_updated_at {
        bail!(
            "note was modified: expected updated_at {}, current updated_at {}",
            expected_updated_at,
            current.updated_at
        );
    }

    write_note_atomic(workspace, note_id, content)
}

pub fn append_to_note(workspace: &Workspace, note_id: &str, content: &str) -> Result<NoteContent> {
    if content.is_empty() {
        return read_note(workspace, note_id);
    }

    let mut note = read_note(workspace, note_id)?;
    if !note.content.ends_with('\n') {
        note.content.push('\n');
    }
    note.content.push_str(content);
    write_note_atomic(workspace, note_id, &note.content)
}

pub fn append_to_clipboard_note(
    workspace: &Workspace,
    clipboard_text: &str,
) -> Result<NoteContent> {
    let clipboard_text = clipboard_text.trim();
    if clipboard_text.is_empty() {
        return read_note(workspace, "clipboard");
    }

    let entry = format!("\n{}\n\n{}\n", timestamp_separator(), clipboard_text);
    append_to_note(workspace, "clipboard", &entry)
}

pub fn rename_note(workspace: &Workspace, note_id: &str, title: String) -> Result<NoteTab> {
    if matches!(note_id, "inbox" | "clipboard") {
        bail!("{note_id} is pinned and cannot be renamed");
    }

    let mut tabs = load_tabs(workspace)?;
    let tab = find_note_tab_mut(&mut tabs, note_id)?;
    let now = now_ms()?;

    tab.title = normalized_title(Some(title));
    tab.system_title = false;
    tab.updated_at = now;
    let renamed = tab.clone();
    save_tabs(workspace, &tabs)?;

    Ok(renamed)
}

pub fn delete_note_to_trash(workspace: &Workspace, note_id: &str) -> Result<NoteTab> {
    if matches!(note_id, "inbox" | "clipboard") {
        bail!("{note_id} is pinned and cannot be deleted");
    }

    let mut tabs = load_tabs(workspace)?;
    let tab = find_note_tab_mut(&mut tabs, note_id)?;
    let source = note_file_path(workspace, &tab.file_name)?;
    let deleted_file_name = format!("deleted-{}-{}", now_ms()?, tab.file_name);
    let target = trash_file_path(workspace, &deleted_file_name)?;

    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to move note {} to trash {}",
            source.display(),
            target.display()
        )
    })?;

    tab.deleted = true;
    tab.updated_at = now_ms()?;
    let deleted = tab.clone();
    save_tabs(workspace, &tabs)?;

    Ok(deleted)
}

pub fn set_note_color(
    workspace: &Workspace,
    note_id: &str,
    color: Option<String>,
) -> Result<NoteTab> {
    let color = color
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty());
    if let Some(value) = &color {
        let valid = value.len() == 7
            && value.starts_with('#')
            && value[1..]
                .chars()
                .all(|character| character.is_ascii_hexdigit());
        if !valid {
            bail!("invalid tab color: {value}");
        }
    }

    let mut tabs = load_tabs(workspace)?;
    let tab = find_note_tab_mut(&mut tabs, note_id)?;
    tab.color = color;
    tab.updated_at = now_ms()?;
    let updated = tab.clone();
    save_tabs(workspace, &tabs)?;
    Ok(updated)
}

fn load_tabs(workspace: &Workspace) -> Result<TabsState> {
    ensure_workspace_layout(workspace)?;
    let contents = fs::read_to_string(&workspace.tabs_path)
        .with_context(|| format!("failed to read tabs file {}", workspace.tabs_path.display()))?;
    serde_json::from_str(&contents).with_context(|| {
        format!(
            "failed to parse tabs file {}",
            workspace.tabs_path.display()
        )
    })
}

fn save_tabs(workspace: &Workspace, tabs: &TabsState) -> Result<()> {
    let contents = serde_json::to_string_pretty(tabs).context("failed to serialize tabs")?;
    write_atomic(&workspace.tabs_path, &contents)
}

fn reconcile_trashed_tabs(workspace: &Workspace, tabs: &mut TabsState) -> Result<bool> {
    let mut changed = false;
    let trash_file_names = fs::read_dir(&workspace.trash_dir)
        .with_context(|| format!("failed to read trash dir {}", workspace.trash_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    for tab in &mut tabs.tabs {
        if tab.deleted || tab.pinned {
            continue;
        }

        let note_path = note_file_path(workspace, &tab.file_name)?;
        let has_matching_trash_file = trash_file_names.iter().any(|file_name| {
            file_name.starts_with("deleted-") && file_name.ends_with(&tab.file_name)
        });
        if !note_path.exists() && has_matching_trash_file {
            tab.deleted = true;
            tab.updated_at = now_ms()?;
            changed = true;
        }
    }

    let active_is_available = tabs
        .tabs
        .iter()
        .any(|tab| tab.id == tabs.active_tab_id && !tab.deleted);
    if !active_is_available {
        if let Some(next_tab) = tabs.tabs.iter().find(|tab| !tab.deleted) {
            tabs.active_tab_id = next_tab.id.clone();
            changed = true;
        }
    }

    Ok(changed)
}

fn find_note_tab(workspace: &Workspace, note_id: &str) -> Result<NoteTab> {
    let tabs = load_tabs(workspace)?;
    tabs.tabs
        .into_iter()
        .find(|tab| tab.id == note_id && !tab.deleted)
        .with_context(|| format!("note not found: {note_id}"))
}

fn find_note_tab_mut<'a>(tabs: &'a mut TabsState, note_id: &str) -> Result<&'a mut NoteTab> {
    tabs.tabs
        .iter_mut()
        .find(|tab| tab.id == note_id && !tab.deleted)
        .with_context(|| format!("note not found: {note_id}"))
}

fn normalized_title(title: Option<String>) -> String {
    let title = title.unwrap_or_else(|| "Untitled".to_owned());
    let title = title.trim();
    if title.is_empty() {
        "Untitled".to_owned()
    } else {
        title.to_owned()
    }
}

fn unique_note_id(tabs: &TabsState, now: i64) -> String {
    let mut suffix = 0;
    loop {
        let id = if suffix == 0 {
            format!("page-{now}")
        } else {
            format!("page-{now}-{suffix}")
        };

        if tabs.tabs.iter().all(|tab| tab.id != id) {
            return id;
        }

        suffix += 1;
    }
}

fn now_ms() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before unix epoch")?;
    Ok(duration.as_millis() as i64)
}

fn timestamp_separator() -> String {
    format_timestamp_separator(&Local::now().format("%Y-%m-%d %H:%M").to_string())
}

fn format_timestamp_separator(timestamp: &str) -> String {
    let rule = "-".repeat(29);
    format!("{rule} {timestamp} {rule}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_workspace;

    #[test]
    fn note_crud_round_trip_uses_markdown_files_and_tabs() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");

        let created = create_note(&workspace, Some("Work".to_owned())).expect("create");
        assert_eq!(created.title, "Work");
        assert!(created.file_name.ends_with(".md"));
        assert_eq!(
            fs::read_to_string(note_file_path(&workspace, &created.file_name).expect("path"))
                .expect("created file"),
            "# Work\n\n"
        );

        let notes = list_notes(&workspace).expect("list");
        assert!(notes.iter().any(|tab| tab.id == created.id));

        let written =
            write_note_atomic(&workspace, &created.id, "# Work\n\nUpdated").expect("write");
        assert_eq!(written.content, "# Work\n\nUpdated");

        let read = read_note(&workspace, &created.id).expect("read");
        assert_eq!(read.content, "# Work\n\nUpdated");

        let renamed = rename_note(&workspace, &created.id, "Renamed".to_owned()).expect("rename");
        assert_eq!(renamed.title, "Renamed");

        let deleted = delete_note_to_trash(&workspace, &created.id).expect("delete");
        assert!(deleted.deleted);
        assert!(list_notes(&workspace)
            .expect("list after delete")
            .iter()
            .all(|tab| tab.id != created.id));
        assert!(fs::read_dir(&workspace.trash_dir)
            .expect("trash dir")
            .any(|entry| entry
                .expect("trash entry")
                .file_name()
                .to_string_lossy()
                .contains(&created.file_name)));
    }

    #[test]
    fn pinned_default_notes_cannot_be_deleted() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");

        assert!(delete_note_to_trash(&workspace, "inbox").is_err());
        assert!(delete_note_to_trash(&workspace, "clipboard").is_err());
        assert!(rename_note(&workspace, "inbox", "Other".to_owned()).is_err());
        assert!(rename_note(&workspace, "clipboard", "Other".to_owned()).is_err());
        assert!(workspace.inbox_path().is_file());
        assert!(workspace.clipboard_path().is_file());
    }

    #[test]
    fn list_notes_reconciles_tabs_when_note_was_already_moved_to_trash() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, None).expect("create note");
        let source = note_file_path(&workspace, &created.file_name).expect("note path");
        let target = trash_file_path(
            &workspace,
            &format!("deleted-{}-{}", now_ms().expect("now"), created.file_name),
        )
        .expect("trash path");
        fs::rename(source, target).expect("move to trash without updating tabs");

        let listed = list_notes(&workspace).expect("list");

        assert!(listed.iter().all(|tab| tab.id != created.id));
        let tabs = load_tabs(&workspace).expect("tabs");
        assert!(
            tabs.tabs
                .iter()
                .find(|tab| tab.id == created.id)
                .expect("created tab")
                .deleted
        );
    }

    #[test]
    fn generated_untitled_marker_is_cleared_by_rename() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");

        let created = create_note(&workspace, None).expect("create untitled note");
        let created_tab = list_notes(&workspace)
            .expect("list")
            .into_iter()
            .find(|tab| tab.id == created.id)
            .expect("created tab");
        assert!(created_tab.system_title);

        let renamed = rename_note(&workspace, &created.id, "Untitled".to_owned()).expect("rename");
        assert!(!renamed.system_title);
    }

    #[test]
    fn tab_color_is_validated_and_persisted() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, None).expect("create note");

        let colored =
            set_note_color(&workspace, &created.id, Some("#a9dceb".to_owned())).expect("set color");
        assert_eq!(colored.color.as_deref(), Some("#A9DCEB"));
        assert_eq!(
            list_notes(&workspace)
                .expect("list")
                .into_iter()
                .find(|tab| tab.id == created.id)
                .and_then(|tab| tab.color),
            Some("#A9DCEB".to_owned())
        );
        assert!(set_note_color(&workspace, &created.id, Some("blue".to_owned())).is_err());
    }

    #[test]
    fn append_to_clipboard_note_adds_separator_and_ignores_empty_text() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");

        let unchanged = append_to_clipboard_note(&workspace, "   ").expect("empty append");
        assert_eq!(unchanged.content, "# Clipboard\n\n");

        let updated = append_to_clipboard_note(&workspace, "copied text").expect("append");

        assert!(!updated.content.contains("\n---\n"));
        assert!(updated.content.contains("----------------------------- 20"));
        assert!(updated.content.contains("copied text"));
    }

    #[test]
    fn clipboard_timestamp_separator_is_fixed_width_and_readable() {
        assert_eq!(
            format_timestamp_separator("2026-07-02 05:47"),
            "----------------------------- 2026-07-02 05:47 -----------------------------"
        );
    }

    #[test]
    fn write_note_atomic_checked_rejects_stale_timestamp() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let note = create_note(&workspace, Some("Checked".to_owned())).expect("create");

        assert!(
            write_note_atomic_checked(&workspace, &note.id, "new", note.updated_at - 1).is_err()
        );
    }
}
