use crate::commands::{display_error, AppState};
use anyhow::{bail, Context, Result};
use neopad_core::{load_config, lock_workspace_for_write, save_config, McpConfig};
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;
use std::{
    net::IpAddr,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::State;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug)]
pub struct OwnedMcpProcess {
    child: Option<Child>,
}

impl OwnedMcpProcess {
    fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn is_running(&mut self) -> Result<bool> {
        let Some(child) = self.child.as_mut() else {
            return Ok(false);
        };
        if child.try_wait()?.is_none() {
            return Ok(true);
        }
        self.child = None;
        Ok(false)
    }
}

impl Drop for OwnedMcpProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpStatus {
    pub enabled: bool,
    pub running: bool,
    pub status: String,
    pub url: String,
    pub host: String,
    pub port: u16,
    pub token: String,
    pub last_error: Option<String>,
}

#[tauri::command]
pub fn get_mcp_status_command(state: State<'_, AppState>) -> Result<McpStatus, String> {
    ensure_configured(&state).map_err(display_error)?;
    if load_config(&state.workspace)
        .map(|config| config.mcp.enabled)
        .unwrap_or(false)
    {
        let _ = start_if_enabled(&state);
    }
    status(&state).map_err(display_error)
}

#[tauri::command]
pub fn set_mcp_enabled_command(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<McpStatus, String> {
    {
        let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
        let mut config = load_config(&state.workspace).map_err(display_error)?;
        normalize_mcp_config(&mut config.mcp);
        config.mcp.enabled = enabled;
        save_config(&state.workspace, &config).map_err(display_error)?;
    }

    if enabled {
        let _ = start_if_enabled(&state);
    } else {
        stop_owned_process(&state).map_err(display_error)?;
        clear_error(&state);
    }

    status(&state).map_err(display_error)
}

#[tauri::command]
pub fn regenerate_mcp_token_command(state: State<'_, AppState>) -> Result<McpStatus, String> {
    let was_enabled = {
        let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
        let mut config = load_config(&state.workspace).map_err(display_error)?;
        normalize_mcp_config(&mut config.mcp);
        config.mcp.token = generate_token();
        let enabled = config.mcp.enabled;
        save_config(&state.workspace, &config).map_err(display_error)?;
        enabled
    };

    stop_owned_process(&state).map_err(display_error)?;
    if was_enabled {
        let _ = start_if_enabled(&state);
    }

    status(&state).map_err(display_error)
}

pub fn start_if_enabled(state: &AppState) -> Result<()> {
    ensure_configured(state)?;
    let config = load_config(&state.workspace)?;
    if !config.mcp.enabled {
        return Ok(());
    }
    start_owned_process(state, &config.mcp)
}

pub fn stop_owned_process(state: &AppState) -> Result<()> {
    let mut process = state
        .mcp_process
        .lock()
        .map_err(|error| anyhow::anyhow!("failed to lock MCP process state: {error}"))?;
    if let Some(mut process) = process.take() {
        process.stop();
    }
    Ok(())
}

fn status(state: &AppState) -> Result<McpStatus> {
    let config = load_config(&state.workspace)?;
    let running = is_running(state)?;
    let last_error = state
        .mcp_error
        .lock()
        .map_err(|error| anyhow::anyhow!("failed to lock MCP error state: {error}"))?
        .clone();
    let service_status = if running {
        "Running"
    } else if !config.mcp.enabled {
        "Off"
    } else if last_error.is_some() {
        "Failed"
    } else {
        "Stopped"
    };

    Ok(McpStatus {
        enabled: config.mcp.enabled,
        running,
        status: service_status.to_owned(),
        url: mcp_url(&config.mcp),
        host: config.mcp.host,
        port: config.mcp.port,
        token: config.mcp.token,
        last_error,
    })
}

fn ensure_configured(state: &AppState) -> Result<()> {
    let _lock = lock_workspace_for_write(&state.workspace)?;
    let mut config = load_config(&state.workspace)?;
    let before = config.mcp.clone();
    normalize_mcp_config(&mut config.mcp);
    if config.mcp != before {
        save_config(&state.workspace, &config)?;
    }
    Ok(())
}

fn normalize_mcp_config(config: &mut McpConfig) {
    let host_is_loopback = config
        .host
        .parse::<IpAddr>()
        .is_ok_and(|address| address.is_loopback());
    if !host_is_loopback {
        config.host = "127.0.0.1".to_owned();
    }
    if config.port == 0 {
        config.port = 8765;
    }
    if config.token.trim().is_empty() {
        config.token = generate_token();
    }
}

fn start_owned_process(state: &AppState, config: &McpConfig) -> Result<()> {
    if is_running(state)? {
        return Ok(());
    }

    clear_error(state);
    let binary = resolve_mcp_binary();
    let mut command = Command::new(&binary);
    command
        .arg("--workspace")
        .arg(&state.workspace.root)
        .arg("serve")
        .arg("--host")
        .arg(&config.host)
        .arg("--port")
        .arg(config.port.to_string())
        .env("NEOPAD_MCP_TOKEN", &config.token)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to start {}", binary.display()))?;
    if let Some(stderr) = child.stderr.take() {
        capture_stderr(stderr, Arc::clone(&state.mcp_error));
    }

    thread::sleep(Duration::from_millis(150));
    if let Some(status) = child.try_wait()? {
        let error = current_error(state)
            .unwrap_or_else(|| format!("neopad-mcp exited during startup with status {}", status));
        set_error(state, error.clone());
        bail!(error);
    }

    let mut process = state
        .mcp_process
        .lock()
        .map_err(|error| anyhow::anyhow!("failed to lock MCP process state: {error}"))?;
    *process = Some(OwnedMcpProcess::new(child));
    Ok(())
}

fn is_running(state: &AppState) -> Result<bool> {
    let mut process = state
        .mcp_process
        .lock()
        .map_err(|error| anyhow::anyhow!("failed to lock MCP process state: {error}"))?;
    if let Some(child) = process.as_mut() {
        if child.is_running()? {
            return Ok(true);
        }
        *process = None;
    }
    Ok(false)
}

fn resolve_mcp_binary() -> PathBuf {
    if let Some(path) = std::env::var_os("NEOPAD_MCP_BINARY") {
        return PathBuf::from(path);
    }

    let exe_name = if cfg!(windows) {
        "neopad-mcp.exe"
    } else {
        "neopad-mcp"
    };

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let sibling = dir.join(exe_name);
            if sibling.exists() {
                return sibling;
            }
            let debug_sibling = dir.join("..").join(exe_name);
            if debug_sibling.exists() {
                return debug_sibling;
            }
        }
    }

    PathBuf::from(exe_name)
}

fn capture_stderr(stderr: std::process::ChildStderr, error: Arc<Mutex<Option<String>>>) {
    thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(|line| line.ok()) {
            if let Ok(mut slot) = error.lock() {
                *slot = Some(line);
            }
        }
    });
}

fn mcp_url(config: &McpConfig) -> String {
    format!("http://{}:{}/mcp", config.host, config.port)
}

fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}

fn clear_error(state: &AppState) {
    if let Ok(mut error) = state.mcp_error.lock() {
        *error = None;
    }
}

fn set_error(state: &AppState, message: String) {
    if let Ok(mut error) = state.mcp_error.lock() {
        *error = Some(message);
    }
}

fn current_error(state: &AppState) -> Option<String> {
    state.mcp_error.lock().ok().and_then(|error| error.clone())
}

#[cfg(test)]
mod tests {
    use super::normalize_mcp_config;
    use neopad_core::McpConfig;

    #[test]
    fn managed_mcp_config_forces_loopback_and_generates_a_token() {
        let mut config = McpConfig {
            enabled: true,
            host: "192.168.1.50".to_owned(),
            port: 0,
            token: String::new(),
        };

        normalize_mcp_config(&mut config);

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8765);
        assert_eq!(config.token.len(), 48);
    }
}
