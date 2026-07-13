use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REPLACE_RETRY_DELAYS: [Duration; 8] = [
    Duration::from_millis(15),
    Duration::from_millis(30),
    Duration::from_millis(60),
    Duration::from_millis(120),
    Duration::from_millis(240),
    Duration::from_millis(480),
    Duration::from_millis(960),
    Duration::from_millis(1_920),
];

pub fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    write_atomic_bytes(path, contents.as_bytes())
}

pub fn write_atomic_bytes(path: &Path, contents: &[u8]) -> Result<()> {
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
        .write_all(contents)
        .with_context(|| format!("failed to write temp file {}", tmp_path.display()))?;
    tmp_file
        .sync_all()
        .with_context(|| format!("failed to flush temp file {}", tmp_path.display()))?;
    drop(tmp_file);

    match replace_file_with_retry(&tmp_path, path) {
        Ok(()) => {
            sync_parent_directory(parent)?;
            Ok(())
        }
        Err(error) => Err(error).with_context(|| {
            format!(
                "failed to replace {} after retrying; the new content was preserved in {}",
                path.display(),
                tmp_path.display(),
            )
        }),
    }
}

fn replace_file_with_retry(tmp_path: &Path, target_path: &Path) -> std::io::Result<()> {
    retry_file_operation(|| replace_file(tmp_path, target_path), thread::sleep)
}

fn retry_file_operation<F, S>(mut operation: F, mut sleep: S) -> std::io::Result<()>
where
    F: FnMut() -> std::io::Result<()>,
    S: FnMut(Duration),
{
    for delay in REPLACE_RETRY_DELAYS {
        match operation() {
            Ok(()) => return Ok(()),
            Err(error) if is_transient_replace_error(&error) => sleep(delay),
            Err(error) => return Err(error),
        }
    }
    operation()
}

#[cfg(windows)]
fn is_transient_replace_error(error: &std::io::Error) -> bool {
    matches!(error.raw_os_error(), Some(5 | 32 | 33 | 1175 | 1224))
}

#[cfg(not(windows))]
fn is_transient_replace_error(_error: &std::io::Error) -> bool {
    false
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

pub(crate) fn temporary_target_file_name(temp_file_name: &str) -> Option<&str> {
    let stem = temp_file_name.strip_prefix('.')?.strip_suffix(".tmp")?;
    let (stem, nanos) = stem.rsplit_once('.')?;
    let (target, process_id) = stem.rsplit_once('.')?;
    (process_id.parse::<u32>().is_ok() && nanos.parse::<u128>().is_ok()).then_some(target)
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

    #[cfg(windows)]
    #[test]
    fn retries_transient_replace_failures() {
        let mut attempts = 0;
        let mut delays = Vec::new();

        retry_file_operation(
            || {
                attempts += 1;
                if attempts < 3 {
                    Err(std::io::Error::from_raw_os_error(32))
                } else {
                    Ok(())
                }
            },
            |delay| delays.push(delay),
        )
        .expect("transient error is retried");

        assert_eq!(attempts, 3);
        assert_eq!(delays, REPLACE_RETRY_DELAYS[..2]);
    }

    #[cfg(windows)]
    #[test]
    fn does_not_retry_permanent_replace_failures() {
        let mut attempts = 0;

        let error = retry_file_operation(
            || {
                attempts += 1;
                Err(std::io::Error::from_raw_os_error(3))
            },
            |_| panic!("permanent error must not sleep"),
        )
        .expect_err("permanent error is returned");

        assert_eq!(attempts, 1);
        assert_eq!(error.raw_os_error(), Some(3));
    }
}
