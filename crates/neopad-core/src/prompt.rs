use crate::{
    atomic_write::write_atomic,
    path::{ensure_inside_workspace, prompt_directory_path, prompt_relative_path},
    Workspace,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_PROMPT_BYTES: u64 = 128 * 1024;
const MAX_PROMPT_NAME_CHARS: usize = 160;
const TRASH_PREFIX: &str = "deleted-";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptEntry {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub relative_path: String,
    pub content: String,
    pub updated_at: i64,
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrashedPromptEntry {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub original_file_name: String,
    pub original_relative_path: String,
    pub deleted_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptRecord {
    pub id: String,
    pub relative_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trash_file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptState {
    pub version: u32,
    pub prompts: Vec<PromptRecord>,
}

impl Default for PromptState {
    fn default() -> Self {
        Self {
            version: 1,
            prompts: Vec::new(),
        }
    }
}

/// Lists prompts that can be attached to an AI request.
///
/// Empty Markdown files are deliberately omitted from this view, while
/// `list_prompt_files` keeps them visible to the workspace file browser.
pub fn list_prompts(workspace: &Workspace) -> Result<Vec<PromptEntry>> {
    Ok(list_prompt_files(workspace)?
        .into_iter()
        .filter(|prompt| !prompt.content.trim().is_empty())
        .collect())
}

pub fn list_prompt_files(workspace: &Workspace) -> Result<Vec<PromptEntry>> {
    reconcile_prompt_metadata(workspace)?;
    let state = load_prompt_state(workspace)?;
    let mut entries = state
        .prompts
        .iter()
        .filter(|record| record.deleted_at.is_none())
        .map(|record| read_prompt_record(workspace, record))
        .collect::<Result<Vec<_>>>()?;
    entries.sort_by(|left, right| {
        left.relative_path
            .to_lowercase()
            .cmp(&right.relative_path.to_lowercase())
    });
    Ok(entries)
}

pub fn read_prompt(workspace: &Workspace, prompt_id: &str) -> Result<PromptEntry> {
    reconcile_prompt_metadata(workspace)?;
    let state = load_prompt_state(workspace)?;
    let record = resolve_prompt_record(&state, prompt_id, false)?;
    read_prompt_record(workspace, record)
}

pub fn create_prompt(workspace: &Workspace, name: &str) -> Result<PromptEntry> {
    create_prompt_in_directory(workspace, name, "")
}

pub fn create_prompt_in_directory(
    workspace: &Workspace,
    name: &str,
    directory: &str,
) -> Result<PromptEntry> {
    fs::create_dir_all(&workspace.prompts_dir).with_context(|| {
        format!(
            "failed to create prompts directory at {}",
            workspace.prompts_dir.display()
        )
    })?;
    let file_name = prompt_file_name(name)?;
    let relative_path = prompt_target_relative_path(workspace, directory, &file_name)?;
    ensure_prompt_path_available(workspace, &relative_path, None)?;
    let path = prompt_relative_path(workspace, &relative_path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create prompt directory {}", parent.display()))?;
    }
    write_atomic(&path, "")?;
    let mut state = load_prompt_state(workspace)?;
    let id = unique_prompt_id(&state, now_ms()?);
    let record = PromptRecord {
        id,
        relative_path,
        deleted_at: None,
        trash_file_name: None,
    };
    state.prompts.push(record.clone());
    if let Err(error) = save_prompt_state(workspace, &state) {
        fs::remove_file(&path).with_context(|| {
            format!("failed to roll back prompt creation after metadata error: {error:#}")
        })?;
        return Err(error);
    }
    read_prompt_record(workspace, &record)
}

pub fn write_prompt_atomic_checked(
    workspace: &Workspace,
    prompt_id: &str,
    content: &str,
    expected_revision: &str,
) -> Result<PromptEntry> {
    if content.len() as u64 > MAX_PROMPT_BYTES {
        bail!("prompt content exceeds the 128 KiB limit");
    }
    let current = read_prompt(workspace, prompt_id)?;
    if current.revision != expected_revision {
        bail!("prompt was modified outside NeoPad: {prompt_id}");
    }
    let path = prompt_file_path(workspace, prompt_id)?;
    write_atomic(&path, content)?;
    read_prompt(workspace, prompt_id)
}

pub fn rename_prompt(workspace: &Workspace, prompt_id: &str, name: &str) -> Result<PromptEntry> {
    reconcile_prompt_metadata(workspace)?;
    let mut state = load_prompt_state(workspace)?;
    let record_index = prompt_record_index(&state, prompt_id, false)?;
    let record = state.prompts[record_index].clone();
    let source = prompt_relative_path(workspace, &record.relative_path)?;
    let source_metadata = fs::symlink_metadata(&source)
        .with_context(|| format!("failed to inspect prompt at {}", source.display()))?;
    if !source_metadata.file_type().is_file() {
        bail!("prompt is not a regular file: {prompt_id}");
    }

    let next_file_name = prompt_file_name(name)?;
    let directory = Path::new(&record.relative_path)
        .parent()
        .and_then(|path| path.to_str())
        .unwrap_or_default();
    let next_relative_path = prompt_target_relative_path(workspace, directory, &next_file_name)?;
    if next_relative_path == record.relative_path {
        return read_prompt_record(workspace, &record);
    }
    ensure_prompt_path_available(workspace, &next_relative_path, Some(&record.relative_path))?;
    let target = prompt_relative_path(workspace, &next_relative_path)?;
    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to rename prompt {} to {}",
            source.display(),
            target.display()
        )
    })?;
    state.prompts[record_index].relative_path = next_relative_path;
    if let Err(error) = save_prompt_state(workspace, &state) {
        fs::rename(&target, &source).with_context(|| {
            format!("failed to roll back prompt rename after metadata error: {error:#}")
        })?;
        return Err(error);
    }
    read_prompt_record(workspace, &state.prompts[record_index])
}

pub fn move_prompt(workspace: &Workspace, prompt_id: &str, directory: &str) -> Result<PromptEntry> {
    reconcile_prompt_metadata(workspace)?;
    let mut state = load_prompt_state(workspace)?;
    let record_index = prompt_record_index(&state, prompt_id, false)?;
    let record = state.prompts[record_index].clone();
    let file_name = Path::new(&record.relative_path)
        .file_name()
        .and_then(|name| name.to_str())
        .context("prompt file name is not valid UTF-8")?;
    let next_relative_path = prompt_target_relative_path(workspace, directory, file_name)?;
    if next_relative_path == record.relative_path {
        return read_prompt_record(workspace, &record);
    }
    ensure_prompt_path_available(workspace, &next_relative_path, Some(&record.relative_path))?;
    let source = prompt_relative_path(workspace, &record.relative_path)?;
    let target = prompt_relative_path(workspace, &next_relative_path)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create prompt directory {}", parent.display()))?;
    }
    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to move prompt {} to {}",
            source.display(),
            target.display()
        )
    })?;
    state.prompts[record_index].relative_path = next_relative_path;
    if let Err(error) = save_prompt_state(workspace, &state) {
        fs::rename(&target, &source).with_context(|| {
            format!("failed to roll back prompt move after metadata error: {error:#}")
        })?;
        return Err(error);
    }
    read_prompt_record(workspace, &state.prompts[record_index])
}

pub fn trash_prompt(workspace: &Workspace, prompt_id: &str) -> Result<TrashedPromptEntry> {
    let prompt = read_prompt(workspace, prompt_id)?;
    let mut state = load_prompt_state(workspace)?;
    let record_index = prompt_record_index(&state, prompt_id, false)?;
    let trash_dir = prompt_trash_dir(workspace);
    fs::create_dir_all(&trash_dir).with_context(|| {
        format!(
            "failed to create prompt trash directory at {}",
            trash_dir.display()
        )
    })?;
    ensure_inside_workspace(&workspace.root, &trash_dir)?;

    let deleted_at = now_ms()?;
    let deleted_file_name = format!("{TRASH_PREFIX}{deleted_at}-{}.md", prompt.id);
    let source = prompt_file_path(workspace, prompt_id)?;
    let target = prompt_trash_file_path(workspace, &deleted_file_name)?;
    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to move prompt {} to trash {}",
            source.display(),
            target.display()
        )
    })?;

    state.prompts[record_index].deleted_at = Some(deleted_at);
    state.prompts[record_index].trash_file_name = Some(deleted_file_name.clone());
    if let Err(error) = save_prompt_state(workspace, &state) {
        fs::rename(&target, &source).with_context(|| {
            format!("failed to roll back prompt trash after metadata error: {error:#}")
        })?;
        return Err(error);
    }

    Ok(TrashedPromptEntry {
        id: prompt.id,
        name: prompt.name,
        file_name: deleted_file_name,
        original_file_name: prompt.file_name,
        original_relative_path: prompt.relative_path,
        deleted_at,
    })
}

pub fn list_trashed_prompts(workspace: &Workspace) -> Result<Vec<TrashedPromptEntry>> {
    let trash_dir = prompt_trash_dir(workspace);
    if !trash_dir.exists() {
        return Ok(Vec::new());
    }
    ensure_inside_workspace(&workspace.root, &trash_dir)?;
    reconcile_prompt_metadata(workspace)?;
    let state = load_prompt_state(workspace)?;
    let mut entries = state
        .prompts
        .iter()
        .filter_map(|record| {
            let deleted_at = record.deleted_at?;
            let file_name = record.trash_file_name.clone()?;
            let original_file_name = Path::new(&record.relative_path)
                .file_name()?
                .to_str()?
                .to_owned();
            let name = Path::new(&original_file_name)
                .file_stem()?
                .to_str()?
                .to_owned();
            Some(TrashedPromptEntry {
                id: record.id.clone(),
                name,
                file_name,
                original_file_name,
                original_relative_path: record.relative_path.clone(),
                deleted_at,
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.file_name.cmp(&right.file_name))
    });
    Ok(entries)
}

pub fn restore_prompt_from_trash(
    workspace: &Workspace,
    trashed_prompt_id: &str,
) -> Result<PromptEntry> {
    reconcile_prompt_metadata(workspace)?;
    let mut state = load_prompt_state(workspace)?;
    let record_index = prompt_record_index(&state, trashed_prompt_id, true)?;
    let record = state.prompts[record_index].clone();
    ensure_prompt_path_available(workspace, &record.relative_path, None)?;
    let trash_file_name = record
        .trash_file_name
        .as_deref()
        .context("trashed prompt metadata has no trash file name")?;
    let source = prompt_trash_file_path(workspace, trash_file_name)?;
    let target = prompt_relative_path(workspace, &record.relative_path)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to recreate prompt directory {}", parent.display()))?;
    }
    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to restore prompt {} to {}",
            source.display(),
            target.display()
        )
    })?;
    state.prompts[record_index].deleted_at = None;
    state.prompts[record_index].trash_file_name = None;
    if let Err(error) = save_prompt_state(workspace, &state) {
        fs::rename(&target, &source).with_context(|| {
            format!("failed to return restored prompt to trash after metadata error: {error:#}")
        })?;
        return Err(error);
    }
    read_prompt_record(workspace, &state.prompts[record_index])
}

pub fn prompt_file_path(workspace: &Workspace, prompt_id: &str) -> Result<PathBuf> {
    let state = load_prompt_state(workspace)?;
    let record = resolve_prompt_record(&state, prompt_id, false)?;
    prompt_relative_path(workspace, &record.relative_path)
}

fn prompt_trash_dir(workspace: &Workspace) -> PathBuf {
    workspace.trash_dir.join("prompts")
}

fn prompt_trash_file_path(workspace: &Workspace, file_name: &str) -> Result<PathBuf> {
    validate_prompt_file_name(file_name)?;
    let trash_dir = prompt_trash_dir(workspace);
    let target = trash_dir.join(file_name);
    ensure_inside_workspace(&workspace.root, &target)?;
    Ok(target)
}

fn read_prompt_record(workspace: &Workspace, record: &PromptRecord) -> Result<PromptEntry> {
    let path = prompt_relative_path(workspace, &record.relative_path)?;
    ensure_inside_workspace(&workspace.prompts_dir, &path)?;
    let metadata = fs::symlink_metadata(&path)
        .with_context(|| format!("failed to inspect prompt at {}", path.display()))?;
    if !metadata.file_type().is_file() {
        bail!("prompt is not a regular file: {}", path.display());
    }
    if metadata.len() > MAX_PROMPT_BYTES {
        bail!(
            "prompt file exceeds the 128 KiB limit: {}",
            path.file_name()
                .map(|name| name.to_string_lossy())
                .unwrap_or_default()
        );
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .context("prompt file name is not valid UTF-8")?
        .to_owned();
    validate_prompt_file_name(&file_name)?;
    let name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .context("prompt name is not valid UTF-8")?
        .trim()
        .to_owned();
    if name.is_empty() {
        bail!("prompt name cannot be empty");
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read prompt at {}", path.display()))?;
    let updated_at = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map_or(now_ms()?, |duration| duration.as_millis() as i64);

    Ok(PromptEntry {
        id: record.id.clone(),
        name,
        file_name,
        relative_path: record.relative_path.clone(),
        revision: content_revision(&content),
        content,
        updated_at,
    })
}

fn markdown_files(directory: &Path, label: &str) -> Result<Vec<PathBuf>> {
    fs::read_dir(directory)
        .with_context(|| {
            format!(
                "failed to read {label} directory at {}",
                directory.display()
            )
        })?
        .filter_map(|entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                let is_markdown = path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));
                is_markdown.then_some(Ok(path))
            }
            Err(error) => Some(Err(error)),
        })
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn markdown_files_recursive(directory: &Path, label: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut pending = vec![directory.to_path_buf()];
    while let Some(current) = pending.pop() {
        for entry in fs::read_dir(&current)
            .with_context(|| format!("failed to read {label} directory at {}", current.display()))?
        {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
            {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    Ok(files)
}

fn ensure_prompt_path_available(
    workspace: &Workspace,
    relative_path: &str,
    current_relative_path: Option<&str>,
) -> Result<()> {
    let candidate = relative_path.to_lowercase();
    let current = current_relative_path.map(str::to_lowercase);
    for path in markdown_files_recursive(&workspace.prompts_dir, "prompts")? {
        let existing = relative_path_string(&workspace.prompts_dir, &path)?;
        let existing_lower = existing.to_lowercase();
        if existing_lower == candidate && current.as_deref() != Some(existing_lower.as_str()) {
            bail!("a prompt already exists at {relative_path}");
        }
    }
    Ok(())
}

pub fn reconcile_prompt_metadata(workspace: &Workspace) -> Result<bool> {
    let mut state = load_prompt_state(workspace)?;
    let files = markdown_files_recursive(&workspace.prompts_dir, "prompts")?;
    let mut unmatched = files
        .iter()
        .map(|path| relative_path_string(&workspace.prompts_dir, path))
        .collect::<Result<Vec<_>>>()?;
    let mut changed = false;
    let mut stale_ids = Vec::new();

    for record in state
        .prompts
        .iter_mut()
        .filter(|record| record.deleted_at.is_none())
    {
        if let Some(index) = unmatched
            .iter()
            .position(|path| path.eq_ignore_ascii_case(&record.relative_path))
        {
            unmatched.remove(index);
            continue;
        }
        let file_name = Path::new(&record.relative_path).file_name();
        let matches = unmatched
            .iter()
            .enumerate()
            .filter(|(_, path)| Path::new(path).file_name() == file_name)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            record.relative_path = unmatched.remove(matches[0]);
            changed = true;
        } else {
            stale_ids.push(record.id.clone());
        }
    }
    if !stale_ids.is_empty() {
        state
            .prompts
            .retain(|record| !stale_ids.contains(&record.id));
        changed = true;
    }

    for relative_path in unmatched {
        let id = unique_prompt_id(&state, now_ms()?);
        state.prompts.push(PromptRecord {
            id,
            relative_path,
            deleted_at: None,
            trash_file_name: None,
        });
        changed = true;
    }

    let trash_dir = prompt_trash_dir(workspace);
    let missing_deleted_ids = state
        .prompts
        .iter()
        .filter(|record| record.deleted_at.is_some())
        .filter(|record| {
            record
                .trash_file_name
                .as_deref()
                .is_none_or(|name| !trash_dir.join(name).is_file())
        })
        .map(|record| record.id.clone())
        .collect::<Vec<_>>();
    if !missing_deleted_ids.is_empty() {
        state
            .prompts
            .retain(|record| !missing_deleted_ids.contains(&record.id));
        changed = true;
    }

    if trash_dir.is_dir() {
        let tracked = state
            .prompts
            .iter()
            .filter_map(|record| record.trash_file_name.clone())
            .collect::<Vec<_>>();
        for path in markdown_files(&trash_dir, "prompt trash")? {
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if tracked.iter().any(|tracked| tracked == file_name) {
                continue;
            }
            let Some((deleted_at, original_file_name)) = parse_trashed_prompt_file_name(file_name)
            else {
                continue;
            };
            state.prompts.push(PromptRecord {
                id: unique_prompt_id(&state, deleted_at),
                relative_path: original_file_name,
                deleted_at: Some(deleted_at),
                trash_file_name: Some(file_name.to_owned()),
            });
            changed = true;
        }
    }

    if changed {
        save_prompt_state(workspace, &state)?;
    }
    Ok(changed)
}

pub fn list_prompt_directories(workspace: &Workspace) -> Result<Vec<String>> {
    list_relative_directories(&workspace.prompts_dir)
}

pub fn create_prompt_directory(workspace: &Workspace, relative_path: &str) -> Result<String> {
    let normalized = normalized_directory(relative_path)?;
    let path = prompt_directory_path(workspace, &normalized)?;
    fs::create_dir_all(&path)
        .with_context(|| format!("failed to create prompt directory {}", path.display()))?;
    Ok(normalized)
}

pub fn move_prompt_directory(
    workspace: &Workspace,
    relative_path: &str,
    target_parent: &str,
) -> Result<String> {
    let source = normalized_directory(relative_path)?;
    let name = Path::new(&source)
        .file_name()
        .and_then(|value| value.to_str())
        .context("prompt directory name is not valid UTF-8")?;
    let target = if target_parent.trim().is_empty() {
        name.to_owned()
    } else {
        format!("{}/{}", normalized_directory(target_parent)?, name)
    };
    relocate_prompt_directory(workspace, &source, &target)
}

pub fn rename_prompt_directory(
    workspace: &Workspace,
    relative_path: &str,
    new_name: &str,
) -> Result<String> {
    let source = normalized_directory(relative_path)?;
    let name = normalized_directory_name(new_name)?;
    let target = match Path::new(&source).parent().and_then(Path::to_str) {
        Some(parent) if !parent.is_empty() => format!("{parent}/{name}"),
        _ => name,
    };
    relocate_prompt_directory(workspace, &source, &target)
}

pub fn delete_prompt_directory_to_trash(
    workspace: &Workspace,
    relative_path: &str,
) -> Result<usize> {
    let relative_path = normalized_directory(relative_path)?;
    let directory = prompt_directory_path(workspace, &relative_path)?;
    ensure_regular_directory(&directory, "prompt")?;
    reconcile_prompt_metadata(workspace)?;

    let state = load_prompt_state(workspace)?;
    let prompt_ids = state
        .prompts
        .iter()
        .filter(|record| {
            record.deleted_at.is_none()
                && path_is_inside_directory(&record.relative_path, &relative_path)
        })
        .map(|record| record.id.clone())
        .collect::<Vec<_>>();
    let managed_paths = state
        .prompts
        .iter()
        .filter(|record| record.deleted_at.is_none())
        .map(|record| record.relative_path.clone())
        .collect::<std::collections::HashSet<_>>();

    for file in regular_files_recursive_strict(&directory)? {
        let path = relative_path_string(&workspace.prompts_dir, &file)?;
        if !managed_paths.contains(&path) {
            bail!("prompt directory contains an unmanaged file: {path}");
        }
    }

    for prompt_id in &prompt_ids {
        trash_prompt(workspace, prompt_id)?;
    }
    fs::remove_dir_all(&directory)
        .with_context(|| format!("failed to remove prompt directory {}", directory.display()))?;
    Ok(prompt_ids.len())
}

fn relocate_prompt_directory(
    workspace: &Workspace,
    source_relative_path: &str,
    target_relative_path: &str,
) -> Result<String> {
    let source_relative_path = normalized_directory(source_relative_path)?;
    let target_relative_path = normalized_directory(target_relative_path)?;
    if source_relative_path == target_relative_path {
        return Ok(source_relative_path);
    }
    if path_is_inside_directory(&target_relative_path, &source_relative_path) {
        bail!("cannot move a prompt directory into itself");
    }

    let source = prompt_directory_path(workspace, &source_relative_path)?;
    let target = prompt_directory_path(workspace, &target_relative_path)?;
    ensure_regular_directory(&source, "prompt")?;
    if target.exists() {
        bail!("prompt directory already exists: {target_relative_path}");
    }
    let target_parent = target.parent().context("prompt directory has no parent")?;
    ensure_regular_directory(target_parent, "prompt target parent")?;

    reconcile_prompt_metadata(workspace)?;
    let mut state = load_prompt_state(workspace)?;
    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to move prompt directory {} to {}",
            source.display(),
            target.display()
        )
    })?;
    for record in &mut state.prompts {
        if let Some(next) = replaced_directory_prefix(
            &record.relative_path,
            &source_relative_path,
            &target_relative_path,
        ) {
            record.relative_path = next;
        }
    }
    if let Err(error) = save_prompt_state(workspace, &state) {
        fs::rename(&target, &source).with_context(|| {
            format!("failed to roll back prompt directory move after metadata error: {error:#}")
        })?;
        return Err(error);
    }
    Ok(target_relative_path)
}

fn normalized_directory_name(name: &str) -> Result<String> {
    let normalized = normalized_directory(name.trim())?;
    if normalized.contains('/') {
        bail!("directory name must not contain path separators");
    }
    Ok(normalized)
}

fn ensure_regular_directory(path: &Path, label: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect {label} directory {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!(
            "{label} path is not a regular directory: {}",
            path.display()
        );
    }
    Ok(())
}

fn path_is_inside_directory(path: &str, directory: &str) -> bool {
    path.strip_prefix(directory)
        .is_some_and(|suffix| suffix.starts_with('/'))
}

fn replaced_directory_prefix(path: &str, source: &str, target: &str) -> Option<String> {
    path.strip_prefix(source)
        .filter(|suffix| suffix.starts_with('/'))
        .map(|suffix| format!("{target}{suffix}"))
}

fn regular_files_recursive_strict(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut pending = vec![directory.to_path_buf()];
    while let Some(current) = pending.pop() {
        for entry in fs::read_dir(&current)
            .with_context(|| format!("failed to read directory {}", current.display()))?
        {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                bail!(
                    "directory contains a symbolic link: {}",
                    entry.path().display()
                );
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                files.push(entry.path());
            } else {
                bail!(
                    "directory contains an unsupported entry: {}",
                    entry.path().display()
                );
            }
        }
    }
    files.sort();
    Ok(files)
}

fn list_relative_directories(root: &Path) -> Result<Vec<String>> {
    let mut directories = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(current) = pending.pop() {
        for entry in fs::read_dir(&current)
            .with_context(|| format!("failed to read directory {}", current.display()))?
        {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() && !file_type.is_symlink() {
                let path = entry.path();
                directories.push(relative_path_string(root, &path)?);
                pending.push(path);
            }
        }
    }
    directories.sort_by_key(|path| path.to_lowercase());
    Ok(directories)
}

fn load_prompt_state(workspace: &Workspace) -> Result<PromptState> {
    let contents = fs::read_to_string(&workspace.prompts_meta_path).with_context(|| {
        format!(
            "failed to read prompt metadata {}",
            workspace.prompts_meta_path.display()
        )
    })?;
    serde_json::from_str(&contents).with_context(|| {
        format!(
            "failed to parse prompt metadata {}",
            workspace.prompts_meta_path.display()
        )
    })
}

fn save_prompt_state(workspace: &Workspace, state: &PromptState) -> Result<()> {
    let contents =
        serde_json::to_string_pretty(state).context("failed to serialize prompt metadata")?;
    write_atomic(&workspace.prompts_meta_path, &contents)
}

fn resolve_prompt_record<'a>(
    state: &'a PromptState,
    prompt_id: &str,
    deleted: bool,
) -> Result<&'a PromptRecord> {
    state
        .prompts
        .iter()
        .find(|record| {
            record.deleted_at.is_some() == deleted
                && (record.id == prompt_id
                    || (!deleted && record.relative_path.eq_ignore_ascii_case(prompt_id)))
        })
        .with_context(|| format!("prompt not found: {prompt_id}"))
}

fn prompt_record_index(state: &PromptState, prompt_id: &str, deleted: bool) -> Result<usize> {
    state
        .prompts
        .iter()
        .position(|record| {
            record.deleted_at.is_some() == deleted
                && (record.id == prompt_id
                    || (!deleted && record.relative_path.eq_ignore_ascii_case(prompt_id)))
        })
        .with_context(|| format!("prompt not found: {prompt_id}"))
}

fn unique_prompt_id(state: &PromptState, now: i64) -> String {
    let mut suffix = 0_u32;
    loop {
        let id = if suffix == 0 {
            format!("prompt-{now}")
        } else {
            format!("prompt-{now}-{suffix}")
        };
        if state.prompts.iter().all(|record| record.id != id) {
            return id;
        }
        suffix += 1;
    }
}

fn prompt_target_relative_path(
    workspace: &Workspace,
    directory: &str,
    file_name: &str,
) -> Result<String> {
    if directory.trim().is_empty() {
        prompt_relative_path(workspace, file_name)?;
        return Ok(file_name.to_owned());
    }
    let directory = normalized_directory(directory)?;
    prompt_directory_path(workspace, &directory)?;
    Ok(format!("{directory}/{file_name}"))
}

fn normalized_directory(relative_path: &str) -> Result<String> {
    let normalized = Path::new(relative_path)
        .components()
        .map(|component| {
            let Component::Normal(value) = component else {
                bail!("directory contains unsafe path components");
            };
            value
                .to_str()
                .context("directory path is not valid UTF-8")
                .map(str::to_owned)
        })
        .collect::<Result<Vec<_>>>()?
        .join("/");
    if normalized.is_empty() {
        bail!("directory cannot be empty");
    }
    Ok(normalized)
}

fn relative_path_string(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .with_context(|| format!("path is outside {}: {}", root.display(), path.display()))?;
    relative
        .components()
        .map(|component| {
            let Component::Normal(value) = component else {
                bail!("relative path contains unsafe components");
            };
            value
                .to_str()
                .context("relative path is not valid UTF-8")
                .map(str::to_owned)
        })
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.join("/"))
}

fn prompt_file_name(name: &str) -> Result<String> {
    let mut name = name.trim();
    if name.to_ascii_lowercase().ends_with(".md") {
        name = &name[..name.len() - 3];
        name = name.trim_end();
    }
    validate_prompt_name(name)?;
    Ok(format!("{name}.md"))
}

fn validate_prompt_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("prompt name cannot be empty");
    }
    if name.chars().count() > MAX_PROMPT_NAME_CHARS {
        bail!("prompt name is too long");
    }
    if name.ends_with(['.', ' ']) {
        bail!("prompt name cannot end with a dot or space");
    }
    if name.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
    }) {
        bail!("prompt name contains characters that are not allowed in file names");
    }
    let reserved = name
        .trim_end_matches('.')
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let is_reserved = matches!(reserved.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (reserved.len() == 4
            && matches!(&reserved[..3], "COM" | "LPT")
            && matches!(reserved.as_bytes()[3], b'1'..=b'9'));
    if is_reserved {
        bail!("prompt name is reserved by the operating system");
    }
    Ok(())
}

fn validate_prompt_file_name(file_name: &str) -> Result<()> {
    let path = Path::new(file_name);
    if file_name.trim().is_empty() {
        bail!("prompt file name cannot be empty");
    }
    if path.is_absolute() {
        bail!("prompt file name cannot be an absolute path");
    }
    if path.components().count() != 1 {
        bail!("prompt file name cannot include path separators");
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        bail!("prompt file name cannot include relative path components");
    }
    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
    {
        bail!("prompt file name must end with .md");
    }
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .context("prompt file name is not valid UTF-8")?;
    validate_prompt_name(stem)?;
    Ok(())
}

fn parse_trashed_prompt_file_name(file_name: &str) -> Option<(i64, String)> {
    validate_prompt_file_name(file_name).ok()?;
    let remainder = file_name.strip_prefix(TRASH_PREFIX)?;
    let (timestamp, original_file_name) = remainder.split_once('-')?;
    let deleted_at = timestamp.parse().ok()?;
    validate_prompt_file_name(original_file_name).ok()?;
    Some((deleted_at, original_file_name.to_owned()))
}

fn content_revision(content: &str) -> String {
    format!("{:x}", Sha256::digest(content.as_bytes()))
}

fn now_ms() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before unix epoch")?;
    Ok(duration.as_millis() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_workspace;

    #[test]
    fn lists_markdown_prompts_in_name_order() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        fs::write(
            workspace.prompts_dir.join("Weekly.md"),
            "Create a weekly report.",
        )
        .expect("weekly prompt");
        fs::write(workspace.prompts_dir.join("Review.md"), "Review this note.")
            .expect("review prompt");
        fs::write(workspace.prompts_dir.join("ignore.txt"), "ignored").expect("ignored file");

        let prompts = list_prompts(&workspace).expect("list prompts");

        assert_eq!(prompts.len(), 2);
        assert_eq!(prompts[0].name, "Review");
        assert_eq!(prompts[1].file_name, "Weekly.md");
        assert_eq!(prompts[1].relative_path, "Weekly.md");
        assert!(prompts[1].id.starts_with("prompt-"));
        assert_eq!(prompts[1].content, "Create a weekly report.");
        assert!(!prompts[1].revision.is_empty());
    }

    #[test]
    fn management_list_keeps_empty_prompt_files() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        fs::write(workspace.prompts_dir.join("Empty.md"), "  \n").expect("empty prompt");

        assert!(list_prompts(&workspace).expect("usable prompts").is_empty());
        let files = list_prompt_files(&workspace).expect("prompt files");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "Empty");
    }

    #[test]
    fn creates_writes_renames_trashes_and_restores_prompt() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");

        let created = create_prompt(&workspace, "Review").expect("create prompt");
        assert_eq!(created.file_name, "Review.md");
        let written = write_prompt_atomic_checked(
            &workspace,
            &created.id,
            "Review this note.",
            &created.revision,
        )
        .expect("write prompt");
        let renamed =
            rename_prompt(&workspace, &written.id, "Careful review").expect("rename prompt");
        assert_eq!(renamed.file_name, "Careful review.md");
        assert_eq!(renamed.content, "Review this note.");

        let trashed = trash_prompt(&workspace, &renamed.id).expect("trash prompt");
        assert!(list_prompt_files(&workspace)
            .expect("prompt files")
            .is_empty());
        assert_eq!(
            list_trashed_prompts(&workspace).expect("trash"),
            vec![trashed.clone()]
        );

        let restored = restore_prompt_from_trash(&workspace, &trashed.id).expect("restore prompt");
        assert_eq!(restored.file_name, "Careful review.md");
        assert_eq!(restored.content, "Review this note.");
    }

    #[test]
    fn checked_write_rejects_external_edits() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        let created = create_prompt(&workspace, "Review").expect("create prompt");
        fs::write(
            workspace.prompts_dir.join(&created.file_name),
            "changed outside NeoPad",
        )
        .expect("external write");

        let error = write_prompt_atomic_checked(
            &workspace,
            &created.id,
            "stale editor content",
            &created.revision,
        )
        .expect_err("external edit must be rejected");
        assert!(error.to_string().contains("outside NeoPad"));
    }

    #[test]
    fn rejects_unsafe_and_duplicate_prompt_names() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        create_prompt(&workspace, "Review").expect("create prompt");

        for name in [
            "../escape",
            "nested/name",
            "CON",
            "CON.txt",
            "bad:name",
            "Review.",
        ] {
            assert!(
                create_prompt(&workspace, name).is_err(),
                "{name} should fail"
            );
        }
        assert!(create_prompt(&workspace, "review").is_err());
        assert!(read_prompt(&workspace, "../Review.md").is_err());
    }

    #[test]
    fn prompt_directories_preserve_stable_ids_across_moves_and_trash() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        create_prompt_directory(&workspace, "coding/rust").expect("create directory");
        let created = create_prompt_in_directory(&workspace, "Review", "coding/rust")
            .expect("create nested prompt");
        assert_eq!(created.relative_path, "coding/rust/Review.md");

        create_prompt_directory(&workspace, "writing").expect("create target directory");
        let moved = move_prompt(&workspace, &created.id, "writing").expect("move prompt");
        assert_eq!(moved.id, created.id);
        assert_eq!(moved.relative_path, "writing/Review.md");

        let trashed = trash_prompt(&workspace, &created.id).expect("trash prompt");
        assert_eq!(trashed.id, created.id);
        assert_eq!(trashed.original_relative_path, "writing/Review.md");
        let restored = restore_prompt_from_trash(&workspace, &created.id).expect("restore prompt");
        assert_eq!(restored.id, created.id);
        assert_eq!(restored.relative_path, "writing/Review.md");
    }

    #[test]
    fn prompt_directories_can_be_moved_and_renamed_with_stable_ids() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        let first =
            create_prompt_in_directory(&workspace, "Review", "coding/rust").expect("first prompt");
        let second = create_prompt_in_directory(&workspace, "Explain", "coding/rust/deep")
            .expect("second prompt");
        create_prompt_directory(&workspace, "reference").expect("target parent");

        let moved =
            move_prompt_directory(&workspace, "coding/rust", "reference").expect("move directory");
        assert_eq!(moved, "reference/rust");
        let renamed =
            rename_prompt_directory(&workspace, &moved, "systems").expect("rename directory");
        assert_eq!(renamed, "reference/systems");
        let first_after = read_prompt(&workspace, &first.id).expect("first after move");
        let second_after = read_prompt(&workspace, &second.id).expect("second after move");
        assert_eq!(first_after.id, first.id);
        assert_eq!(first_after.relative_path, "reference/systems/Review.md");
        assert_eq!(second_after.id, second.id);
        assert_eq!(
            second_after.relative_path,
            "reference/systems/deep/Explain.md"
        );
        assert!(move_prompt_directory(&workspace, "reference", "reference/systems").is_err());
    }

    #[test]
    fn deleting_prompt_directory_trashes_prompts_and_restore_recreates_it() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        let prompt =
            create_prompt_in_directory(&workspace, "Review", "old/project").expect("prompt");

        assert_eq!(
            delete_prompt_directory_to_trash(&workspace, "old").expect("delete directory"),
            1
        );
        assert!(!workspace.prompts_dir.join("old").exists());
        assert!(list_trashed_prompts(&workspace)
            .expect("trash")
            .iter()
            .any(|entry| entry.id == prompt.id));

        let restored = restore_prompt_from_trash(&workspace, &prompt.id).expect("restore prompt");
        assert_eq!(restored.relative_path, "old/project/Review.md");
        assert!(workspace
            .prompts_dir
            .join("old/project/Review.md")
            .is_file());
    }

    #[test]
    fn prompt_reconciliation_tracks_external_directory_moves_by_file_name() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        let created =
            create_prompt_in_directory(&workspace, "Review", "coding").expect("create prompt");
        fs::create_dir_all(workspace.prompts_dir.join("writing")).expect("target directory");
        fs::rename(
            workspace.prompts_dir.join("coding/Review.md"),
            workspace.prompts_dir.join("writing/Review.md"),
        )
        .expect("external move");

        reconcile_prompt_metadata(&workspace).expect("reconcile");
        let moved = read_prompt(&workspace, &created.id).expect("stable prompt");

        assert_eq!(moved.id, created.id);
        assert_eq!(moved.relative_path, "writing/Review.md");
    }
}
