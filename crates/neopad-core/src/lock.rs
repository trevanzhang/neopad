use crate::Workspace;
use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::{File, OpenOptions};

pub struct WorkspaceWriteLock(File);

impl Drop for WorkspaceWriteLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

pub fn lock_workspace_for_write(workspace: &Workspace) -> Result<WorkspaceWriteLock> {
    let path = workspace.root.join(".write.lock");
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .with_context(|| format!("failed to open workspace lock {}", path.display()))?;
    file.lock_exclusive()
        .with_context(|| format!("failed to lock workspace {}", workspace.root.display()))?;
    Ok(WorkspaceWriteLock(file))
}
