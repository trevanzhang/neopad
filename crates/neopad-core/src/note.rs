use crate::atomic_write::write_atomic;
use crate::path::{archive_file_path, note_file_path, trash_file_path};
use crate::{ensure_workspace_layout, NoteTab, TabsState, Workspace};
use anyhow::{bail, Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecoverableNoteWrite {
    pub recovery_file_name: String,
    pub target_file_name: String,
}

pub fn list_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    Ok(existing_tabs(workspace, load_tabs(workspace)?.tabs)?
        .into_iter()
        .filter(|tab| !tab.deleted && !tab.archived)
        .collect())
}

pub fn list_archived_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    Ok(existing_tabs(workspace, load_tabs(workspace)?.tabs)?
        .into_iter()
        .filter(|tab| !tab.deleted && tab.archived)
        .collect())
}

pub fn list_trashed_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    Ok(load_tabs(workspace)?
        .tabs
        .into_iter()
        .filter(|tab| tab.deleted)
        .collect())
}

pub fn list_recoverable_note_writes(workspace: &Workspace) -> Result<Vec<RecoverableNoteWrite>> {
    let mut recoveries = fs::read_dir(&workspace.notes_dir)
        .with_context(|| {
            format!(
                "failed to read notes directory {}",
                workspace.notes_dir.display()
            )
        })?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let recovery_file_name = path.file_name()?.to_str()?.to_owned();
            let target_file_name =
                crate::atomic_write::temporary_target_file_name(&recovery_file_name)?.to_owned();
            (path.is_file() && target_file_name.ends_with(".md")).then_some(RecoverableNoteWrite {
                recovery_file_name,
                target_file_name,
            })
        })
        .collect::<Vec<_>>();
    recoveries.sort_by(|left, right| left.recovery_file_name.cmp(&right.recovery_file_name));
    Ok(recoveries)
}

pub fn restore_recoverable_note_write(
    workspace: &Workspace,
    recovery_file_name: &str,
) -> Result<String> {
    let recovery = list_recoverable_note_writes(workspace)?
        .into_iter()
        .find(|entry| entry.recovery_file_name == recovery_file_name)
        .with_context(|| "recoverable note write not found")?;
    let recovery_path = workspace.notes_dir.join(&recovery.recovery_file_name);
    let target_path = note_file_path(workspace, &recovery.target_file_name)?;
    let contents = fs::read_to_string(&recovery_path)
        .with_context(|| format!("failed to read recovery file {}", recovery_path.display()))?;
    write_atomic(&target_path, &contents)?;
    let _ = fs::remove_file(&recovery_path);
    Ok(recovery.target_file_name)
}

pub fn clear_trash(workspace: &Workspace) -> Result<()> {
    clear_trash_with(workspace, move_to_system_trash)
}

fn clear_trash_with<F>(workspace: &Workspace, mut move_to_trash: F) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    ensure_workspace_layout(workspace)?;
    let mut trashed_files = fs::read_dir(&workspace.trash_dir)
        .with_context(|| {
            format!(
                "failed to read trash directory {}",
                workspace.trash_dir.display()
            )
        })?
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().is_file() => Some(Ok(entry.path())),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<std::io::Result<Vec<_>>>()?;
    trashed_files.sort();

    let mut tabs = load_tabs(workspace)?;
    if trashed_files.is_empty() {
        tabs.tabs.retain(|tab| !tab.deleted);
        return save_tabs(workspace, &tabs);
    }

    let mut failures = Vec::new();
    for source in trashed_files {
        if let Err(error) = move_to_trash(&source) {
            failures.push(format!("{}: {error:#}", source.display()));
        }
    }

    let mut remaining_deleted_ids = HashSet::new();
    for tab in tabs.tabs.iter().filter(|tab| tab.deleted) {
        if find_trashed_note_path(workspace, &tab.file_name)?.is_some() {
            remaining_deleted_ids.insert(tab.id.clone());
        }
    }
    tabs.tabs
        .retain(|tab| !tab.deleted || remaining_deleted_ids.contains(&tab.id));
    save_tabs(workspace, &tabs)?;

    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "failed to move {} trashed note(s) to the system Trash: {}",
            failures.len(),
            failures.join("; ")
        )
    }
}

fn move_to_system_trash(path: &Path) -> Result<()> {
    trash::delete(path)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("failed to move {} to the system Trash", path.display()))
}

pub fn list_open_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    Ok(existing_tabs(workspace, load_tabs(workspace)?.tabs)?
        .into_iter()
        .filter(|tab| !tab.deleted && tab.open)
        .collect())
}

pub fn list_searchable_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    Ok(existing_tabs(workspace, load_tabs(workspace)?.tabs)?
        .into_iter()
        .filter(|tab| !tab.deleted)
        .collect())
}

pub fn list_recent_notes(workspace: &Workspace, limit: usize) -> Result<Vec<NoteTab>> {
    let mut notes = existing_tabs(workspace, load_tabs(workspace)?.tabs)?
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
        content_revision: Some(content_revision(&content)),
    });
    tabs.active_tab_id = id.clone();
    if let Err(error) = save_tabs(workspace, &tabs) {
        let rollback = trash_file_path(workspace, &format!("orphaned-{}-{file_name}", now_ms()?))?;
        fs::rename(&path, &rollback).with_context(|| {
            format!("failed to roll back note creation after metadata error: {error:#}")
        })?;
        return Err(error);
    }

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
    let now = next_updated_at(tab.updated_at)?;
    let previous_content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read note file {}", path.display()))?;

    write_atomic(&path, content)?;
    tab.updated_at = now;
    tab.content_revision = Some(content_revision(content));

    let note = NoteContent {
        id: tab.id.clone(),
        title: tab.title.clone(),
        file_name: tab.file_name.clone(),
        content: content.to_owned(),
        updated_at: now,
    };
    if let Err(error) = save_tabs(workspace, &tabs) {
        write_atomic(&path, &previous_content).with_context(|| {
            format!("failed to roll back note content after metadata error: {error:#}")
        })?;
        return Err(error);
    }

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
    let tab = find_note_tab(workspace, note_id)?;
    let path = note_path(workspace, &tab)?;
    let disk_content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read note file {}", path.display()))?;
    if tab
        .content_revision
        .is_some_and(|revision| revision != content_revision(&disk_content))
    {
        bail!("note was modified outside NeoPad: {note_id}");
    }
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
    let now = next_updated_at(tab.updated_at)?;
    let previous_title = tab.title.clone();
    let next_title = normalized_title(Some(title));
    let mut previous_content = None;

    if tab.system_title {
        let path = note_path(workspace, tab)?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read note file {}", path.display()))?;
        if let Some(updated_content) =
            renamed_default_heading(&content, &previous_title, &next_title)
        {
            previous_content = Some((path.clone(), content));
            write_atomic(&path, &updated_content)?;
        }
    }

    tab.title = next_title;
    tab.system_title = false;
    tab.updated_at = now;
    let renamed = tab.clone();
    if let Err(error) = save_tabs(workspace, &tabs) {
        if let Some((path, content)) = previous_content {
            write_atomic(&path, &content).with_context(|| {
                format!("failed to roll back note rename after metadata error: {error:#}")
            })?;
        }
        return Err(error);
    }

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
    tab.updated_at = next_updated_at(tab.updated_at)?;
    let deleted = tab.clone();
    if let Err(error) = save_tabs(workspace, &tabs) {
        fs::rename(&target, &source).with_context(|| {
            format!("failed to restore trashed note after metadata error: {error:#}")
        })?;
        return Err(error);
    }

    Ok(deleted)
}

pub fn restore_note_from_trash(workspace: &Workspace, note_id: &str) -> Result<NoteTab> {
    let mut tabs = load_tabs(workspace)?;
    let tab = tabs
        .tabs
        .iter_mut()
        .find(|tab| tab.id == note_id && tab.deleted)
        .with_context(|| format!("trashed note not found: {note_id}"))?;
    let source = trashed_note_path(workspace, &tab.file_name)?;
    let target = if tab.archived {
        archive_file_path(workspace, &tab.file_name)?
    } else {
        note_file_path(workspace, &tab.file_name)?
    };
    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to restore trashed note {} to {}",
            source.display(),
            target.display()
        )
    })?;
    tab.deleted = false;
    tab.open = !tab.archived;
    tab.last_opened_at = if tab.archived { None } else { Some(now_ms()?) };
    tab.updated_at = next_updated_at(tab.updated_at)?;
    let restored = tab.clone();
    if let Err(error) = save_tabs(workspace, &tabs) {
        fs::rename(&target, &source).with_context(|| {
            format!("failed to return note to trash after metadata error: {error:#}")
        })?;
        return Err(error);
    }
    Ok(restored)
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
            tabs.tabs[tab_index].updated_at = next_updated_at(tabs.tabs[tab_index].updated_at)?;
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

pub fn note_file_path_for_id(workspace: &Workspace, note_id: &str) -> Result<std::path::PathBuf> {
    let tabs = load_tabs(workspace)?;
    let tab = tabs
        .tabs
        .into_iter()
        .find(|tab| tab.id == note_id)
        .with_context(|| format!("note not found: {note_id}"))?;
    if tab.deleted {
        trashed_note_path(workspace, &tab.file_name)
    } else {
        note_path(workspace, &tab)
    }
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
    tab.updated_at = next_updated_at(tab.updated_at)?;
    let archived = tab.clone();
    if let Err(error) = save_tabs(workspace, &tabs) {
        fs::rename(&target, &source).with_context(|| {
            format!("failed to restore archived note after metadata error: {error:#}")
        })?;
        return Err(error);
    }
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
    tab.updated_at = next_updated_at(tab.updated_at)?;
    let restored = tab.clone();
    if let Err(error) = save_tabs(workspace, &tabs) {
        fs::rename(&target, &source).with_context(|| {
            format!("failed to re-archive note after metadata error: {error:#}")
        })?;
        return Err(error);
    }
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
    tab.updated_at = next_updated_at(tab.updated_at)?;
    let updated = tab.clone();
    save_tabs(workspace, &tabs)?;
    Ok(updated)
}

fn load_tabs(workspace: &Workspace) -> Result<TabsState> {
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

/// Reconciles tab metadata with note files already present on disk.
///
/// Callers must hold `lock_workspace_for_write` because this operation may
/// rewrite the complete tabs metadata file.
pub fn reconcile_note_metadata(workspace: &Workspace) -> Result<bool> {
    let mut tabs = load_tabs(workspace)?;
    let changed = reconcile_trashed_tabs(workspace, &mut tabs)?;
    if changed {
        save_tabs(workspace, &tabs)?;
    }
    Ok(changed)
}

fn reconcile_trashed_tabs(workspace: &Workspace, tabs: &mut TabsState) -> Result<bool> {
    let mut changed = false;

    let mut stale_deleted_ids = HashSet::new();
    for tab in &tabs.tabs {
        if tab.deleted && find_trashed_note_path(workspace, &tab.file_name)?.is_none() {
            stale_deleted_ids.insert(tab.id.clone());
        }
    }
    if !stale_deleted_ids.is_empty() {
        tabs.tabs.retain(|tab| !stale_deleted_ids.contains(&tab.id));
        changed = true;
    }

    for tab in &mut tabs.tabs {
        if tab.deleted {
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

        let mut current_path = note_path(workspace, tab)?;
        if !current_path.is_file() && !tab.pinned {
            let alternate = if tab.archived {
                note_file_path(workspace, &tab.file_name)?
            } else {
                archive_file_path(workspace, &tab.file_name)?
            };
            if alternate.is_file() {
                tab.archived = !tab.archived;
                tab.open = !tab.archived;
                tab.updated_at = next_updated_at(tab.updated_at)?;
                current_path = alternate;
                changed = true;
            }
        }

        if !current_path.is_file() {
            if tab.pinned {
                continue;
            }
            tab.deleted = true;
            tab.open = false;
            tab.updated_at = next_updated_at(tab.updated_at)?;
            changed = true;
            continue;
        }

        let content = fs::read_to_string(&current_path)
            .with_context(|| format!("failed to read note file {}", current_path.display()))?;
        let revision = content_revision(&content);
        if tab.content_revision.as_deref() != Some(revision.as_str()) {
            tab.content_revision = Some(revision);
            tab.updated_at = next_updated_at(tab.updated_at)?;
            changed = true;
        }
    }

    changed |= reconcile_orphaned_notes(workspace, tabs, false)?;
    changed |= reconcile_orphaned_notes(workspace, tabs, true)?;
    changed |= reconcile_orphaned_trashed_notes(workspace, tabs)?;

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

fn reconcile_orphaned_trashed_notes(workspace: &Workspace, tabs: &mut TabsState) -> Result<bool> {
    let mut referenced = tabs
        .tabs
        .iter()
        .filter(|tab| tab.deleted)
        .map(|tab| tab.file_name.clone())
        .collect::<HashSet<_>>();
    let mut candidates = Vec::new();

    for entry in fs::read_dir(&workspace.trash_dir).with_context(|| {
        format!(
            "failed to read trash directory {}",
            workspace.trash_dir.display()
        )
    })? {
        let entry = entry?;
        let path = entry.path();
        let Some(trashed_file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(original_file_name) = original_file_name_from_trashed(trashed_file_name) else {
            continue;
        };
        if path.is_file()
            && !matches!(original_file_name, "inbox.md" | "clipboard.md")
            && note_file_path(workspace, original_file_name).is_ok()
        {
            candidates.push((original_file_name.to_owned(), path));
        }
    }

    candidates.sort_by(|left, right| right.1.cmp(&left.1));
    let mut changed = false;
    for (file_name, path) in candidates {
        if !referenced.insert(file_name.clone()) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read restored trashed note {}", path.display()))?;
        let base_id = Path::new(&file_name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("recovered")
            .to_owned();
        let mut id = base_id.clone();
        let mut suffix = 1;
        while tabs.tabs.iter().any(|tab| tab.id == id) {
            id = format!("{base_id}-recovered-{suffix}");
            suffix += 1;
        }
        let title = content
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("# "))
            .filter(|title| !title.trim().is_empty())
            .unwrap_or(&base_id)
            .trim()
            .to_owned();
        let now = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map_or(now_ms()?, |duration| duration.as_millis() as i64);
        tabs.tabs.push(NoteTab {
            id,
            title,
            file_name,
            created_at: now,
            updated_at: now,
            pinned: false,
            deleted: true,
            archived: false,
            open: false,
            last_opened_at: None,
            system_title: false,
            color: None,
            content_revision: Some(content_revision(&content)),
        });
        changed = true;
    }

    Ok(changed)
}

fn original_file_name_from_trashed(file_name: &str) -> Option<&str> {
    let remainder = file_name.strip_prefix("deleted-")?;
    let (deleted_at, original_file_name) = remainder.split_once('-')?;
    (!deleted_at.is_empty()
        && deleted_at
            .chars()
            .all(|character| character.is_ascii_digit()))
    .then_some(original_file_name)
}

fn reconcile_orphaned_notes(
    workspace: &Workspace,
    tabs: &mut TabsState,
    archived: bool,
) -> Result<bool> {
    let directory = if archived {
        &workspace.archive_dir
    } else {
        &workspace.notes_dir
    };
    let referenced = tabs
        .tabs
        .iter()
        .filter(|tab| !tab.deleted && tab.archived == archived)
        .map(|tab| tab.file_name.clone())
        .collect::<HashSet<_>>();
    let mut orphans = Vec::new();

    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to read notes directory {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if path.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("md")
            && !referenced.contains(file_name)
        {
            orphans.push((file_name.to_owned(), path));
        }
    }

    let changed = !orphans.is_empty();
    for (file_name, path) in orphans {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read orphaned note {}", path.display()))?;
        let base_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("recovered")
            .to_owned();
        let mut id = base_id.clone();
        let mut suffix = 1;
        while tabs.tabs.iter().any(|tab| tab.id == id) {
            id = format!("{base_id}-{suffix}");
            suffix += 1;
        }
        let title = content
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("# "))
            .filter(|title| !title.trim().is_empty())
            .unwrap_or(&base_id)
            .trim()
            .to_owned();
        let now = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map_or(now_ms()?, |duration| duration.as_millis() as i64);
        tabs.tabs.push(NoteTab {
            id,
            title,
            file_name,
            created_at: now,
            updated_at: now,
            pinned: false,
            deleted: false,
            archived,
            open: !archived,
            last_opened_at: None,
            system_title: false,
            color: None,
            content_revision: Some(content_revision(&content)),
        });
    }

    Ok(changed)
}

fn existing_tabs(workspace: &Workspace, tabs: Vec<NoteTab>) -> Result<Vec<NoteTab>> {
    tabs.into_iter()
        .filter_map(|tab| match note_path(workspace, &tab) {
            Ok(path) if path.is_file() => Some(Ok(tab)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn note_path(workspace: &Workspace, tab: &NoteTab) -> Result<std::path::PathBuf> {
    if tab.archived {
        archive_file_path(workspace, &tab.file_name)
    } else {
        note_file_path(workspace, &tab.file_name)
    }
}

fn trashed_note_path(workspace: &Workspace, file_name: &str) -> Result<std::path::PathBuf> {
    find_trashed_note_path(workspace, file_name)?
        .with_context(|| format!("trashed note file not found for {file_name}"))
}

fn find_trashed_note_path(
    workspace: &Workspace,
    file_name: &str,
) -> Result<Option<std::path::PathBuf>> {
    let suffix = format!("-{file_name}");
    let mut matches = fs::read_dir(&workspace.trash_dir)
        .with_context(|| {
            format!(
                "failed to read trash directory {}",
                workspace.trash_dir.display()
            )
        })?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            (path.is_file() && name.ends_with(&suffix)).then_some(path)
        })
        .collect::<Vec<_>>();
    matches.sort();
    Ok(matches.pop())
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

fn next_updated_at(previous: i64) -> Result<i64> {
    Ok(now_ms()?.max(previous.saturating_add(1)))
}

fn content_revision(content: &str) -> String {
    format!("{:x}", Sha256::digest(content.as_bytes()))
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
    fn preserved_note_write_can_be_listed_and_restored() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let recovery_file_name = ".clipboard.md.100.200.tmp";
        let recovery_path = workspace.notes_dir.join(recovery_file_name);
        fs::write(&recovery_path, "# Clipboard\n\nRecovered content").expect("recovery file");
        fs::write(workspace.notes_dir.join("not-a-note.tmp"), "ignore").expect("unrelated temp");

        assert_eq!(
            list_recoverable_note_writes(&workspace).expect("list recovery"),
            vec![RecoverableNoteWrite {
                recovery_file_name: recovery_file_name.to_owned(),
                target_file_name: "clipboard.md".to_owned(),
            }]
        );

        let restored = restore_recoverable_note_write(&workspace, recovery_file_name)
            .expect("restore recovery");
        assert_eq!(restored, "clipboard.md");
        assert_eq!(
            fs::read_to_string(workspace.notes_dir.join("clipboard.md")).expect("restored note"),
            "# Clipboard\n\nRecovered content"
        );
        assert!(!recovery_path.exists());
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
    fn trashed_notes_can_be_listed_and_restored() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, Some("Restore me".to_owned())).expect("create");

        let deleted = delete_note_to_trash(&workspace, &created.id).expect("delete");
        assert!(deleted.deleted);
        assert_eq!(list_trashed_notes(&workspace).expect("list trash").len(), 1);

        let restored = restore_note_from_trash(&workspace, &created.id).expect("restore");
        assert!(!restored.deleted);
        assert!(note_file_path(&workspace, &restored.file_name)
            .expect("note path")
            .is_file());
        assert!(list_trashed_notes(&workspace)
            .expect("list trash")
            .is_empty());
    }

    #[test]
    fn reconciliation_removes_metadata_for_manually_cleared_trash() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, Some("Manual cleanup".to_owned())).expect("create");
        delete_note_to_trash(&workspace, &created.id).expect("delete");
        let trashed_path = trashed_note_path(&workspace, &created.file_name).expect("trashed path");
        fs::remove_file(trashed_path).expect("manual cleanup");

        assert!(reconcile_note_metadata(&workspace).expect("reconcile"));
        assert!(list_trashed_notes(&workspace)
            .expect("list trash")
            .is_empty());
        assert!(load_tabs(&workspace)
            .expect("tabs")
            .tabs
            .iter()
            .all(|tab| tab.id != created.id));
    }

    #[test]
    fn clear_trash_moves_files_out_of_workspace_and_removes_deleted_metadata() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, Some("Clear me".to_owned())).expect("create");
        delete_note_to_trash(&workspace, &created.id).expect("delete");
        let system_trash = temp_dir.path().join("system-trash");
        fs::create_dir(&system_trash).expect("system trash");

        clear_trash_with(&workspace, |source| {
            let target = system_trash.join(source.file_name().expect("trashed file name"));
            fs::rename(source, target).context("fake system trash move")
        })
        .expect("clear trash");

        assert!(fs::read_dir(&workspace.trash_dir)
            .expect("trash dir")
            .next()
            .is_none());
        assert!(list_trashed_notes(&workspace)
            .expect("list trash")
            .is_empty());
        assert!(load_tabs(&workspace)
            .expect("tabs")
            .tabs
            .iter()
            .all(|tab| tab.id != created.id));
        let moved_files = fs::read_dir(&system_trash)
            .expect("system trash")
            .map(|entry| entry.expect("moved entry").path())
            .collect::<Vec<_>>();
        assert_eq!(moved_files.len(), 1);
        assert!(fs::read_to_string(&moved_files[0])
            .expect("preserved note")
            .contains("Clear me"));
    }

    #[test]
    fn reconciliation_rediscovers_notes_restored_from_the_system_trash() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, Some("Restore me".to_owned())).expect("create");
        delete_note_to_trash(&workspace, &created.id).expect("delete");
        let system_trash = temp_dir.path().join("system-trash");
        fs::create_dir(&system_trash).expect("system trash");

        clear_trash_with(&workspace, |source| {
            let target = system_trash.join(source.file_name().expect("trashed file name"));
            fs::rename(source, target).context("fake system trash move")
        })
        .expect("clear trash");
        assert!(list_trashed_notes(&workspace)
            .expect("empty NeoPad trash")
            .is_empty());

        let system_trashed_path = fs::read_dir(&system_trash)
            .expect("system trash")
            .next()
            .expect("system trash entry")
            .expect("system trash file")
            .path();
        let restored_path = workspace
            .trash_dir
            .join(system_trashed_path.file_name().expect("restored file name"));
        fs::rename(system_trashed_path, &restored_path).expect("restore from system trash");

        assert!(reconcile_note_metadata(&workspace).expect("reconcile restored trash"));
        let trashed = list_trashed_notes(&workspace).expect("list restored trash");
        assert_eq!(trashed.len(), 1);
        assert_eq!(trashed[0].title, "Restore me");
        assert_eq!(trashed[0].file_name, created.file_name);
        assert!(trashed[0].deleted);

        let restored = restore_note_from_trash(&workspace, &trashed[0].id)
            .expect("restore note through NeoPad");
        assert_eq!(restored.file_name, created.file_name);
        assert!(note_file_path(&workspace, &created.file_name)
            .expect("restored note path")
            .is_file());
    }

    #[test]
    fn clear_trash_keeps_failed_files_and_metadata_visible() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let moved = create_note(&workspace, Some("Move me".to_owned())).expect("first note");
        let failed = create_note(&workspace, Some("Keep me".to_owned())).expect("second note");
        delete_note_to_trash(&workspace, &moved.id).expect("delete first");
        delete_note_to_trash(&workspace, &failed.id).expect("delete second");
        let failed_path = trashed_note_path(&workspace, &failed.file_name).expect("failed path");
        let system_trash = temp_dir.path().join("system-trash");
        fs::create_dir(&system_trash).expect("system trash");

        let error = clear_trash_with(&workspace, |source| {
            if source == failed_path {
                bail!("simulated system Trash failure");
            }
            let target = system_trash.join(source.file_name().expect("trashed file name"));
            fs::rename(source, target).context("fake system trash move")
        })
        .expect_err("partial failure");

        assert!(error.to_string().contains("failed to move 1 trashed note"));
        assert!(trashed_note_path(&workspace, &failed.file_name)
            .expect("failed note remains")
            .is_file());
        let trashed = list_trashed_notes(&workspace).expect("list trash");
        assert_eq!(trashed.len(), 1);
        assert_eq!(trashed[0].id, failed.id);
        assert!(trashed.iter().all(|tab| tab.id != moved.id));
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

        reconcile_note_metadata(&workspace).expect("reconcile");
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

        reconcile_note_metadata(&workspace).expect("reconcile");
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

    #[test]
    fn consecutive_writes_always_advance_the_conflict_version() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, Some("Versioned".to_owned())).expect("create");

        let first = write_note_atomic_checked(&workspace, &created.id, "first", created.updated_at)
            .expect("first write");
        let second = write_note_atomic_checked(&workspace, &created.id, "second", first.updated_at)
            .expect("second write");

        assert!(first.updated_at > created.updated_at);
        assert!(second.updated_at > first.updated_at);
        assert!(
            write_note_atomic_checked(&workspace, &created.id, "stale", first.updated_at).is_err()
        );
    }

    #[test]
    fn checked_write_rejects_content_changed_outside_neopad() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let created = create_note(&workspace, Some("External edit".to_owned())).expect("create");
        let path = note_file_path(&workspace, &created.file_name).expect("path");
        fs::write(path, "changed outside NeoPad").expect("external write");

        let error = write_note_atomic_checked(
            &workspace,
            &created.id,
            "stale editor content",
            created.updated_at,
        )
        .expect_err("external edit must be detected");

        assert!(error.to_string().contains("outside NeoPad"));
    }

    #[test]
    fn reconciliation_recovers_orphaned_and_interrupted_archive_notes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        fs::write(
            workspace.notes_dir.join("recovered.md"),
            "# Recovered\n\nBody",
        )
        .expect("orphan note");
        let created = create_note(&workspace, Some("Interrupted".to_owned())).expect("create");
        fs::rename(
            note_file_path(&workspace, &created.file_name).expect("active path"),
            archive_file_path(&workspace, &created.file_name).expect("archive path"),
        )
        .expect("interrupted archive move");

        reconcile_note_metadata(&workspace).expect("reconcile");

        let recovered = list_notes(&workspace).expect("active notes");
        assert!(recovered.iter().any(|tab| tab.file_name == "recovered.md"));
        let archived = list_archived_notes(&workspace).expect("archived notes");
        assert!(archived.iter().any(|tab| tab.id == created.id));
    }
}
