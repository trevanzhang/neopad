mod commands;
mod hotkey;
mod tray;
mod window;

use commands::{
    app_version, create_note_command, delete_note_command, get_shortcut_warnings_command,
    get_ui_config_command, get_workspace_command, hide_window_command, list_notes_command,
    open_trash_command, quit_app_command, read_note_command, rename_note_command,
    save_clipboard_command, save_markdown_file_command, save_ui_config_command,
    search_notes_command, set_autostart_command, set_close_to_minimize_command,
    set_note_color_command, set_snap_to_edges_command, set_window_opacity_command,
    show_window_command, toggle_always_on_top_command, toggle_main_window_maximize_command,
    toggle_window_command, update_clipboard_shortcut_command, update_toggle_shortcut_command,
    write_note_command, AppState,
};
use neopad_core::init_workspace;
use std::sync::{atomic::AtomicBool, Mutex};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
use tauri_plugin_window_state::StateFlags;

fn build_state() -> AppState {
    let workspace_path = std::env::var_os("NEOPAD_WORKSPACE").map(std::path::PathBuf::from);
    let workspace = init_workspace(workspace_path).expect("failed to initialize NeoPad workspace");
    AppState {
        workspace,
        shortcut_warnings: Mutex::new(Vec::new()),
        is_quitting: AtomicBool::new(false),
        always_on_top: AtomicBool::new(false),
        close_to_minimize: AtomicBool::new(true),
        snap_to_edges: AtomicBool::new(false),
        window_opacity: Mutex::new(1.0),
        toggle_shortcut: Mutex::new(Shortcut::new(Some(Modifiers::ALT), Code::KeyZ)),
        clipboard_shortcut: Mutex::new(Shortcut::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::KeyV,
        )),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = window::show_main_window(app);
        }))
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::POSITION)
                .build(),
        )
        .manage(build_state())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(hotkey::handle_global_shortcut)
                .build(),
        )
        .setup(|app| {
            window::install_main_window_icon(app);
            window::place_main_window_at_bottom_right(app);
            window::install_close_to_hide_handler(app);
            tray::create_tray(app)?;
            hotkey::register_global_shortcuts(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            get_workspace_command,
            get_ui_config_command,
            save_ui_config_command,
            list_notes_command,
            read_note_command,
            create_note_command,
            write_note_command,
            rename_note_command,
            delete_note_command,
            set_note_color_command,
            search_notes_command,
            save_markdown_file_command,
            show_window_command,
            hide_window_command,
            set_autostart_command,
            set_close_to_minimize_command,
            set_snap_to_edges_command,
            set_window_opacity_command,
            update_toggle_shortcut_command,
            update_clipboard_shortcut_command,
            toggle_main_window_maximize_command,
            open_trash_command,
            quit_app_command,
            toggle_window_command,
            toggle_always_on_top_command,
            save_clipboard_command,
            get_shortcut_warnings_command,
            tray::set_tray_language_command
        ])
        .run(tauri::generate_context!())
        .expect("failed to run NeoPad");
}
