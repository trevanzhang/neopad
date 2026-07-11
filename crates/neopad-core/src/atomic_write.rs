use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("target path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create parent directory {}", parent.display()))?;

    let tmp_path = temp_path(path)?;
    let mut tmp_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)
        .with_context(|| format!("failed to create temp file {}", tmp_path.display()))?;
    tmp_file
        .write_all(contents.as_bytes())
        .with_context(|| format!("failed to write temp file {}", tmp_path.display()))?;
    tmp_file
        .sync_all()
        .with_context(|| format!("failed to flush temp file {}", tmp_path.display()))?;
    drop(tmp_file);

    match replace_file(&tmp_path, path) {
        Ok(()) => {
            sync_parent_directory(parent)?;
            Ok(())
        }
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

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> Result<()> {
    fs::File::open(parent)
        .with_context(|| format!("failed to open parent directory {}", parent.display()))?
        .sync_all()
        .with_context(|| format!("failed to flush parent directory {}", parent.display()))
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> Result<()> {
    Ok(())
}

#[cfg(not(windows))]
fn replace_file(tmp_path: &Path, target_path: &Path) -> std::io::Result<()> {
    fs::rename(tmp_path, target_path)
}

#[cfg(windows)]
fn replace_file(tmp_path: &Path, target_path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, ReplaceFileW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        REPLACEFILE_WRITE_THROUGH,
    };

    let wide = |path: &Path| {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>()
    };
    let source = wide(tmp_path);
    let target = wide(target_path);

    let result = unsafe {
        if target_path.exists() {
            ReplaceFileW(
                target.as_ptr(),
                source.as_ptr(),
                std::ptr::null(),
                REPLACEFILE_WRITE_THROUGH,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } else {
            MoveFileExW(
                source.as_ptr(),
                target.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        }
    };

    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
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
