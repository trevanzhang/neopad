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
    validate_note_file_name(file_name)?;
    let target = workspace.archive_dir.join(file_name);
    ensure_inside_workspace(&workspace.root, &target)?;
    Ok(target)
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
}
