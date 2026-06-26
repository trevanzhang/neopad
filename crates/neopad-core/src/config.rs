use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    pub workspace_dir: String,
    pub theme: Theme,
    pub default_hotkey: String,
    pub clipboard_hotkey: String,
    pub hide_on_esc: bool,
    pub always_on_top: bool,
    pub start_at_login: bool,
    pub show_in_taskbar: bool,
    pub auto_save_debounce_ms: u64,
    pub default_open_mode: String,
    pub preview_mode: PreviewMode,
    pub clipboard_target: String,
    pub max_search_results: usize,
    pub mcp: McpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    pub enabled: bool,
    pub default_read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PreviewMode {
    Edit,
    Preview,
    Split,
}

impl AppConfig {
    pub fn default_for_workspace(workspace_dir: &str) -> Self {
        Self {
            version: 1,
            workspace_dir: workspace_dir.to_owned(),
            theme: Theme::System,
            default_hotkey: "Alt+Z".to_owned(),
            clipboard_hotkey: "Ctrl+Shift+V".to_owned(),
            hide_on_esc: true,
            always_on_top: false,
            start_at_login: false,
            show_in_taskbar: false,
            auto_save_debounce_ms: 500,
            default_open_mode: "edit".to_owned(),
            preview_mode: PreviewMode::Split,
            clipboard_target: "clipboard".to_owned(),
            max_search_results: 100,
            mcp: McpConfig {
                enabled: true,
                default_read_only: true,
            },
        }
    }

    pub fn default_for_path(workspace_dir: &PathBuf) -> Self {
        Self::default_for_workspace(&workspace_dir.to_string_lossy())
    }
}
