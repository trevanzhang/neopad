use crate::Workspace;
use anyhow::{bail, Context, Result};
use std::path::{Component, Path, PathBuf};

pub fn note_file_path(workspace: &Workspace, file_name: &str) -> Result<PathBuf> {
    validate_note_file_name(file_name)?;
    let target = workspace.notes_dir.join(file_name);
    ensure_inside_workspace(&workspace.root, &target)?;
    Ok(target)
}

pub fn trash_file_path(workspace: &Workspace, file_name: &str) -> Result<PathBuf> {
    validate_note_file_name(file_name)?;
    let target = workspace.trash_dir.join(file_name);
    ensure_inside_workspace(&workspace.root, &target)?;
    Ok(target)
}

pub fn archive_file_path(workspace: &Workspace, file_name: &str) -> Result<PathBuf> {
    safe_relative_path(&workspace.archive_dir, file_name, true)
}

pub fn archive_directory_path(workspace: &Workspace, relative_path: &str) -> Result<PathBuf> {
    safe_relative_path(&workspace.archive_dir, relative_path, false)
}

pub fn prompt_relative_path(workspace: &Workspace, relative_path: &str) -> Result<PathBuf> {
    safe_relative_path(&workspace.prompts_dir, relative_path, true)
}

pub fn prompt_directory_path(workspace: &Workspace, relative_path: &str) -> Result<PathBuf> {
    safe_relative_path(&workspace.prompts_dir, relative_path, false)
}

pub fn ensure_inside_workspace(workspace: &Path, target: &Path) -> Result<()> {
    let workspace = normalize_existing_path(workspace)
        .with_context(|| format!("failed to resolve workspace path {}", workspace.display()))?;
    let target = normalize_maybe_missing_path(target)
        .with_context(|| format!("failed to resolve target path {}", target.display()))?;

    if !target.starts_with(&workspace) {
        bail!("path is outside workspace: {}", target.display());
    }

    Ok(())
}

fn validate_note_file_name(file_name: &str) -> Result<()> {
    let path = Path::new(file_name);
    if file_name.trim().is_empty() {
        bail!("note file name cannot be empty");
    }
    if path.is_absolute() {
        bail!("note file name cannot be an absolute path");
    }
    if path.components().count() != 1 {
        bail!("note file name cannot include path separators");
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        bail!("note file name cannot include relative path components");
    }
    if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
        bail!("note file name must end with .md");
    }

    Ok(())
}

fn safe_relative_path(root: &Path, relative_path: &str, markdown_file: bool) -> Result<PathBuf> {
    let relative = Path::new(relative_path);
    if relative_path.trim().is_empty() {
        bail!("relative path cannot be empty");
    }
    if relative.is_absolute() {
        bail!("relative path cannot be absolute");
    }
    if relative.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::CurDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        bail!("relative path contains unsafe components");
    }
    if markdown_file
        && !relative
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
    {
        bail!("relative file path must end with .md");
    }

    for component in relative.components() {
        let Component::Normal(name) = component else {
            bail!("relative path contains unsafe components");
        };
        let name = name.to_str().context("relative path is not valid UTF-8")?;
        validate_path_component(name)?;
    }

    let target = root.join(relative);
    ensure_inside_existing_root(root, &target)?;
    Ok(target)
}

fn validate_path_component(name: &str) -> Result<()> {
    if name.is_empty() || name.ends_with(['.', ' ']) {
        bail!("path component is empty or ends with a dot or space");
    }
    if name.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
    }) {
        bail!("path component contains characters that are not allowed in file names");
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
        bail!("path component is reserved by the operating system");
    }
    Ok(())
}

fn ensure_inside_existing_root(root: &Path, target: &Path) -> Result<()> {
    let canonical_root = normalize_existing_path(root)
        .with_context(|| format!("failed to resolve workspace path {}", root.display()))?;
    let relative = target
        .strip_prefix(root)
        .context("target path is outside its workspace root")?;
    let mut current = canonical_root.clone();
    for component in relative.components() {
        current.push(component.as_os_str());
        if current.exists() {
            let metadata = std::fs::symlink_metadata(&current)
                .with_context(|| format!("failed to inspect path {}", current.display()))?;
            if metadata.file_type().is_symlink() {
                bail!("symbolic links are not allowed in workspace paths");
            }
            current = normalize_existing_path(&current)?;
            if !current.starts_with(&canonical_root) {
                bail!("path is outside workspace: {}", target.display());
            }
        }
    }
    Ok(())
}

fn normalize_existing_path(path: &Path) -> Result<PathBuf> {
    path.canonicalize().map_err(Into::into)
}

fn normalize_maybe_missing_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return normalize_existing_path(path);
    }

    let parent = path
        .parent()
        .context("target path must have a parent directory")?;
    let parent = normalize_existing_path(parent)?;
    let file_name = path
        .file_name()
        .context("target path must include a file name")?;
    Ok(parent.join(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_workspace;

    #[test]
    fn note_file_path_rejects_unsafe_names() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");

        for file_name in ["../x.md", "nested/x.md", "x.txt", "", "/abs.md", "."] {
            assert!(
                note_file_path(&workspace, file_name).is_err(),
                "{file_name} should be rejected"
            );
        }
    }

    #[test]
    fn note_file_path_accepts_plain_markdown_file_name() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");

        let path = note_file_path(&workspace, "inbox.md").expect("note path");

        assert_eq!(path, workspace.notes_dir.join("inbox.md"));
    }

    #[test]
    fn archive_paths_allow_safe_nested_markdown_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");

        let path = archive_file_path(&workspace, "work/client/note.md").expect("nested path");

        assert_eq!(path, workspace.archive_dir.join("work/client/note.md"));
        assert!(archive_file_path(&workspace, "../note.md").is_err());
        assert!(archive_file_path(&workspace, "work/CON/note.md").is_err());
    }
}
