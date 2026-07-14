use crate::{atomic_write::write_atomic, path::ensure_inside_workspace, Workspace};
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
    pub deleted_at: i64,
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
    let mut paths = markdown_files(&workspace.prompts_dir, "prompts")?;
    sort_paths_by_file_name(&mut paths);
    paths
        .into_iter()
        .map(|path| read_prompt_path(workspace, &path))
        .collect()
}

pub fn read_prompt(workspace: &Workspace, prompt_id: &str) -> Result<PromptEntry> {
    let path = prompt_file_path(workspace, prompt_id)?;
    read_prompt_path(workspace, &path)
}

pub fn create_prompt(workspace: &Workspace, name: &str) -> Result<PromptEntry> {
    fs::create_dir_all(&workspace.prompts_dir).with_context(|| {
        format!(
            "failed to create prompts directory at {}",
            workspace.prompts_dir.display()
        )
    })?;
    let file_name = prompt_file_name(name)?;
    ensure_prompt_name_available(workspace, &file_name, None)?;
    let path = prompt_file_path(workspace, &file_name)?;
    write_atomic(&path, "")?;
    read_prompt_path(workspace, &path)
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
    read_prompt_path(workspace, &path)
}

pub fn rename_prompt(workspace: &Workspace, prompt_id: &str, name: &str) -> Result<PromptEntry> {
    let source = prompt_file_path(workspace, prompt_id)?;
    let source_metadata = fs::symlink_metadata(&source)
        .with_context(|| format!("failed to inspect prompt at {}", source.display()))?;
    if !source_metadata.file_type().is_file() {
        bail!("prompt is not a regular file: {prompt_id}");
    }

    let next_file_name = prompt_file_name(name)?;
    if next_file_name == prompt_id {
        return read_prompt_path(workspace, &source);
    }
    ensure_prompt_name_available(workspace, &next_file_name, Some(prompt_id))?;
    let target = prompt_file_path(workspace, &next_file_name)?;
    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to rename prompt {} to {}",
            source.display(),
            target.display()
        )
    })?;
    read_prompt_path(workspace, &target)
}

pub fn trash_prompt(workspace: &Workspace, prompt_id: &str) -> Result<TrashedPromptEntry> {
    let prompt = read_prompt(workspace, prompt_id)?;
    let trash_dir = prompt_trash_dir(workspace);
    fs::create_dir_all(&trash_dir).with_context(|| {
        format!(
            "failed to create prompt trash directory at {}",
            trash_dir.display()
        )
    })?;
    ensure_inside_workspace(&workspace.root, &trash_dir)?;

    let deleted_at = now_ms()?;
    let deleted_file_name = format!("{TRASH_PREFIX}{deleted_at}-{}", prompt.file_name);
    let source = prompt_file_path(workspace, prompt_id)?;
    let target = prompt_trash_file_path(workspace, &deleted_file_name)?;
    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to move prompt {} to trash {}",
            source.display(),
            target.display()
        )
    })?;

    Ok(TrashedPromptEntry {
        id: deleted_file_name.clone(),
        name: prompt.name,
        file_name: deleted_file_name,
        original_file_name: prompt.file_name,
        deleted_at,
    })
}

pub fn list_trashed_prompts(workspace: &Workspace) -> Result<Vec<TrashedPromptEntry>> {
    let trash_dir = prompt_trash_dir(workspace);
    if !trash_dir.exists() {
        return Ok(Vec::new());
    }
    ensure_inside_workspace(&workspace.root, &trash_dir)?;
    let mut entries = markdown_files(&trash_dir, "prompt trash")?
        .into_iter()
        .filter_map(|path| {
            let file_name = path.file_name()?.to_str()?.to_owned();
            let (deleted_at, original_file_name) = parse_trashed_prompt_file_name(&file_name)?;
            let name = Path::new(&original_file_name)
                .file_stem()?
                .to_str()?
                .to_owned();
            Some(TrashedPromptEntry {
                id: file_name.clone(),
                name,
                file_name,
                original_file_name,
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
    let (_, original_file_name) = parse_trashed_prompt_file_name(trashed_prompt_id)
        .with_context(|| format!("trashed prompt name is invalid: {trashed_prompt_id}"))?;
    ensure_prompt_name_available(workspace, &original_file_name, None)?;
    let source = prompt_trash_file_path(workspace, trashed_prompt_id)?;
    let target = prompt_file_path(workspace, &original_file_name)?;
    fs::rename(&source, &target).with_context(|| {
        format!(
            "failed to restore prompt {} to {}",
            source.display(),
            target.display()
        )
    })?;
    read_prompt_path(workspace, &target)
}

pub fn prompt_file_path(workspace: &Workspace, file_name: &str) -> Result<PathBuf> {
    validate_prompt_file_name(file_name)?;
    let target = workspace.prompts_dir.join(file_name);
    ensure_inside_workspace(&workspace.root, &target)?;
    Ok(target)
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

fn read_prompt_path(workspace: &Workspace, path: &Path) -> Result<PromptEntry> {
    ensure_inside_workspace(&workspace.prompts_dir, path)?;
    let metadata = fs::symlink_metadata(path)
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
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read prompt at {}", path.display()))?;
    let updated_at = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map_or(now_ms()?, |duration| duration.as_millis() as i64);

    Ok(PromptEntry {
        id: file_name.clone(),
        name,
        file_name,
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

fn sort_paths_by_file_name(paths: &mut [PathBuf]) {
    paths.sort_by_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    });
}

fn ensure_prompt_name_available(
    workspace: &Workspace,
    file_name: &str,
    current_file_name: Option<&str>,
) -> Result<()> {
    let candidate = file_name.to_lowercase();
    let current = current_file_name.map(str::to_lowercase);
    for path in markdown_files(&workspace.prompts_dir, "prompts")? {
        let Some(existing) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let existing_lower = existing.to_lowercase();
        if existing_lower == candidate && current.as_deref() != Some(existing_lower.as_str()) {
            bail!("a prompt named {file_name} already exists");
        }
    }
    Ok(())
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
        assert_eq!(prompts[1].id, "Weekly.md");
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
}
