use neopad_core::{
    append_to_clipboard_note, claim_due_reminders, complete_due_reminders, complete_reminder,
    create_note, delete_note_to_trash, export_note_file, list_notes, list_reminders, load_config,
    lock_workspace_for_write, read_note, rename_note, reopen_reminder, save_config, search_notes,
    write_note_atomic_checked, NoteContent, NoteTab, PreviewMode, Reminder, SearchResult, Theme,
    UiConfig, Workspace,
};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::Shortcut;

#[derive(Debug)]
pub struct AppState {
    pub workspace: Workspace,
    pub shortcut_warnings: Mutex<Vec<String>>,
    pub mcp_process: Mutex<Option<std::process::Child>>,
    pub mcp_error: Arc<Mutex<Option<String>>>,
    pub is_quitting: AtomicBool,
    pub always_on_top: AtomicBool,
    pub close_to_minimize: AtomicBool,
    pub snap_to_edges: AtomicBool,
    pub window_opacity: Mutex<f64>,
    pub toggle_shortcut: Mutex<Shortcut>,
    pub clipboard_shortcut: Mutex<Shortcut>,
    pub startup_hidden: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub root: String,
    pub notes_dir: String,
    pub meta_dir: String,
    pub trash_dir: String,
    pub backups_dir: String,
    pub config_path: String,
    pub tabs_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfigInfo {
    pub initialized: bool,
    pub ui: UiConfig,
    pub preview_mode: PreviewMode,
    pub theme: Theme,
}

#[tauri::command]
pub fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
pub fn get_workspace_command(state: State<'_, AppState>) -> WorkspaceInfo {
    WorkspaceInfo::from(&state.workspace)
}

#[tauri::command]
pub fn get_ui_config_command(state: State<'_, AppState>) -> Result<UiConfigInfo, String> {
    load_config(&state.workspace)
        .map(|config| UiConfigInfo {
            initialized: config.version >= 2,
            ui: config.ui,
            preview_mode: config.preview_mode,
            theme: config.theme,
        })
        .map_err(display_error)
}

#[tauri::command]
pub fn save_ui_config_command(
    state: State<'_, AppState>,
    ui: UiConfig,
    preview_mode: PreviewMode,
    theme: Theme,
) -> Result<(), String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    let mut config = load_config(&state.workspace).map_err(display_error)?;
    config.version = 2;
    config.start_at_login = ui.run_at_startup;
    config.default_open_mode = "edit".to_owned();
    config.preview_mode = preview_mode;
    config.theme = theme;
    config.default_hotkey = shortcut_label(&ui.shortcut_base_key, &ui.shortcut_modifiers);
    config.clipboard_hotkey = shortcut_label(
        &ui.clipboard_shortcut_base_key,
        &ui.clipboard_shortcut_modifiers,
    );
    config.ui = ui;
    save_config(&state.workspace, &config).map_err(display_error)
}

fn shortcut_label(base_key: &str, modifiers: &[String]) -> String {
    modifiers
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(base_key.trim()))
        .collect::<Vec<_>>()
        .join("+")
}

#[tauri::command]
pub fn list_notes_command(state: State<'_, AppState>) -> Result<Vec<NoteTab>, String> {
    list_notes(&state.workspace).map_err(display_error)
}

#[tauri::command]
pub fn read_note_command(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<NoteContent, String> {
    read_note(&state.workspace, &note_id).map_err(display_error)
}

#[tauri::command]
pub fn create_note_command(
    state: State<'_, AppState>,
    title: Option<String>,
) -> Result<NoteContent, String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    create_note(&state.workspace, title).map_err(display_error)
}

#[tauri::command]
pub fn write_note_command(
    state: State<'_, AppState>,
    note_id: String,
    content: String,
    expected_updated_at: i64,
) -> Result<NoteContent, String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    write_note_atomic_checked(&state.workspace, &note_id, &content, expected_updated_at)
        .map_err(display_error)
}

#[tauri::command]
pub fn rename_note_command(
    state: State<'_, AppState>,
    note_id: String,
    title: String,
) -> Result<NoteTab, String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    rename_note(&state.workspace, &note_id, title).map_err(display_error)
}

#[tauri::command]
pub fn delete_note_command(state: State<'_, AppState>, note_id: String) -> Result<NoteTab, String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    delete_note_to_trash(&state.workspace, &note_id).map_err(display_error)
}

#[tauri::command]
pub fn set_note_color_command(
    state: State<'_, AppState>,
    note_id: String,
    color: Option<String>,
) -> Result<NoteTab, String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    neopad_core::set_note_color(&state.workspace, &note_id, color).map_err(display_error)
}

#[tauri::command]
pub fn search_notes_command(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>, String> {
    search_notes(&state.workspace, &query, limit.unwrap_or(100)).map_err(display_error)
}

#[tauri::command]
pub fn list_reminders_command(state: State<'_, AppState>) -> Result<Vec<Reminder>, String> {
    list_reminders(&state.workspace).map_err(display_error)
}

#[tauri::command]
pub fn claim_due_reminders_command(state: State<'_, AppState>) -> Result<Vec<Reminder>, String> {
    claim_due_reminders(&state.workspace).map_err(display_error)
}

#[tauri::command]
pub fn complete_reminder_command(
    state: State<'_, AppState>,
    note_id: String,
    line_number: usize,
    reminder_id: String,
) -> Result<(), String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    complete_reminder(&state.workspace, &note_id, line_number, &reminder_id).map_err(display_error)
}

#[tauri::command]
pub fn reopen_reminder_command(
    state: State<'_, AppState>,
    note_id: String,
    line_number: usize,
    reminder_id: String,
) -> Result<(), String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    reopen_reminder(&state.workspace, &note_id, line_number, &reminder_id).map_err(display_error)
}

#[tauri::command]
pub fn complete_due_reminders_command(state: State<'_, AppState>) -> Result<usize, String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    complete_due_reminders(&state.workspace).map_err(display_error)
}

#[tauri::command]
pub async fn save_markdown_file_command(
    window: tauri::WebviewWindow,
    suggested_file_name: String,
    content: String,
) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = rfd::FileDialog::new()
            .set_parent(&window)
            .set_title("Save Markdown file")
            .set_file_name(&suggested_file_name)
            .add_filter("Markdown", &["md", "markdown"])
            .save_file();

        let Some(path) = path else {
            return Ok(false);
        };

        export_note_file(&path, &content).map_err(display_error)?;
        Ok(true)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn export_all_notes_zip_command(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let workspace = state.workspace.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = rfd::FileDialog::new()
            .set_parent(&window)
            .set_title("Export NeoPad notes")
            .set_file_name("neopad-export.zip")
            .add_filter("Zip archive", &["zip"])
            .save_file();

        let Some(path) = path else {
            return Ok(false);
        };

        let file = File::create(&path).map_err(|error| error.to_string())?;
        let mut archive = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for tab in list_notes(&workspace).map_err(display_error)? {
            let note = read_note(&workspace, &tab.id).map_err(display_error)?;
            archive
                .start_file(&tab.file_name, options)
                .map_err(|error| error.to_string())?;
            archive
                .write_all(note.content.as_bytes())
                .map_err(|error| error.to_string())?;
        }

        archive.finish().map_err(|error| error.to_string())?;
        Ok(true)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn show_window_command(app: AppHandle) -> Result<(), String> {
    crate::window::show_main_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn hide_window_command(app: AppHandle) -> Result<(), String> {
    crate::window::hide_main_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_autostart_command(
    app: AppHandle,
    enabled: bool,
    start_hidden: bool,
) -> Result<(), String> {
    set_autostart(&app, enabled, start_hidden).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_start_hidden_command(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    let mut config = load_config(&state.workspace).map_err(display_error)?;
    config.ui.start_hidden = enabled;
    save_config(&state.workspace, &config).map_err(display_error)
}

#[tauri::command]
pub fn complete_startup_command(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if !state.startup_hidden {
        crate::window::show_main_window(&app).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_close_to_minimize_command(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    state.close_to_minimize.store(enabled, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn set_snap_to_edges_command(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    state.snap_to_edges.store(enabled, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn set_window_opacity_command(app: AppHandle, opacity: f64) -> Result<(), String> {
    crate::window::set_main_window_opacity(&app, opacity)
}

#[tauri::command]
pub fn update_toggle_shortcut_command(
    app: AppHandle,
    state: State<'_, AppState>,
    base_key: String,
    modifiers: Vec<String>,
) -> Result<(), String> {
    crate::hotkey::update_toggle_window_shortcut(&app, &state, &base_key, &modifiers)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_clipboard_shortcut_command(
    app: AppHandle,
    state: State<'_, AppState>,
    base_key: String,
    modifiers: Vec<String>,
) -> Result<(), String> {
    crate::hotkey::update_clipboard_shortcut(&app, &state, &base_key, &modifiers)
        .map_err(display_error)
}

#[tauri::command]
pub fn toggle_main_window_maximize_command(app: AppHandle) -> Result<(), String> {
    crate::window::toggle_main_window_maximize(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_trash_command(state: State<'_, AppState>) -> Result<(), String> {
    open_path(&state.workspace.trash_dir).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn quit_app_command(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    set_quitting(&state);
    crate::mcp::stop_owned_process(&state).map_err(display_error)?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn toggle_window_command(app: AppHandle) -> Result<(), String> {
    crate::window::toggle_main_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn toggle_always_on_top_command(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let enabled = !state.always_on_top.load(Ordering::SeqCst);
    crate::window::set_main_window_always_on_top(&app, enabled)
        .map_err(|error| error.to_string())?;
    state.always_on_top.store(enabled, Ordering::SeqCst);
    Ok(enabled)
}

#[tauri::command]
pub fn save_clipboard_command(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<NoteContent, String> {
    save_clipboard_text(&app, &state)
}

#[tauri::command]
pub fn get_shortcut_warnings_command(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state
        .shortcut_warnings
        .lock()
        .map(|warnings| warnings.clone())
        .map_err(|error| error.to_string())
}

pub fn save_clipboard_text(app: &AppHandle, state: &AppState) -> Result<NoteContent, String> {
    let text = app
        .clipboard()
        .read_text()
        .map_err(|error| format!("failed to read text clipboard: {error}"))?;

    let _lock = lock_workspace_for_write(&state.workspace).map_err(display_error)?;
    append_to_clipboard_note(&state.workspace, &text).map_err(display_error)
}

pub fn set_quitting(state: &AppState) {
    state.is_quitting.store(true, Ordering::SeqCst);
}

impl From<&Workspace> for WorkspaceInfo {
    fn from(workspace: &Workspace) -> Self {
        Self {
            root: path_to_string(&workspace.root),
            notes_dir: path_to_string(&workspace.notes_dir),
            meta_dir: path_to_string(&workspace.meta_dir),
            trash_dir: path_to_string(&workspace.trash_dir),
            backups_dir: path_to_string(&workspace.backups_dir),
            config_path: path_to_string(&workspace.config_path),
            tabs_path: path_to_string(&workspace.tabs_path),
        }
    }
}

fn path_to_string(path: &std::path::Path) -> String {
    path.to_string_lossy().to_string()
}

fn open_path(path: &std::path::Path) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer");
        command.arg(path);
        command
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(path);
        command
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };

    command.spawn()?;
    Ok(())
}

fn set_autostart(_app: &AppHandle, enabled: bool, start_hidden: bool) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let exe_path = std::env::current_exe()?;
        let command = autostart_command_value(&exe_path, start_hidden);
        let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
        let status = if enabled {
            Command::new("reg")
                .args([
                    "add", key, "/v", "NeoPad", "/t", "REG_SZ", "/d", &command, "/f",
                ])
                .status()?
        } else {
            Command::new("reg")
                .args(["delete", key, "/v", "NeoPad", "/f"])
                .status()?
        };
        if !status.success() && enabled {
            anyhow::bail!("failed to update Windows Run registry key");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, enabled, start_hidden);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn autostart_command_value(exe_path: &std::path::Path, start_hidden: bool) -> String {
    let exe = exe_path.to_string_lossy();
    if start_hidden {
        format!(r#""{}" --hidden"#, exe)
    } else {
        format!(r#""{}""#, exe)
    }
}

pub(crate) fn display_error(error: anyhow::Error) -> String {
    error
        .chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(": ")
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use super::autostart_command_value;
    use std::path::Path;

    #[test]
    fn autostart_command_quotes_exe_without_escape_slashes() {
        let command =
            autostart_command_value(Path::new(r"C:\Program Files\neopad\neopad-app.exe"), false);

        assert_eq!(command, r#""C:\Program Files\neopad\neopad-app.exe""#);
    }

    #[test]
    fn autostart_command_appends_hidden_argument() {
        let command =
            autostart_command_value(Path::new(r"C:\Program Files\neopad\neopad-app.exe"), true);

        assert_eq!(
            command,
            r#""C:\Program Files\neopad\neopad-app.exe" --hidden"#
        );
    }
}
