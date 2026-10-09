use crate::commands::{display_error, AppState};
use anyhow::{bail, Result};
use neopad_core::{agent_tools_status, install_agent_cli, AgentToolsStatus};
use std::path::PathBuf;
use tauri::State;

fn bundled_cli() -> Result<PathBuf> {
    let name = if cfg!(windows) {
        "neopad-cli.exe"
    } else {
        "neopad-cli"
    };
    let executable = std::env::current_exe()?;
    let parent = executable
        .parent()
        .ok_or_else(|| anyhow::anyhow!("app executable has no parent"))?;
    let candidates = vec![parent.join(name)];
    #[cfg(debug_assertions)]
    let candidates = {
        let mut candidates = candidates;
        if let Some(path) = std::env::var_os("NEOPAD_CLI_BINARY") {
            candidates.insert(0, path.into());
        }
        candidates.push(parent.join("..").join(name));
        candidates.push(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/release")
                .join(name),
        );
        candidates
    };
    if let Some(path) = candidates.into_iter().find(|path| path.is_file()) {
        return Ok(path);
    }
    bail!("bundled NeoPad CLI is missing; build neopad-cli or reinstall NeoPad")
}

pub fn deploy(state: &AppState) -> Result<AgentToolsStatus> {
    install_agent_cli(&state.workspace, &bundled_cli()?)
}

#[tauri::command]
pub fn get_agent_tools_status_command(
    state: State<'_, AppState>,
) -> Result<AgentToolsStatus, String> {
    agent_tools_status(&state.workspace, &bundled_cli().map_err(display_error)?)
        .map_err(display_error)
}

#[tauri::command]
pub fn install_agent_cli_command(state: State<'_, AppState>) -> Result<AgentToolsStatus, String> {
    deploy(&state).map_err(display_error)
}
