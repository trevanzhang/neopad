use crate::{atomic_write::write_atomic_bytes, lock_workspace_for_write, Workspace};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentToolsStatus {
    pub cli_path: PathBuf,
    pub version: String,
    pub installed: bool,
    pub matches_bundle: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWorkspaceDiagnostics {
    pub workspace: PathBuf,
    pub workspace_exists: bool,
    pub config_valid: bool,
    pub cli_path: PathBuf,
    pub cli_installed: bool,
}

/// Inspect an existing workspace without initializing it or writing files.
pub fn inspect_agent_workspace(workspace: &Workspace) -> AgentWorkspaceDiagnostics {
    AgentWorkspaceDiagnostics {
        workspace: workspace.root.clone(),
        workspace_exists: workspace.root.is_dir(),
        config_valid: workspace.config_path.is_file() && crate::load_config(workspace).is_ok(),
        cli_path: agent_cli_path(workspace),
        cli_installed: check_deployment_path(workspace).is_ok()
            && agent_cli_path(workspace).is_file(),
    }
}

pub fn agent_cli_path(workspace: &Workspace) -> PathBuf {
    workspace
        .root
        .join(".agents")
        .join("bin")
        .join(if cfg!(windows) {
            "neopad-cli.exe"
        } else {
            "neopad-cli"
        })
}

// Reject links/junctions in the managed deployment path. Never follow a link
// supplied in the writable data directory when replacing executable files.
fn check_deployment_path(workspace: &Workspace) -> Result<()> {
    for path in [
        workspace.root.join(".agents"),
        workspace.root.join(".agents/bin"),
        agent_cli_path(workspace),
    ] {
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                #[cfg(windows)]
                let linked = {
                    use std::os::windows::fs::MetadataExt;
                    metadata.file_attributes() & 0x400 != 0
                };
                #[cfg(not(windows))]
                let linked = metadata.file_type().is_symlink();
                if linked {
                    bail!(
                        "agent CLI deployment path contains a link: {}",
                        path.display()
                    );
                }
                if path == agent_cli_path(workspace) && !metadata.is_file() {
                    bail!("agent CLI target is not a regular file");
                }
                if path != agent_cli_path(workspace) && !metadata.is_dir() {
                    bail!("agent CLI parent is not a directory");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

pub fn agent_tools_status(workspace: &Workspace, source: &Path) -> Result<AgentToolsStatus> {
    check_deployment_path(workspace)?;
    let target = agent_cli_path(workspace);
    let installed = target.is_file();
    #[cfg(unix)]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        installed && fs::metadata(&target)?.permissions().mode() & 0o100 != 0
    };
    #[cfg(not(unix))]
    let executable = installed;
    let matches_bundle = installed
        && executable
        && source.is_file()
        && Sha256::digest(fs::read(&target)?) == Sha256::digest(fs::read(source)?);
    Ok(AgentToolsStatus {
        cli_path: target,
        version: env!("CARGO_PKG_VERSION").to_owned(),
        installed,
        matches_bundle,
    })
}

pub fn install_agent_cli(workspace: &Workspace, source: &Path) -> Result<AgentToolsStatus> {
    let _lock = lock_workspace_for_write(workspace)?;
    check_deployment_path(workspace)?;
    let source = source
        .canonicalize()
        .context("bundled NeoPad CLI is missing; build or reinstall NeoPad")?;
    let status = agent_tools_status(workspace, &source)?;
    if !status.matches_bundle {
        let contents = fs::read(&source).context("could not read bundled NeoPad CLI")?;
        write_atomic_bytes(&status.cli_path, &contents)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&status.cli_path, fs::Permissions::from_mode(0o700))?;
    }
    agent_tools_status(workspace, &source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_workspace;

    #[test]
    fn deployment_repairs_binary_and_preserves_user_guides() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        let source = temp.path().join("cli");
        fs::write(&source, b"version one").unwrap();
        assert!(
            install_agent_cli(&workspace, &source)
                .unwrap()
                .matches_bundle
        );
        let guide = workspace.root.join(".agents/AGENTS.md");
        fs::write(&guide, "user instructions").unwrap();
        fs::write(agent_cli_path(&workspace), b"damaged").unwrap();
        assert!(
            !agent_tools_status(&workspace, &source)
                .unwrap()
                .matches_bundle
        );
        assert!(
            install_agent_cli(&workspace, &source)
                .unwrap()
                .matches_bundle
        );
        fs::write(&source, b"version two").unwrap();
        install_agent_cli(&workspace, &source).unwrap();
        assert_eq!(
            fs::read(agent_cli_path(&workspace)).unwrap(),
            b"version two"
        );
        assert_eq!(fs::read_to_string(guide).unwrap(), "user instructions");
    }

    #[test]
    fn deployment_rejects_a_non_directory_without_touching_it() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        fs::write(workspace.root.join(".agents"), "user file").unwrap();
        let source = temp.path().join("cli");
        fs::write(&source, b"binary").unwrap();
        assert!(install_agent_cli(&workspace, &source).is_err());
        assert_eq!(
            fs::read_to_string(workspace.root.join(".agents")).unwrap(),
            "user file"
        );
    }

    #[cfg(unix)]
    #[test]
    fn deployment_repairs_missing_execute_permission() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let workspace = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        let source = temp.path().join("cli");
        fs::write(&source, b"binary").unwrap();
        install_agent_cli(&workspace, &source).unwrap();
        fs::set_permissions(
            agent_cli_path(&workspace),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        assert!(
            !agent_tools_status(&workspace, &source)
                .unwrap()
                .matches_bundle
        );
        assert!(
            install_agent_cli(&workspace, &source)
                .unwrap()
                .matches_bundle
        );
    }

    #[cfg(unix)]
    #[test]
    fn deployment_rejects_symlink_without_writing_outside_workspace() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = init_workspace(Some(temp.path().join("workspace"))).unwrap();
        let outside = temp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, workspace.root.join(".agents")).unwrap();
        let source = temp.path().join("cli");
        fs::write(&source, b"binary").unwrap();
        assert!(install_agent_cli(&workspace, &source).is_err());
        assert!(!outside.join("bin").exists());
    }
}
