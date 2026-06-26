mod commands;
mod hotkey;
mod tray;
mod window;

use commands::{
    app_version, create_note_command, delete_note_command, get_shortcut_warnings_command,
    get_workspace_command, hide_window_command, list_notes_command, open_trash_command,
    quit_app_command, read_note_command, rename_note_command, save_clipboard_command,
    search_notes_command, show_window_command, toggle_always_on_top_command, toggle_window_command,
    write_note_command, AppState,
};
use neopad_core::init_workspace;
use std::sync::{atomic::AtomicBool, Mutex};

fn build_state() -> AppState {
    let workspace = init_workspace(None).expect("failed to initialize NeoPad workspace");
    AppState {
        workspace,
        shortcut_warnings: Mutex::new(Vec::new()),
        is_quitting: AtomicBool::new(false),
        always_on_top: AtomicBool::new(false),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(build_state())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(hotkey::handle_global_shortcut)
                .build(),
        )
        .setup(|app| {
            window::install_main_window_icon(app);
            window::install_close_to_hide_handler(app);
            tray::create_tray(app)?;
            hotkey::register_global_shortcuts(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            get_workspace_command,
            list_notes_command,
            read_note_command,
            create_note_command,
            write_note_command,
            rename_note_command,
            delete_note_command,
            search_notes_command,
            show_window_command,
            hide_window_command,
            open_trash_command,
            quit_app_command,
            toggle_window_command,
            toggle_always_on_top_command,
            save_clipboard_command,
            get_shortcut_warnings_command
        ])
        .run(tauri::generate_context!())
        .expect("failed to run NeoPad");
}
