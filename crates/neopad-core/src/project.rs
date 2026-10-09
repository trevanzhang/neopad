use crate::{
    note::{
        content_revision, ensure_regular_directory, load_tabs, markdown_files_recursive,
        next_updated_at, normalized_directory, normalized_directory_name, note_path, now_ms,
        path_is_inside_directory, regular_files_recursive_strict, relative_path_string,
        replaced_directory_prefix, save_tabs,
    },
    path::{project_directory_path, project_file_path},
    NoteContent, NoteTab, TabsState, Workspace,
};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};

pub fn list_project_notes(workspace: &Workspace) -> Result<Vec<NoteTab>> {
    Ok(crate::list_notes(workspace)?
        .into_iter()
        .filter(|tab| tab.project_relative_path.is_some())
        .collect())
}

pub fn list_project_directories(workspace: &Workspace) -> Result<Vec<String>> {
    crate::note::list_relative_directories(&workspace.projects_dir)
}

pub fn create_project_directory(workspace: &Workspace, directory: &str) -> Result<String> {
    let directory = normalized_directory(directory)?;
    fs::create_dir_all(project_directory_path(workspace, &directory)?)?;
    Ok(directory)
}

/// Callers hold the workspace lock, just as for ordinary note operations.
pub fn create_project_note(
    workspace: &Workspace,
    directory: &str,
    title: Option<String>,
    body: &str,
) -> Result<NoteContent> {
    let directory = normalized_directory(directory)?;
    create_project_directory(workspace, &directory)?;
    let note = crate::create_note_with_body(workspace, title, body)?;
    move_note_to_project(workspace, &note.id, &directory)?;
    crate::read_note(workspace, &note.id)
}

pub fn move_note_to_project(workspace: &Workspace, id: &str, directory: &str) -> Result<NoteTab> {
    let directory = normalized_directory(directory)?;
    let mut tabs = load_tabs(workspace)?;
    let tab = crate::note::find_note_tab_mut(&mut tabs, id)?;
    if tab.pinned {
        bail!("pinned notes cannot be moved to a project");
    }
    let source = note_path(workspace, tab)?;
    let relative = format!("{directory}/{}", tab.file_name);
    let target = project_file_path(workspace, &relative)?;
    if source == target {
        return Ok(tab.clone());
    }
    if target.exists() {
        bail!("a project note already exists at {relative}");
    }
    fs::create_dir_all(target.parent().context("project note has no parent")?)?;
    let updated_at = next_updated_at(tab.updated_at)?;
    fs::rename(&source, &target)?;
    tab.project_relative_path = Some(relative);
    tab.archived = false;
    tab.archive_relative_path = None;
    tab.updated_at = updated_at;
    let moved = tab.clone();
    if let Err(error) = save_tabs(workspace, &tabs) {
        fs::rename(&target, &source).context("could not roll back project move")?;
        return Err(error);
    }
    Ok(moved)
}

pub fn move_note_to_capture(workspace: &Workspace, id: &str) -> Result<NoteTab> {
    let mut tabs = load_tabs(workspace)?;
    let tab = crate::note::find_note_tab_mut(&mut tabs, id)?;
    let source = note_path(workspace, tab)?;
    let target = crate::path::note_file_path(workspace, &tab.file_name)?;
    if source == target {
        return Ok(tab.clone());
    }
    if target.exists() {
        bail!("a capture note already exists with this file name");
    }
    let updated_at = next_updated_at(tab.updated_at)?;
    fs::rename(&source, &target)?;
    tab.project_relative_path = None;
    tab.archive_relative_path = None;
    tab.archived = false;
    tab.open = true;
    tab.updated_at = updated_at;
    let moved = tab.clone();
    if let Err(error) = save_tabs(workspace, &tabs) {
        fs::rename(&target, &source).context("could not roll back capture move")?;
        return Err(error);
    }
    Ok(moved)
}

pub fn rename_project_directory(
    workspace: &Workspace,
    directory: &str,
    name: &str,
) -> Result<String> {
    let directory = normalized_directory(directory)?;
    let name = normalized_directory_name(name)?;
    let target = match Path::new(&directory).parent().and_then(Path::to_str) {
        Some(parent) if !parent.is_empty() => format!("{parent}/{name}"),
        _ => name,
    };
    relocate_directory(workspace, &directory, &target, false)
}

pub fn move_project_directory(
    workspace: &Workspace,
    directory: &str,
    parent: &str,
) -> Result<String> {
    let directory = normalized_directory(directory)?;
    let name = Path::new(&directory)
        .file_name()
        .and_then(|value| value.to_str())
        .context("invalid project name")?;
    let target = if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{}/{name}", normalized_directory(parent)?)
    };
    relocate_directory(workspace, &directory, &target, false)
}

pub fn archive_project_directory(workspace: &Workspace, directory: &str) -> Result<String> {
    let directory = normalized_directory(directory)?;
    relocate_directory(
        workspace,
        &directory,
        &format!("projects/{directory}"),
        true,
    )
}

fn relocate_directory(
    workspace: &Workspace,
    source: &str,
    target: &str,
    archive: bool,
) -> Result<String> {
    let source = normalized_directory(source)?;
    let target = normalized_directory(target)?;
    if !archive && source == target {
        return Ok(source);
    }
    if !archive && path_is_inside_directory(&target, &source) {
        bail!("cannot move a project into itself");
    }
    let source_path = project_directory_path(workspace, &source)?;
    let target_path = if archive {
        crate::path::archive_directory_path(workspace, &target)?
    } else {
        project_directory_path(workspace, &target)?
    };
    ensure_regular_directory(&source_path, "project")?;
    // Never move unmanaged attachments or links as a side effect of a note action.
    let files = regular_files_recursive_strict(&source_path)?;
    if files
        .iter()
        .any(|file| file.extension().and_then(|ext| ext.to_str()) != Some("md"))
    {
        bail!("project contains an unmanaged file");
    }
    crate::reconcile_note_metadata(workspace)?;
    let mut tabs = load_tabs(workspace)?;
    if target_path.exists() {
        bail!("target directory already exists: {target}");
    }
    fs::create_dir_all(target_path.parent().context("project has no parent")?)?;
    for tab in tabs
        .tabs
        .iter_mut()
        .filter(|tab| !tab.deleted && !tab.archived)
    {
        if let Some(relative) = &tab.project_relative_path {
            if let Some(next) = replaced_directory_prefix(relative, &source, &target) {
                if archive {
                    tab.archived = true;
                    tab.archive_relative_path = Some(next);
                    tab.open = false;
                } else {
                    tab.project_relative_path = Some(next);
                }
                tab.updated_at = next_updated_at(tab.updated_at)?;
            }
        }
    }
    fs::rename(&source_path, &target_path)?;
    if let Err(error) = save_tabs(workspace, &tabs) {
        fs::rename(&target_path, &source_path)
            .context("could not roll back project directory move")?;
        return Err(error);
    }
    Ok(target)
}

pub fn delete_project_directory_to_trash(workspace: &Workspace, directory: &str) -> Result<usize> {
    let directory = normalized_directory(directory)?;
    let path = project_directory_path(workspace, &directory)?;
    ensure_regular_directory(&path, "project")?;
    if regular_files_recursive_strict(&path)?
        .iter()
        .any(|file| file.extension().and_then(|ext| ext.to_str()) != Some("md"))
    {
        bail!("project contains an unmanaged file");
    }
    crate::reconcile_note_metadata(workspace)?;
    let notes = list_project_notes(workspace)?;
    let ids = notes
        .iter()
        .filter(|tab| {
            tab.project_relative_path
                .as_deref()
                .is_some_and(|p| path_is_inside_directory(p, &directory))
        })
        .map(|tab| tab.id.clone())
        .collect::<Vec<_>>();
    for id in &ids {
        crate::delete_note_to_trash(workspace, id)?;
    }
    // Verify the resolved target stays under the projects root before deleting empty directories.
    crate::path::ensure_inside_workspace(&workspace.projects_dir, &path)?;
    if !regular_files_recursive_strict(&path)?.is_empty() {
        bail!("project still contains files");
    }
    fs::remove_dir_all(path)?;
    Ok(ids.len())
}

pub(crate) fn reconcile_project_locations(
    workspace: &Workspace,
    tabs: &mut TabsState,
) -> Result<bool> {
    let files = markdown_files_recursive(&workspace.projects_dir)?;
    let mut owned = tabs
        .tabs
        .iter()
        .filter(|tab| !tab.deleted && !tab.archived)
        .filter_map(|tab| tab.project_relative_path.clone())
        .collect::<HashSet<_>>();
    let mut changed = false;
    let archived_paths = tabs
        .tabs
        .iter()
        .filter(|tab| !tab.deleted && tab.archived)
        .filter_map(|tab| tab.archive_relative_path.clone())
        .collect::<HashSet<_>>();
    for tab in tabs
        .tabs
        .iter_mut()
        .filter(|tab| !tab.deleted && !tab.archived && tab.project_relative_path.is_some())
    {
        if note_path(workspace, tab)?.is_file() {
            continue;
        }
        // Recover the file move if a project archive was interrupted before
        // its metadata commit. Never adopt a file owned by another note.
        let archived = format!("projects/{}", tab.project_relative_path.as_deref().unwrap());
        let archived_path = crate::path::archive_file_path(workspace, &archived)?;
        if !archived_paths.contains(&archived)
            && fs::read_to_string(&archived_path).is_ok_and(|content| {
                tab.content_revision.as_deref() == Some(content_revision(&content).as_str())
            })
        {
            tab.archived = true;
            tab.archive_relative_path = Some(archived);
            tab.open = false;
            tab.updated_at = next_updated_at(tab.updated_at)?;
            changed = true;
            continue;
        }
        let candidates = files
            .iter()
            .filter(|file| {
                file.file_name().and_then(|name| name.to_str()) == Some(&tab.file_name)
                    && relative_path_string(&workspace.projects_dir, file)
                        .is_ok_and(|path| !owned.contains(&path))
                    && fs::read_to_string(file).is_ok_and(|content| {
                        tab.content_revision.as_deref() == Some(content_revision(&content).as_str())
                    })
            })
            .collect::<Vec<_>>();
        if candidates.len() == 1 {
            tab.project_relative_path = Some(relative_path_string(
                &workspace.projects_dir,
                candidates[0],
            )?);
            owned.insert(tab.project_relative_path.clone().unwrap());
            tab.updated_at = next_updated_at(tab.updated_at)?;
            changed = true;
        }
    }
    Ok(changed)
}

pub(crate) fn reconcile_project_notes(workspace: &Workspace, tabs: &mut TabsState) -> Result<bool> {
    let known = tabs
        .tabs
        .iter()
        .filter(|tab| !tab.deleted && !tab.archived)
        .filter_map(|tab| tab.project_relative_path.clone())
        .collect::<HashSet<_>>();
    let mut changed = false;
    for path in markdown_files_recursive(&workspace.projects_dir)?
        .into_iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
    {
        let relative = relative_path_string(&workspace.projects_dir, &path)?;
        if known.contains(&relative) {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("invalid project file name")?
            .to_owned();
        project_file_path(workspace, &relative)?;
        let mut id = format!(
            "project-{}",
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("note")
        );
        while tabs.tabs.iter().any(|tab| tab.id == id) {
            id.push('-');
        }
        tabs.tabs
            .push(project_tab(id, file_name, relative, &content, None)?);
        changed = true;
    }
    Ok(changed)
}

fn project_tab(
    id: String,
    file_name: String,
    relative: String,
    content: &str,
    trash: Option<String>,
) -> Result<NoteTab> {
    let now = now_ms()?;
    let title = content
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .filter(|title| !title.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| file_name.trim_end_matches(".md").to_owned());
    Ok(NoteTab {
        id,
        title,
        file_name,
        created_at: now,
        updated_at: now,
        pinned: false,
        deleted: trash.is_some(),
        archived: false,
        archive_relative_path: None,
        project_relative_path: Some(relative),
        trash_file_name: trash,
        open: false,
        last_opened_at: None,
        system_title: false,
        color: None,
        content_revision: Some(content_revision(content)),
    })
}

#[derive(Deserialize, Default)]
struct LegacyState {
    #[serde(default)]
    prompts: Vec<LegacyPrompt>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyPrompt {
    id: String,
    relative_path: String,
    deleted_at: Option<i64>,
    trash_file_name: Option<String>,
}

/// Import old prompts as ordinary notes. Journal note metadata before moving each
/// file so an interrupted migration can resume without losing or duplicating it.
pub(crate) fn migrate_legacy_prompts(workspace: &Workspace) -> Result<()> {
    let legacy = workspace.root.join("prompts");
    let old_meta = workspace.meta_dir.join("prompts.json");
    let old_trash = workspace.trash_dir.join("prompts");
    if !legacy.exists() && !old_trash.exists() {
        return Ok(());
    }
    let records: LegacyState = if old_meta.is_file() {
        serde_json::from_slice(&fs::read(&old_meta)?)?
    } else {
        LegacyState::default()
    };
    let mut tabs = load_tabs(workspace)?;
    let files = if legacy.is_dir() {
        regular_files_recursive_strict(&legacy)?
    } else {
        Vec::new()
    };
    for source in files
        .into_iter()
        .filter(|file| file.extension().and_then(|ext| ext.to_str()) == Some("md"))
    {
        let relative = relative_path_string(&legacy, &source)?;
        let record = records
            .prompts
            .iter()
            .find(|record| record.deleted_at.is_none() && record.relative_path == relative);
        let id = record
            .map(|record| format!("legacy-prompt-{}", record.id))
            .unwrap_or_else(|| format!("legacy-prompt-{}", hex_revision(&relative)));
        import_legacy_file(workspace, &mut tabs, &source, &id, &relative, false)?;
    }
    if old_trash.is_dir() {
        for source in regular_files_recursive_strict(&old_trash)?
            .into_iter()
            .filter(|file| file.extension().and_then(|ext| ext.to_str()) == Some("md"))
        {
            let name = source
                .file_name()
                .and_then(|name| name.to_str())
                .context("invalid old trash name")?;
            let record = records.prompts.iter().find(|record| {
                record.deleted_at.is_some() && record.trash_file_name.as_deref() == Some(name)
            });
            let relative = record
                .map(|record| record.relative_path.clone())
                .unwrap_or_else(|| format!("Recovered/{name}"));
            let id = record
                .map(|record| format!("legacy-prompt-{}", record.id))
                .unwrap_or_else(|| format!("legacy-prompt-{}", hex_revision(name)));
            import_legacy_file(workspace, &mut tabs, &source, &id, &relative, true)?;
        }
    }
    Ok(())
}

fn hex_revision(value: &str) -> String {
    content_revision(value)
}

fn import_legacy_file(
    workspace: &Workspace,
    tabs: &mut TabsState,
    source: &Path,
    id: &str,
    relative: &str,
    deleted: bool,
) -> Result<()> {
    let content = fs::read_to_string(source)?;
    let file_name = Path::new(relative)
        .file_name()
        .and_then(|name| name.to_str())
        .context("invalid legacy file name")?
        .to_owned();
    let destination = if let Some(tab) = tabs.tabs.iter().find(|tab| tab.id == id) {
        if let Some(name) = &tab.trash_file_name {
            crate::path::trash_file_path(workspace, name)?
        } else {
            project_file_path(
                workspace,
                tab.project_relative_path
                    .as_deref()
                    .context("migration journal missing project path")?,
            )?
        }
    } else {
        let mut migrated = format!("原提示词/{relative}");
        let mut suffix = 1;
        while project_file_path(workspace, &migrated)?.exists()
            || tabs
                .tabs
                .iter()
                .any(|tab| tab.project_relative_path.as_deref() == Some(&migrated))
        {
            migrated = format!("原提示词-{suffix}/{relative}");
            suffix += 1;
        }
        let trash = if deleted {
            Some(crate::note::unique_trash_file_name(workspace, &file_name)?)
        } else {
            None
        };
        let tab = project_tab(id.to_owned(), file_name, migrated, &content, trash)?;
        let target = if let Some(name) = &tab.trash_file_name {
            crate::path::trash_file_path(workspace, name)?
        } else {
            project_file_path(workspace, tab.project_relative_path.as_deref().unwrap())?
        };
        tabs.tabs.push(tab);
        save_tabs(workspace, tabs)?;
        target
    };
    if destination.exists() {
        bail!("legacy migration target already exists; original file preserved");
    }
    fs::create_dir_all(
        destination
            .parent()
            .context("migration target has no parent")?,
    )?;
    fs::rename(source, destination)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn interrupted_project_archive_keeps_note_identity() {
        let temp = tempfile::tempdir().unwrap();
        let ws = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        let note = create_project_note(&ws, "Demo/Sub", None, "preserved").unwrap();
        fs::create_dir_all(ws.archive_dir.join("projects")).unwrap();
        fs::rename(
            ws.projects_dir.join("Demo"),
            ws.archive_dir.join("projects/Demo"),
        )
        .unwrap();
        reconcile_note_metadata(&ws).unwrap();
        assert_eq!(list_archived_notes(&ws).unwrap()[0].id, note.id);
        assert_eq!(read_note(&ws, &note.id).unwrap().content, note.content);
        unarchive_note(&ws, &note.id).unwrap();
        assert_eq!(list_project_notes(&ws).unwrap()[0].id, note.id);
    }

    #[test]
    fn project_reminders_keep_delivery_state_across_folder_moves() {
        let temp = tempfile::tempdir().unwrap();
        let ws = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        let note =
            create_project_note(&ws, "Demo", None, "- [ ] @remind 2000-01-01 09:00 call").unwrap();
        let reminder = claim_due_reminders(&ws).unwrap().remove(0);
        rename_project_directory(&ws, "Demo", "Renamed").unwrap();
        assert!(claim_due_reminders(&ws).unwrap().is_empty());
        assert_eq!(list_reminders(&ws).unwrap()[0].id, reminder.id);
        complete_reminder(&ws, &note.id, reminder.line_number, &reminder.id).unwrap();
        assert!(read_note(&ws, &note.id).unwrap().content.contains("- [x]"));
    }

    #[test]
    fn interrupted_project_writes_are_recoverable_in_their_own_directory() {
        let temp = tempfile::tempdir().unwrap();
        let ws = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        create_project_directory(&ws, "Demo/Sub").unwrap();
        let file = ws.projects_dir.join("Demo/Sub/.draft.md.100.200.tmp");
        fs::write(&file, "recovered").unwrap();
        reconcile_note_metadata(&ws).unwrap();
        assert!(list_project_notes(&ws).unwrap().is_empty());
        let entries = list_recoverable_note_writes(&ws).unwrap();
        assert_eq!(entries[0].target_file_name, "projects/Demo/Sub/draft.md");
        restore_recoverable_note_write(&ws, &entries[0].recovery_file_name).unwrap();
        reconcile_note_metadata(&ws).unwrap();
        let note = list_project_notes(&ws).unwrap().remove(0);
        assert_eq!(read_note(&ws, &note.id).unwrap().content, "recovered");
        assert!(!file.exists());
    }

    #[test]
    fn project_lifecycle_preserves_identity_content_and_search() {
        let temp = tempfile::tempdir().unwrap();
        let ws = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        let note = create_project_note(&ws, "Demo/Research", Some("Plan".into()), "project needle")
            .unwrap();
        assert!(!ws.notes_dir.join(&note.file_name).exists());
        assert_eq!(search_notes(&ws, "needle", 10).unwrap()[0].note_id, note.id);
        let revised =
            write_note_atomic_checked(&ws, &note.id, "edited needle", note.updated_at).unwrap();
        assert!(write_note_atomic_checked(&ws, &note.id, "stale", note.updated_at).is_err());
        rename_project_directory(&ws, "Demo", "Renamed").unwrap();
        assert_eq!(read_note(&ws, &note.id).unwrap().content, revised.content);
        archive_project_directory(&ws, "Renamed").unwrap();
        assert_eq!(list_archived_notes(&ws).unwrap()[0].id, note.id);
        assert!(ws
            .archive_dir
            .join(format!("projects/Renamed/Research/{}", note.file_name))
            .is_file());
        unarchive_note(&ws, &note.id).unwrap();
        delete_note_to_trash(&ws, &note.id).unwrap();
        restore_note_from_trash(&ws, &note.id).unwrap();
        move_note_to_capture(&ws, &note.id).unwrap();
        assert_eq!(read_note(&ws, &note.id).unwrap().content, revised.content);
        assert!(ws.notes_dir.join(&note.file_name).is_file());
    }

    #[test]
    fn duplicate_names_have_independent_trash_and_restore_locations() {
        let temp = tempfile::tempdir().unwrap();
        let ws = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        for (folder, body) in [("One", "first"), ("Two", "second")] {
            create_project_directory(&ws, folder).unwrap();
            fs::write(ws.projects_dir.join(folder).join("same.md"), body).unwrap();
        }
        reconcile_note_metadata(&ws).unwrap();
        let notes = list_project_notes(&ws).unwrap();
        assert_eq!(notes.len(), 2);
        for note in &notes {
            delete_note_to_trash(&ws, &note.id).unwrap();
        }
        reconcile_note_metadata(&ws).unwrap();
        let trashed = list_trashed_notes(&ws).unwrap();
        assert_eq!(trashed.len(), 2);
        assert_ne!(trashed[0].trash_file_name, trashed[1].trash_file_name);
        for note in &notes {
            restore_note_from_trash(&ws, &note.id).unwrap();
        }
        assert_eq!(
            fs::read_to_string(ws.projects_dir.join("One/same.md")).unwrap(),
            "first"
        );
        assert_eq!(
            fs::read_to_string(ws.projects_dir.join("Two/same.md")).unwrap(),
            "second"
        );
    }

    #[test]
    fn migration_preserves_nested_empty_and_trashed_prompts_and_runs_once() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("prompts/Writing")).unwrap();
        fs::create_dir_all(root.join("trash/prompts")).unwrap();
        fs::create_dir_all(root.join("meta")).unwrap();
        fs::create_dir_all(root.join("projects/原提示词/Writing")).unwrap();
        fs::write(root.join("projects/原提示词/Writing/Review.md"), "existing").unwrap();
        fs::write(root.join("prompts/Writing/Review.md"), "正文 unchanged").unwrap();
        fs::write(root.join("prompts/Empty.md"), "").unwrap();
        fs::write(root.join("trash/prompts/deleted-1-old.md"), "recover me").unwrap();
        fs::write(root.join("meta/prompts.json"), r#"{"version":1,"prompts":[{"id":"old","relativePath":"Writing/Review.md"},{"id":"trashed","relativePath":"Writing/Old.md","deletedAt":1,"trashFileName":"deleted-1-old.md"}]}"#).unwrap();
        let ws = init_workspace(Some(root.clone())).unwrap();
        reconcile_note_metadata(&ws).unwrap();
        assert_eq!(
            read_note(&ws, "legacy-prompt-old").unwrap().content,
            "正文 unchanged"
        );
        assert_eq!(
            fs::read_to_string(root.join("projects/原提示词/Writing/Review.md")).unwrap(),
            "existing"
        );
        assert_eq!(list_project_notes(&ws).unwrap().len(), 3);
        assert_eq!(list_trashed_notes(&ws).unwrap().len(), 1);
        init_workspace(Some(root)).unwrap();
        assert_eq!(list_project_notes(&ws).unwrap().len(), 3);
        restore_note_from_trash(&ws, "legacy-prompt-trashed").unwrap();
        assert_eq!(
            read_note(&ws, "legacy-prompt-trashed").unwrap().content,
            "recover me"
        );
        assert!(ws.projects_dir.join("原提示词/Writing/Old.md").is_file());
    }

    #[test]
    fn migration_resumes_a_journal_written_before_the_file_move() {
        let temp = tempfile::tempdir().unwrap();
        let ws = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        fs::create_dir_all(ws.root.join("prompts")).unwrap();
        fs::write(ws.root.join("prompts/Resume.md"), "body").unwrap();
        let relative = "原提示词/Resume.md";
        let id = format!("legacy-prompt-{}", hex_revision("Resume.md"));
        let mut tabs = load_tabs(&ws).unwrap();
        tabs.tabs.push(
            project_tab(
                id.clone(),
                "Resume.md".into(),
                relative.into(),
                "body",
                None,
            )
            .unwrap(),
        );
        save_tabs(&ws, &tabs).unwrap();
        init_workspace(Some(ws.root.clone())).unwrap();
        assert_eq!(read_note(&ws, &id).unwrap().content, "body");
        assert_eq!(list_project_notes(&ws).unwrap().len(), 1);
    }

    #[test]
    fn projects_reject_unsafe_moves_and_preserve_unmanaged_files() {
        let temp = tempfile::tempdir().unwrap();
        let ws = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        for path in ["../escape", "/outside", "C:/outside", "Demo/../escape"] {
            assert!(create_project_directory(&ws, path).is_err());
        }
        let note = create_project_note(&ws, "Demo", None, "kept").unwrap();
        assert!(move_project_directory(&ws, "Demo", "Demo/Sub").is_err());
        assert!(move_note_to_project(&ws, "inbox", "Demo").is_err());
        fs::write(ws.projects_dir.join("Demo/attachment.bin"), b"kept").unwrap();
        assert!(delete_project_directory_to_trash(&ws, "Demo").is_err());
        assert!(archive_project_directory(&ws, "Demo").is_err());
        assert!(read_note(&ws, &note.id).unwrap().content.contains("kept"));
    }

    #[test]
    fn deleting_a_project_and_restoring_recreates_nested_directories() {
        let temp = tempfile::tempdir().unwrap();
        let ws = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        let note = create_project_note(&ws, "Demo/Sub", None, "body").unwrap();
        assert_eq!(delete_project_directory_to_trash(&ws, "Demo").unwrap(), 1);
        assert!(!ws.projects_dir.join("Demo").exists());
        restore_note_from_trash(&ws, &note.id).unwrap();
        assert!(ws.projects_dir.join("Demo/Sub").is_dir());
    }

    #[test]
    fn external_moves_do_not_adopt_other_notes_with_the_same_name() {
        let temp = tempfile::tempdir().unwrap();
        let ws = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        for folder in ["One", "Two"] {
            create_project_directory(&ws, folder).unwrap();
            fs::write(ws.projects_dir.join(folder).join("same.md"), folder).unwrap();
        }
        reconcile_note_metadata(&ws).unwrap();
        let notes = list_project_notes(&ws).unwrap();
        let first = notes
            .iter()
            .find(|note| note.project_relative_path.as_deref() == Some("One/same.md"))
            .unwrap();
        fs::rename(ws.projects_dir.join("One"), ws.projects_dir.join("Moved")).unwrap();
        reconcile_note_metadata(&ws).unwrap();
        assert_eq!(read_note(&ws, &first.id).unwrap().content, "One");
        let moved = list_project_notes(&ws)
            .unwrap()
            .into_iter()
            .find(|note| note.id == first.id)
            .unwrap();
        assert_eq!(
            moved.project_relative_path.as_deref(),
            Some("Moved/same.md")
        );
    }
}
