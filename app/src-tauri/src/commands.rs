use neopad_core::{
    append_to_clipboard_note, create_note, delete_note_to_trash, list_notes, read_note,
    rename_note, search_notes, write_note_atomic, NoteContent, NoteTab, SearchResult, Workspace,
};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Debug)]
pub struct AppState {
    pub workspace: Workspace,
    pub shortcut_warnings: Mutex<Vec<String>>,
    pub is_quitting: AtomicBool,
    pub always_on_top: AtomicBool,
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

#[tauri::command]
pub fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
pub fn get_workspace_command(state: State<'_, AppState>) -> WorkspaceInfo {
    WorkspaceInfo::from(&state.workspace)
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
    create_note(&state.workspace, title).map_err(display_error)
}

#[tauri::command]
pub fn write_note_command(
    state: State<'_, AppState>,
    note_id: String,
    content: String,
) -> Result<NoteContent, String> {
    write_note_atomic(&state.workspace, &note_id, &content).map_err(display_error)
}

#[tauri::command]
pub fn rename_note_command(
    state: State<'_, AppState>,
    note_id: String,
    title: String,
) -> Result<NoteTab, String> {
    rename_note(&state.workspace, &note_id, title).map_err(display_error)
}

#[tauri::command]
pub fn delete_note_command(state: State<'_, AppState>, note_id: String) -> Result<NoteTab, String> {
    delete_note_to_trash(&state.workspace, &note_id).map_err(display_error)
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
pub fn show_window_command(app: AppHandle) -> Result<(), String> {
    crate::window::show_main_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn hide_window_command(app: AppHandle) -> Result<(), String> {
    crate::window::hide_main_window(&app).map_err(|error| error.to_string())
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

pub(crate) fn display_error(error: anyhow::Error) -> String {
    error
        .chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(": ")
}
