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
    pub guide_path: PathBuf,
    pub guide_available: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWorkspaceDiagnostics {
    pub workspace: PathBuf,
    pub workspace_exists: bool,
    pub config_valid: bool,
    pub cli_path: PathBuf,
    pub cli_installed: bool,
    pub guide_path: PathBuf,
    pub guide_available: bool,
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
        guide_path: agent_guide_path(workspace),
        guide_available: is_regular_guide(&agent_guide_path(workspace)),
    }
}

pub fn agent_guide_path(workspace: &Workspace) -> PathBuf {
    workspace.root.join("AGENTS.md")
}

// Called under the workspace lock. Preserve every existing entry, including
// empty files, directories and dangling links; never write through a user link.
pub(crate) fn ensure_agent_guide_file(workspace: &Workspace) -> Result<()> {
    let path = agent_guide_path(workspace);
    match fs::symlink_metadata(&path) {
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("could not inspect the Agent guide"),
    }
    crate::workspace::write_file_if_missing(&path, include_str!("../resources/workspace-AGENTS.md"))
}

fn is_regular_guide(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        #[cfg(windows)]
        let linked = {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 0x400 != 0
        };
        #[cfg(not(windows))]
        let linked = metadata.file_type().is_symlink();
        metadata.is_file() && !linked
    })
}

/// Create the default if absent and return a regular, local guide for editing.
pub fn prepare_agent_guide(workspace: &Workspace) -> Result<PathBuf> {
    let _lock = lock_workspace_for_write(workspace)?;
    ensure_agent_guide_file(workspace)?;
    let path = agent_guide_path(workspace);
    if !is_regular_guide(&path) {
        bail!("Agent guide must be a regular file, not a directory or link");
    }
    path.canonicalize()
        .context("could not locate the Agent guide")
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
        guide_path: agent_guide_path(workspace),
        guide_available: is_regular_guide(&agent_guide_path(workspace)),
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
    fn workspace_creates_guide_and_recreates_it_only_when_absent() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("中文 workspace");
        let workspace = init_workspace(Some(root.clone())).unwrap();
        let guide = prepare_agent_guide(&workspace).unwrap();
        let contents = fs::read_to_string(&guide).unwrap();
        assert!(contents.contains("NeoPad 工作指南"));
        assert!(contents.contains("--workspace WORKSPACE"));
        assert!(inspect_agent_workspace(&workspace).guide_available);
        assert_eq!(crate::list_notes(&workspace).unwrap().len(), 2);
        assert!(crate::search_notes(&workspace, "操作入口", 10)
            .unwrap()
            .is_empty());
        fs::remove_file(&guide).unwrap();
        assert!(!inspect_agent_workspace(&workspace).guide_available);
        assert_eq!(prepare_agent_guide(&workspace).unwrap(), guide);
        assert_eq!(fs::read_to_string(&guide).unwrap(), contents);
    }

    #[test]
    fn initialization_and_cli_repair_preserve_custom_and_empty_root_guides() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("cli");
        fs::write(&source, "bundled binary").unwrap();
        for (index, contents) in ["\u{feff}# 我的工作约定\r\n保留原文。\r\n", ""]
            .iter()
            .enumerate()
        {
            let root = temp.path().join(format!("workspace-{index}"));
            fs::create_dir_all(&root).unwrap();
            let guide = root.join("AGENTS.md");
            fs::write(&guide, contents.as_bytes()).unwrap();
            let workspace = init_workspace(Some(root.clone())).unwrap();
            install_agent_cli(&workspace, &source).unwrap();
            fs::write(&source, "new bundled binary").unwrap();
            install_agent_cli(&workspace, &source).unwrap();
            init_workspace(Some(root)).unwrap();
            prepare_agent_guide(&workspace).unwrap();
            assert_eq!(fs::read(guide).unwrap(), contents.as_bytes());
        }
    }

    #[test]
    fn existing_guide_directory_is_preserved_and_cannot_be_opened_as_a_file() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("AGENTS.md")).unwrap();
        fs::write(root.join("AGENTS.md/keep"), "user data").unwrap();
        let workspace = init_workspace(Some(root)).unwrap();
        assert!(!inspect_agent_workspace(&workspace).guide_available);
        assert!(prepare_agent_guide(&workspace).is_err());
        assert_eq!(
            fs::read_to_string(agent_guide_path(&workspace).join("keep")).unwrap(),
            "user data"
        );
    }

    #[cfg(unix)]
    #[test]
    fn guide_links_are_preserved_without_reading_or_writing_their_targets() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir(&root).unwrap();
        let outside = temp.path().join("outside.md");
        std::os::unix::fs::symlink(&outside, root.join("AGENTS.md")).unwrap();
        let workspace = init_workspace(Some(root.clone())).unwrap();
        assert!(!outside.exists());
        assert!(prepare_agent_guide(&workspace).is_err());
        fs::write(&outside, "outside user guide").unwrap();
        init_workspace(Some(root)).unwrap();
        assert!(!inspect_agent_workspace(&workspace).guide_available);
        assert!(prepare_agent_guide(&workspace).is_err());
        assert_eq!(fs::read_to_string(outside).unwrap(), "outside user guide");
    }

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
