//! Windows test startup must pass WebView2 options through the native API.
//! WebView2 150+ ignores environment overrides in elevated host processes.

use std::path::{Path, PathBuf};
use tauri::{utils::config::WindowConfig, App, Config, WebviewWindowBuilder};

const DEFAULT_BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

pub(crate) struct TestWindow {
    config: WindowConfig,
    profile: PathBuf,
}

pub(crate) fn prepare(config: &mut Config, enabled: bool, workspace: &Path) -> Option<TestWindow> {
    if !enabled {
        return None;
    }
    let main = config
        .app
        .windows
        .iter_mut()
        .find(|window| window.label == "main")?;
    let test_window = TestWindow {
        config: main.clone(),
        profile: workspace.join("webview2-profile"),
    };
    // Build this window in setup so absolute profile paths use the native API.
    main.create = false;
    Some(test_window)
}

impl TestWindow {
    pub(crate) fn build(self, app: &App) -> tauri::Result<()> {
        let browser_args = format!(
            "{} --remote-debugging-port=0",
            self.config
                .additional_browser_args
                .as_deref()
                .unwrap_or(DEFAULT_BROWSER_ARGS),
        );
        WebviewWindowBuilder::from_config(app, &self.config)?
            .data_directory(self.profile)
            .additional_browser_args(&browser_args)
            .build()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        let mut config = Config::default();
        config.app.windows.push(WindowConfig::default());
        config
    }

    #[test]
    fn ordinary_startup_keeps_automatic_window_and_browser_settings() {
        let mut config = config();
        config.app.windows[0].additional_browser_args = Some("--custom-setting".into());
        assert!(prepare(&mut config, false, Path::new("unused")).is_none());
        assert!(config.app.windows[0].create);
        assert_eq!(
            config.app.windows[0].additional_browser_args.as_deref(),
            Some("--custom-setting")
        );
        assert!(config.app.windows[0].data_directory.is_none());
    }

    #[test]
    fn test_startup_uses_isolated_profile_and_preserves_window_configuration() {
        let mut config = config();
        config.app.windows[0].visible = false;
        config.app.windows[0].drag_drop_enabled = false;
        let workspace = std::env::temp_dir().join("neopad-webdriver-test");
        let window = prepare(&mut config, true, &workspace).expect("test window");
        assert!(!config.app.windows[0].create);
        assert!(!window.config.visible);
        assert!(!window.config.drag_drop_enabled);
        assert_eq!(window.profile, workspace.join("webview2-profile"));
        assert!(window.profile.is_absolute());
    }

    #[test]
    fn test_startup_only_replaces_the_main_window() {
        let mut config = config();
        config.app.windows.push(WindowConfig {
            label: "other".into(),
            ..Default::default()
        });
        prepare(&mut config, true, Path::new("workspace")).expect("test window");
        assert!(config.app.windows[1].create);
    }
}
