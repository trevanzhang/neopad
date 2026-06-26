use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("target path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create parent directory {}", parent.display()))?;

    let tmp_path = temp_path(path)?;
    fs::write(&tmp_path, contents)
        .with_context(|| format!("failed to write temp file {}", tmp_path.display()))?;

    match replace_file(&tmp_path, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&tmp_path);
            Err(error).with_context(|| {
                format!(
                    "failed to move temp file {} to {}",
                    tmp_path.display(),
                    path.display()
                )
            })
        }
    }
}

fn replace_file(tmp_path: &Path, target_path: &Path) -> std::io::Result<()> {
    match fs::rename(tmp_path, target_path) {
        Ok(()) => Ok(()),
        Err(first_error) if target_path.exists() => {
            fs::remove_file(target_path)?;
            fs::rename(tmp_path, target_path).map_err(|_| first_error)
        }
        Err(error) => Err(error),
    }
}

fn temp_path(path: &Path) -> Result<PathBuf> {
    let file_name = path
        .file_name()
        .with_context(|| format!("target path has no file name: {}", path.display()))?
        .to_string_lossy();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before unix epoch")?
        .as_nanos();

    Ok(path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), now)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_creates_or_replaces_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("note.md");

        write_atomic(&path, "first").expect("first write");
        write_atomic(&path, "second").expect("second write");

        assert_eq!(fs::read_to_string(path).expect("read"), "second");
    }
}
