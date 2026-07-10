use crate::atomic_write::write_atomic;
use crate::path::{archive_file_path, note_file_path, trash_file_path};
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
    Ok(tabs
        .tabs
        .into_iter()
        .filter(|tab| !tab.deleted && !tab.archived)
        .collect())
}

pub fn list_archived_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    let mut tabs = load_tabs(workspace)?;
    if reconcile_trashed_tabs(workspace, &mut tabs)? {
        save_tabs(workspace, &tabs)?;
    }
    Ok(tabs
        .tabs
        .into_iter()
        .filter(|tab| !tab.deleted && tab.archived)
        .collect())
}

pub fn list_open_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    let mut tabs = load_tabs(workspace)?;
    if reconcile_trashed_tabs(workspace, &mut tabs)? {
        save_tabs(workspace, &tabs)?;
    }
    Ok(tabs
        .tabs
        .into_iter()
        .filter(|tab| !tab.deleted && tab.open)
        .collect())
}

pub fn list_searchable_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    let mut tabs = load_tabs(workspace)?;
    if reconcile_trashed_tabs(workspace, &mut tabs)? {
        save_tabs(workspace, &tabs)?;
    }
    Ok(tabs.tabs.into_iter().filter(|tab| !tab.deleted).collect())
}

pub fn list_recent_notes(workspace: &Workspace, limit: usize) -> Result<Vec<NoteTab>> {
    let mut tabs = load_tabs(workspace)?;
    if reconcile_trashed_tabs(workspace, &mut tabs)? {
        save_tabs(workspace, &tabs)?;
    }
    let mut notes = tabs
        .tabs
        .into_iter()
        .filter(|tab| !tab.deleted)
        .collect::<Vec<_>>();
    notes.sort_by_key(|tab| std::cmp::Reverse(tab.last_opened_at.unwrap_or(tab.updated_at)));
    notes.truncate(limit);
    Ok(notes)
}

pub fn read_note(workspace: &Workspace, note_id: &str) -> Result<NoteContent> {
    let tab = find_note_tab(workspace, note_id)?;
    let path = note_path(workspace, &tab)?;
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
        archived: false,
        open: true,
        last_opened_at: Some(now),
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
    let path = note_path(workspace, tab)?;
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

/// Writes a note only if its current `updated_at` matches `expected_updated_at`.
///
/// Callers must hold `lock_workspace_for_write` for the full check-and-write
/// operation; this function deliberately does not acquire the process lock
/// itself so callers can compose multiple workspace writes under one lock.
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
    let previous_title = tab.title.clone();
    let next_title = normalized_title(Some(title));

    if tab.system_title {
        let path = note_path(workspace, tab)?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read note file {}", path.display()))?;
        if let Some(updated_content) =
            renamed_default_heading(&content, &previous_title, &next_title)
        {
            write_atomic(&path, &updated_content)?;
        }
    }

    tab.title = next_title;
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
    let source = note_path(workspace, tab)?;
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
    tab.open = false;
    tab.updated_at = now_ms()?;
    let deleted = tab.clone();
    save_tabs(workspace, &tabs)?;

    Ok(deleted)
}

pub fn close_note(workspace: &Workspace, note_id: &str) -> Result<NoteTab> {
    if matches!(note_id, "inbox" | "clipboard") {
        bail!("{note_id} is pinned and cannot be closed");
    }
    let mut tabs = load_tabs(workspace)?;
    let tab = find_note_tab_mut(&mut tabs, note_id)?;
    tab.open = false;
    let closed = tab.clone();
    save_tabs(workspace, &tabs)?;
    Ok(closed)
}

pub fn open_note(workspace: &Workspace, note_id: &str) -> Result<NoteTab> {
    let mut tabs = load_tabs(workspace)?;
    let tab_index = tabs
        .tabs
        .iter()
        .position(|tab| tab.id == note_id && !tab.deleted)
        .with_context(|| format!("note not found: {note_id}"))?;
    let path = note_path(workspace, &tabs.tabs[tab_index])?;
    if !path.is_file() {
        if !tabs.tabs[tab_index].pinned {
            tabs.tabs[tab_index].deleted = true;
            tabs.tabs[tab_index].open = false;
            tabs.tabs[tab_index].updated_at = now_ms()?;
            if tabs.active_tab_id == note_id {
                if let Some(next_tab) = tabs.tabs.iter().find(|tab| !tab.deleted) {
                    tabs.active_tab_id = next_tab.id.clone();
                }
            }
            save_tabs(workspace, &tabs)?;
        }
        bail!("note file does not exist: {}", path.display());
    }

    let tab = &mut tabs.tabs[tab_index];
    tab.open = true;
    tab.last_opened_at = Some(now_ms()?);
    let opened = tab.clone();
    save_tabs(workspace, &tabs)?;
    Ok(opened)
}

pub fn archive_note(workspace: &Workspace, note_id: &str) -> Result<NoteTab> {
    if matches!(note_id, "inbox" | "clipboard") {
        bail!("{note_id} is pinned and cannot be archived");
    }
    let mut tabs = load_tabs(workspace)?;
    let tab = find_note_tab_mut(&mut tabs, note_id)?;
    let source = note_path(workspace, tab)?;
    let target = archive_file_path(workspace, &tab.file_name)?;
    fs::rename(&source, &target)
        .with_context(|| format!("failed to archive note {}", source.display()))?;
    tab.archived = true;
    tab.open = false;
    tab.updated_at = now_ms()?;
    let archived = tab.clone();
    save_tabs(workspace, &tabs)?;
    Ok(archived)
}

pub fn unarchive_note(workspace: &Workspace, note_id: &str) -> Result<NoteTab> {
    let mut tabs = load_tabs(workspace)?;
    let tab = find_note_tab_mut(&mut tabs, note_id)?;
    if !tab.archived {
        bail!("note is not archived: {note_id}");
    }
    let source = archive_file_path(workspace, &tab.file_name)?;
    let target = note_file_path(workspace, &tab.file_name)?;
    fs::rename(&source, &target)
        .with_context(|| format!("failed to restore archived note {}", source.display()))?;
    tab.archived = false;
    tab.open = true;
    tab.last_opened_at = Some(now_ms()?);
    tab.updated_at = now_ms()?;
    let restored = tab.clone();
    save_tabs(workspace, &tabs)?;
    Ok(restored)
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

    for tab in &mut tabs.tabs {
        if tab.deleted || tab.pinned {
            continue;
        }

        if tab.system_title
            && tab
                .title
                .strip_prefix("Untitled ")
                .is_some_and(|suffix| suffix.parse::<u32>().is_ok())
        {
            tab.title = "Untitled".to_owned();
            changed = true;
        }

        let note_path = note_path(workspace, tab)?;
        if !note_path.is_file() {
            tab.deleted = true;
            tab.open = false;
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

fn note_path(workspace: &Workspace, tab: &NoteTab) -> Result<std::path::PathBuf> {
    if tab.archived {
        archive_file_path(workspace, &tab.file_name)
    } else {
        note_file_path(workspace, &tab.file_name)
    }
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

fn renamed_default_heading(
    content: &str,
    previous_title: &str,
    next_title: &str,
) -> Option<String> {
    let previous_heading = format!("# {previous_title}");
    let remainder = content.strip_prefix(&previous_heading)?;
    if !(remainder.is_empty() || remainder.starts_with('\n') || remainder.starts_with("\r\n")) {
        return None;
    }
    Some(format!("# {next_title}{remainder}"))
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
    fn opening_a_manually_deleted_note_removes_it_from_recent_notes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, None).expect("create note");
        let source = note_file_path(&workspace, &created.file_name).expect("note path");
        fs::remove_file(source).expect("remove note file");

        let error =
            open_note(&workspace, &created.id).expect_err("opening a missing note must fail");
        assert!(error.to_string().contains("note file does not exist"));
        assert!(list_recent_notes(&workspace, 20)
            .expect("recent notes")
            .iter()
            .all(|tab| tab.id != created.id));
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
    fn renaming_a_generated_note_updates_its_default_heading() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, None).expect("create note");

        let renamed =
            rename_note(&workspace, &created.id, "Project notes".to_owned()).expect("rename");

        assert_eq!(renamed.title, "Project notes");
        assert!(!renamed.system_title);
        assert_eq!(
            read_note(&workspace, &created.id)
                .expect("read note")
                .content,
            "# Project notes\n\n"
        );
    }

    #[test]
    fn renaming_a_custom_note_keeps_its_heading() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created =
            create_note(&workspace, Some("Original heading".to_owned())).expect("create note");

        rename_note(&workspace, &created.id, "New tab title".to_owned()).expect("rename");

        assert_eq!(
            read_note(&workspace, &created.id)
                .expect("read note")
                .content,
            "# Original heading\n\n"
        );
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
    fn generated_untitled_notes_use_the_default_title() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");

        let first = create_note(&workspace, None).expect("first note");
        let second = create_note(&workspace, None).expect("second note");

        assert_eq!(first.title, "Untitled");
        assert_eq!(second.title, "Untitled");
        assert_ne!(first.id, second.id);
        assert_ne!(first.file_name, second.file_name);
    }

    #[test]
    fn list_notes_restores_legacy_generated_untitled_titles() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, None).expect("create note");

        let mut tabs = load_tabs(&workspace).expect("load tabs");
        let tab = tabs
            .tabs
            .iter_mut()
            .find(|tab| tab.id == created.id)
            .expect("created tab");
        tab.title = "Untitled 20".to_owned();
        tab.system_title = true;
        save_tabs(&workspace, &tabs).expect("save tabs");

        let restored = list_notes(&workspace)
            .expect("list notes")
            .into_iter()
            .find(|tab| tab.id == created.id)
            .expect("restored tab");
        assert_eq!(restored.title, "Untitled");
    }

    #[test]
    fn archive_moves_note_out_of_active_list_but_keeps_it_searchable() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, Some("Archive me".to_owned())).expect("create");

        archive_note(&workspace, &created.id).expect("archive");

        assert!(list_notes(&workspace)
            .expect("active notes")
            .iter()
            .all(|tab| tab.id != created.id));
        assert!(list_archived_notes(&workspace)
            .expect("archived notes")
            .iter()
            .any(|tab| tab.id == created.id));
        assert!(list_searchable_notes(&workspace)
            .expect("searchable notes")
            .iter()
            .any(|tab| tab.id == created.id));
        assert!(archive_file_path(&workspace, &created.file_name)
            .expect("path")
            .is_file());
    }

    #[test]
    fn closing_note_keeps_file_and_tracks_recent_opening() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, Some("Close me".to_owned())).expect("create");

        close_note(&workspace, &created.id).expect("close");

        assert!(note_file_path(&workspace, &created.file_name)
            .expect("path")
            .is_file());
        assert!(list_open_notes(&workspace)
            .expect("open notes")
            .iter()
            .all(|tab| tab.id != created.id));
        assert!(list_recent_notes(&workspace, 20)
            .expect("recent notes")
            .iter()
            .any(|tab| tab.id == created.id));
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
