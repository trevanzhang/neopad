use crate::{atomic_write::write_atomic, Workspace};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    pub language: String,
    pub vim_mode: bool,
    pub vim_insert_exit_key: String,
    pub tab_bar_orientation: String,
    pub word_wrap: bool,
    pub editor_font_family: String,
    pub editor_background_color: String,
    pub window_opacity: f64,
    pub run_at_startup: bool,
    pub close_to_minimize: bool,
    pub snap_to_edges: bool,
    pub transparency_enabled: bool,
    pub title_double_click_action: String,
    pub shortcut_base_key: String,
    pub shortcut_modifiers: Vec<String>,
    pub clipboard_shortcut_base_key: String,
    pub clipboard_shortcut_modifiers: Vec<String>,
    pub insert_separator_template: String,
    pub insert_date_time_template: String,
    pub insert_date_time_separator_template: String,
    pub custom_insert_texts: Vec<String>,
    #[serde(default = "default_editor_mode_shortcut")]
    pub editor_mode_shortcut: String,
}

fn default_editor_mode_shortcut() -> String {
    "F7".to_owned()
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            language: "en".to_owned(),
            vim_mode: false,
            vim_insert_exit_key: "jj".to_owned(),
            tab_bar_orientation: "horizontal".to_owned(),
            word_wrap: true,
            editor_font_family: r#""Segoe UI", Arial, sans-serif"#.to_owned(),
            editor_background_color: "#ffffff".to_owned(),
            window_opacity: 1.0,
            run_at_startup: false,
            close_to_minimize: true,
            snap_to_edges: false,
            transparency_enabled: true,
            title_double_click_action: "rename".to_owned(),
            shortcut_base_key: "Z".to_owned(),
            shortcut_modifiers: vec!["Alt".to_owned()],
            clipboard_shortcut_base_key: "V".to_owned(),
            clipboard_shortcut_modifiers: vec!["Ctrl".to_owned(), "Shift".to_owned()],
            insert_separator_template: "crlf() + chars('-', 80) + crlf()".to_owned(),
            insert_date_time_template: "date() + ' ' + time()".to_owned(),
            insert_date_time_separator_template:
                "crlf() + chars('-', 29) + ' ' + date() + ' ' + time() + ' ' + chars('-', 29) + crlf()".to_owned(),
            custom_insert_texts: Vec::new(),
            editor_mode_shortcut: default_editor_mode_shortcut(),
        }
    }
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
            version: 2,
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
            ui: UiConfig::default(),
        }
    }

    pub fn default_for_path(workspace_dir: &Path) -> Self {
        Self::default_for_workspace(&workspace_dir.to_string_lossy())
    }
}

pub fn load_config(workspace: &Workspace) -> Result<AppConfig> {
    let contents = fs::read_to_string(&workspace.config_path).with_context(|| {
        format!(
            "failed to read config file {}",
            workspace.config_path.display()
        )
    })?;
    serde_json::from_str(&contents).with_context(|| {
        format!(
            "failed to parse config file {}",
            workspace.config_path.display()
        )
    })
}

pub fn save_config(workspace: &Workspace, config: &AppConfig) -> Result<()> {
    let contents = serde_json::to_string_pretty(config).context("failed to serialize config")?;
    write_atomic(&workspace.config_path, &contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_workspace;

    #[test]
    fn old_config_without_ui_uses_safe_defaults() {
        let json = r#"{
          "version": 1,
          "workspaceDir": "~/.neopad",
          "theme": "system",
          "defaultHotkey": "Alt+Z",
          "clipboardHotkey": "Ctrl+Shift+V",
          "hideOnEsc": true,
          "alwaysOnTop": false,
          "startAtLogin": false,
          "showInTaskbar": false,
          "autoSaveDebounceMs": 500,
          "defaultOpenMode": "edit",
          "previewMode": "split",
          "clipboardTarget": "clipboard",
          "maxSearchResults": 100,
          "mcp": { "enabled": true, "defaultReadOnly": true }
        }"#;
        let config: AppConfig = serde_json::from_str(json).expect("legacy config");
        assert_eq!(config.version, 1);
        assert!(!config.ui.run_at_startup);
        assert_eq!(config.ui.shortcut_base_key, "Z");
    }

    #[test]
    fn old_ui_config_without_editor_mode_shortcut_uses_f7() {
        let mut value = serde_json::to_value(UiConfig::default()).expect("serialize UI config");
        value
            .as_object_mut()
            .expect("UI config object")
            .remove("editorModeShortcut");

        let config: UiConfig = serde_json::from_value(value).expect("legacy UI config");
        assert_eq!(config.editor_mode_shortcut, "F7");
    }

    #[test]
    fn vim_mode_is_required_in_ui_config() {
        let mut value = serde_json::to_value(UiConfig::default()).expect("serialize UI config");
        value
            .as_object_mut()
            .expect("UI config object")
            .remove("vimMode");

        assert!(serde_json::from_value::<UiConfig>(value).is_err());
    }

    #[test]
    fn save_config_round_trips_ui_settings() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        let mut config = load_config(&workspace).expect("load config");
        config.ui.language = "zh".to_owned();
        config.ui.window_opacity = 0.75;
        save_config(&workspace, &config).expect("save config");
        let loaded = load_config(&workspace).expect("reload config");
        assert_eq!(loaded.ui.language, "zh");
        assert_eq!(loaded.ui.window_opacity, 0.75);
    }
}
