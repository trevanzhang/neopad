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
    pub approved_external_markdown_paths: Vec<String>,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub vim_mode: bool,
    #[serde(default = "default_true")]
    pub vim_use_ctrl_shortcuts: bool,
    #[serde(default = "default_vim_insert_exit_key")]
    pub vim_insert_exit_key: String,
    #[serde(default = "default_tab_bar_orientation")]
    pub tab_bar_orientation: String,
    #[serde(default = "default_true")]
    pub word_wrap: bool,
    #[serde(default = "default_editor_font_family")]
    pub editor_font_family: String,
    #[serde(default = "default_editor_font_size")]
    pub editor_font_size: u8,
    #[serde(default = "default_editor_background_color")]
    pub editor_background_color: String,
    #[serde(default = "default_preview_theme")]
    pub preview_theme: String,
    #[serde(default = "default_preview_font_family")]
    pub preview_font_family: String,
    #[serde(default = "default_preview_font_size")]
    pub preview_font_size: u8,
    #[serde(default = "default_preview_line_height")]
    pub preview_line_height: String,
    #[serde(default = "default_preview_content_width")]
    pub preview_content_width: String,
    #[serde(default = "default_window_opacity")]
    pub window_opacity: f64,
    #[serde(default)]
    pub run_at_startup: bool,
    #[serde(default)]
    pub start_hidden: bool,
    #[serde(default = "default_true")]
    pub close_to_minimize: bool,
    #[serde(default)]
    pub snap_to_edges: bool,
    #[serde(default = "default_true")]
    pub transparency_enabled: bool,
    #[serde(default = "default_title_double_click_action")]
    pub title_double_click_action: String,
    #[serde(default = "default_shortcut_base_key")]
    pub shortcut_base_key: String,
    #[serde(default = "default_shortcut_modifiers")]
    pub shortcut_modifiers: Vec<String>,
    #[serde(default = "default_clipboard_shortcut_base_key")]
    pub clipboard_shortcut_base_key: String,
    #[serde(default = "default_clipboard_shortcut_modifiers")]
    pub clipboard_shortcut_modifiers: Vec<String>,
    #[serde(default = "default_insert_separator_template")]
    pub insert_separator_template: String,
    #[serde(default = "default_insert_date_time_template")]
    pub insert_date_time_template: String,
    #[serde(default = "default_insert_date_time_separator_template")]
    pub insert_date_time_separator_template: String,
    #[serde(default)]
    pub custom_insert_texts: Vec<String>,
    #[serde(default = "default_editor_mode_shortcut")]
    pub editor_mode_shortcut: String,
}

fn default_language() -> String {
    "en".to_owned()
}

fn default_vim_insert_exit_key() -> String {
    "jj".to_owned()
}

fn default_tab_bar_orientation() -> String {
    "horizontal".to_owned()
}

fn default_editor_font_family() -> String {
    r#""Segoe UI", Arial, sans-serif"#.to_owned()
}

fn default_editor_font_size() -> u8 {
    14
}

fn default_editor_background_color() -> String {
    "#ffffff".to_owned()
}

fn default_preview_theme() -> String {
    "light".to_owned()
}

fn default_preview_font_family() -> String {
    "editor".to_owned()
}

fn default_preview_font_size() -> u8 {
    14
}

fn default_preview_line_height() -> String {
    "standard".to_owned()
}

fn default_preview_content_width() -> String {
    "standard".to_owned()
}

fn default_window_opacity() -> f64 {
    1.0
}

fn default_title_double_click_action() -> String {
    "rename".to_owned()
}

fn default_shortcut_base_key() -> String {
    "Z".to_owned()
}

fn default_shortcut_modifiers() -> Vec<String> {
    vec!["Alt".to_owned()]
}

fn default_clipboard_shortcut_base_key() -> String {
    "V".to_owned()
}

fn default_clipboard_shortcut_modifiers() -> Vec<String> {
    vec!["Ctrl".to_owned(), "Shift".to_owned()]
}

fn default_insert_separator_template() -> String {
    "crlf() + chars('-', 80) + crlf()".to_owned()
}

fn default_insert_date_time_template() -> String {
    "date() + ' ' + time()".to_owned()
}

fn default_insert_date_time_separator_template() -> String {
    "crlf() + chars('-', 29) + ' ' + date() + ' ' + time() + ' ' + chars('-', 29) + crlf()"
        .to_owned()
}

fn default_editor_mode_shortcut() -> String {
    "F4".to_owned()
}

fn default_true() -> bool {
    true
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            vim_mode: false,
            vim_use_ctrl_shortcuts: true,
            vim_insert_exit_key: default_vim_insert_exit_key(),
            tab_bar_orientation: default_tab_bar_orientation(),
            word_wrap: true,
            editor_font_family: default_editor_font_family(),
            editor_font_size: default_editor_font_size(),
            editor_background_color: default_editor_background_color(),
            preview_theme: default_preview_theme(),
            preview_font_family: default_preview_font_family(),
            preview_font_size: default_preview_font_size(),
            preview_line_height: default_preview_line_height(),
            preview_content_width: default_preview_content_width(),
            window_opacity: default_window_opacity(),
            run_at_startup: false,
            start_hidden: false,
            close_to_minimize: true,
            snap_to_edges: false,
            transparency_enabled: true,
            title_double_click_action: default_title_double_click_action(),
            shortcut_base_key: default_shortcut_base_key(),
            shortcut_modifiers: default_shortcut_modifiers(),
            clipboard_shortcut_base_key: default_clipboard_shortcut_base_key(),
            clipboard_shortcut_modifiers: default_clipboard_shortcut_modifiers(),
            insert_separator_template: default_insert_separator_template(),
            insert_date_time_template: default_insert_date_time_template(),
            insert_date_time_separator_template: default_insert_date_time_separator_template(),
            custom_insert_texts: Vec::new(),
            editor_mode_shortcut: default_editor_mode_shortcut(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_mcp_host")]
    pub host: String,
    #[serde(default = "default_mcp_port")]
    pub port: u16,
    #[serde(default)]
    pub token: String,
}

fn default_mcp_host() -> String {
    "127.0.0.1".to_owned()
}

fn default_mcp_port() -> u16 {
    8765
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
            preview_mode: PreviewMode::Edit,
            clipboard_target: "clipboard".to_owned(),
            max_search_results: 100,
            mcp: McpConfig {
                enabled: false,
                host: default_mcp_host(),
                port: default_mcp_port(),
                token: String::new(),
            },
            approved_external_markdown_paths: Vec::new(),
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
        assert!(config.mcp.enabled);
        assert_eq!(config.mcp.host, "127.0.0.1");
        assert_eq!(config.mcp.port, 8765);
        assert_eq!(config.mcp.token, "");
        assert!(!config.ui.run_at_startup);
        assert!(!config.ui.start_hidden);
        assert_eq!(config.ui.shortcut_base_key, "Z");
    }

    #[test]
    fn old_ui_config_without_editor_mode_shortcut_uses_f4() {
        let mut value = serde_json::to_value(UiConfig::default()).expect("serialize UI config");
        value
            .as_object_mut()
            .expect("UI config object")
            .remove("editorModeShortcut");

        let config: UiConfig = serde_json::from_value(value).expect("legacy UI config");
        assert_eq!(config.editor_mode_shortcut, "F4");
    }

    #[test]
    fn old_ui_config_without_start_hidden_stays_visible() {
        let mut value = serde_json::to_value(UiConfig::default()).expect("serialize UI config");
        value
            .as_object_mut()
            .expect("UI config object")
            .remove("startHidden");

        let config: UiConfig = serde_json::from_value(value).expect("legacy UI config");
        assert!(!config.start_hidden);
    }

    #[test]
    fn old_ui_config_keeps_neopad_ctrl_shortcuts_in_vim_mode() {
        let mut value = serde_json::to_value(UiConfig::default()).expect("serialize UI config");
        value
            .as_object_mut()
            .expect("UI config object")
            .remove("vimUseCtrlShortcuts");

        let config: UiConfig = serde_json::from_value(value).expect("legacy UI config");
        assert!(config.vim_use_ctrl_shortcuts);
    }

    #[test]
    fn old_ui_config_without_vim_fields_uses_defaults() {
        let mut value = serde_json::to_value(UiConfig::default()).expect("serialize UI config");
        value
            .as_object_mut()
            .expect("UI config object")
            .remove("vimMode");
        value
            .as_object_mut()
            .expect("UI config object")
            .remove("vimInsertExitKey");

        let config: UiConfig = serde_json::from_value(value).expect("legacy UI config");
        assert!(!config.vim_mode);
        assert_eq!(config.vim_insert_exit_key, "jj");
    }

    #[test]
    fn sparse_ui_config_uses_defaults_for_missing_fields() {
        let json = r#"{
          "language": "zh",
          "vimMode": true,
          "runAtStartup": true
        }"#;
        let config: UiConfig = serde_json::from_str(json).expect("sparse UI config");
        assert_eq!(config.language, "zh");
        assert!(config.vim_mode);
        assert!(config.run_at_startup);
        assert_eq!(config.vim_insert_exit_key, "jj");
        assert_eq!(config.shortcut_base_key, "Z");
        assert_eq!(config.clipboard_shortcut_modifiers, ["Ctrl", "Shift"]);
        assert_eq!(config.editor_mode_shortcut, "F4");
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
