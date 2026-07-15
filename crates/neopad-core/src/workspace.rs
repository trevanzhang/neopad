use crate::{AppConfig, PromptState, TabsState};
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub root: PathBuf,
    pub notes_dir: PathBuf,
    pub meta_dir: PathBuf,
    pub archive_dir: PathBuf,
    pub trash_dir: PathBuf,
    pub prompts_dir: PathBuf,
    pub config_path: PathBuf,
    pub tabs_path: PathBuf,
    pub prompts_meta_path: PathBuf,
    pub reminders_path: PathBuf,
}

impl Workspace {
    pub fn new(root: PathBuf) -> Self {
        Self {
            notes_dir: root.join("notes"),
            meta_dir: root.join("meta"),
            archive_dir: root.join("archive"),
            trash_dir: root.join("trash"),
            prompts_dir: root.join("prompts"),
            config_path: root.join("config.json"),
            tabs_path: root.join("meta").join("tabs.json"),
            prompts_meta_path: root.join("meta").join("prompts.json"),
            reminders_path: root.join("meta").join("reminders.json"),
            root,
        }
    }

    pub fn inbox_path(&self) -> PathBuf {
        self.notes_dir.join("inbox.md")
    }

    pub fn clipboard_path(&self) -> PathBuf {
        self.notes_dir.join("clipboard.md")
    }
}

pub fn default_workspace_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine user home directory")?;
    Ok(home.join(".neopad"))
}

pub fn init_workspace(path: Option<PathBuf>) -> Result<Workspace> {
    let root = path.unwrap_or(default_workspace_dir()?);
    let workspace = Workspace::new(root);
    fs::create_dir_all(&workspace.root).with_context(|| {
        format!(
            "failed to create workspace directory at {}",
            workspace.root.display()
        )
    })?;
    let _lock = crate::lock_workspace_for_write(&workspace)?;
    ensure_workspace_layout(&workspace)?;
    Ok(workspace)
}

pub fn ensure_workspace_layout(workspace: &Workspace) -> Result<()> {
    fs::create_dir_all(&workspace.notes_dir).with_context(|| {
        format!(
            "failed to create notes directory at {}",
            workspace.notes_dir.display()
        )
    })?;
    fs::create_dir_all(&workspace.meta_dir).with_context(|| {
        format!(
            "failed to create meta directory at {}",
            workspace.meta_dir.display()
        )
    })?;
    fs::create_dir_all(&workspace.archive_dir).with_context(|| {
        format!(
            "failed to create archive directory at {}",
            workspace.archive_dir.display()
        )
    })?;
    fs::create_dir_all(&workspace.trash_dir).with_context(|| {
        format!(
            "failed to create trash directory at {}",
            workspace.trash_dir.display()
        )
    })?;
    fs::create_dir_all(&workspace.prompts_dir).with_context(|| {
        format!(
            "failed to create prompts directory at {}",
            workspace.prompts_dir.display()
        )
    })?;
    write_file_if_missing(&workspace.config_path, &default_config_json(workspace)?)?;
    write_file_if_missing(&workspace.tabs_path, &default_tabs_json()?)?;
    write_file_if_missing(&workspace.prompts_meta_path, &default_prompts_json()?)?;
    write_file_if_missing(&workspace.reminders_path, default_reminders_json())?;
    write_file_if_missing(&workspace.inbox_path(), "# Inbox\n\n")?;
    write_file_if_missing(&workspace.clipboard_path(), "# Clipboard\n\n")?;

    ensure_writable(&workspace.root)?;
    Ok(())
}

fn write_file_if_missing(path: &Path, contents: &str) -> Result<()> {
    let mut file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to create default file at {}", path.display()))
        }
    };
    file.write_all(contents.as_bytes())
        .with_context(|| format!("failed to write default file at {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("failed to flush default file at {}", path.display()))
}

fn default_config_json(workspace: &Workspace) -> Result<String> {
    let default_dir = default_workspace_dir()?;
    let config = if workspace.root == default_dir {
        AppConfig::default_for_workspace("~/.neopad")
    } else {
        AppConfig::default_for_path(&workspace.root)
    };
    serde_json::to_string_pretty(&config).context("failed to serialize default config")
}

fn default_tabs_json() -> Result<String> {
    let tabs = TabsState::default_with_timestamp(now_ms()?);
    serde_json::to_string_pretty(&tabs).context("failed to serialize default tabs")
}

fn default_prompts_json() -> Result<String> {
    serde_json::to_string_pretty(&PromptState::default())
        .context("failed to serialize default prompt metadata")
}

fn default_reminders_json() -> &'static str {
    "{\n  \"version\": 1,\n  \"delivered\": []\n}"
}

fn now_ms() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before unix epoch")?;
    Ok(duration.as_millis() as i64)
}

fn ensure_writable(path: &Path) -> Result<()> {
    let probe_path = path.join(".neopad-write-test");
    fs::write(&probe_path, b"ok")
        .with_context(|| format!("workspace is not writable at {}", path.display()))?;
    fs::remove_file(&probe_path)
        .with_context(|| format!("failed to remove write probe at {}", probe_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PreviewMode, Theme};

    #[test]
    fn init_workspace_creates_required_layout_and_defaults() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let root = temp_dir.path().join("neopad-data");

        let workspace = init_workspace(Some(root.clone())).expect("workspace init");

        assert_eq!(workspace.root, root);
        assert!(workspace.notes_dir.is_dir());
        assert!(workspace.meta_dir.is_dir());
        assert!(workspace.archive_dir.is_dir());
        assert!(workspace.trash_dir.is_dir());
        assert!(workspace.prompts_dir.is_dir());
        assert_eq!(
            fs::read_to_string(workspace.inbox_path()).expect("inbox"),
            "# Inbox\n\n"
        );
        assert_eq!(
            fs::read_to_string(workspace.clipboard_path()).expect("clipboard"),
            "# Clipboard\n\n"
        );

        let config: AppConfig =
            serde_json::from_str(&fs::read_to_string(&workspace.config_path).expect("config"))
                .expect("config json");
        assert_eq!(config.version, 2);
        assert_eq!(config.workspace_dir, root.to_string_lossy());
        assert_eq!(config.theme, Theme::System);
        assert_eq!(config.default_hotkey, "Alt+Z");
        assert_eq!(config.clipboard_hotkey, "Ctrl+Shift+V");
        assert_eq!(config.ui.language, "zh");
        assert_eq!(config.preview_mode, PreviewMode::Edit);
        assert!(config.hide_on_esc);
        assert!(!config.mcp.enabled);
        assert_eq!(config.mcp.host, "127.0.0.1");
        assert_eq!(config.mcp.port, 8765);
        assert_eq!(config.mcp.token, "");

        let tabs: TabsState =
            serde_json::from_str(&fs::read_to_string(&workspace.tabs_path).expect("tabs"))
                .expect("tabs json");
        assert_eq!(tabs.version, 1);
        assert_eq!(tabs.active_tab_id, "inbox");
        assert_eq!(tabs.tabs.len(), 2);
        assert_eq!(tabs.tabs[0].id, "inbox");
        assert_eq!(tabs.tabs[0].file_name, "inbox.md");
        assert!(tabs.tabs[0].pinned);
        assert_eq!(tabs.tabs[1].id, "clipboard");
        assert_eq!(tabs.tabs[1].file_name, "clipboard.md");
        assert!(tabs.tabs[1].pinned);
        assert!(workspace.reminders_path.is_file());
        assert!(workspace.prompts_meta_path.is_file());
    }

    #[test]
    fn ensure_workspace_layout_does_not_overwrite_existing_files() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = Workspace::new(temp_dir.path().join("neopad-data"));
        fs::create_dir_all(&workspace.notes_dir).expect("notes dir");
        fs::write(workspace.inbox_path(), "# Existing\n\nKeep this.\n").expect("write inbox");

        ensure_workspace_layout(&workspace).expect("layout");

        assert_eq!(
            fs::read_to_string(workspace.inbox_path()).expect("inbox"),
            "# Existing\n\nKeep this.\n"
        );
        assert!(workspace.config_path.is_file());
        assert!(workspace.tabs_path.is_file());
        assert!(workspace.clipboard_path().is_file());
    }

    #[test]
    fn default_workspace_dir_uses_home_dot_neopad() {
        let default_dir = default_workspace_dir().expect("default workspace dir");
        assert_eq!(
            default_dir.file_name().and_then(|name| name.to_str()),
            Some(".neopad")
        );
    }
}
