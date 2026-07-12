mod commands;
mod hotkey;
mod mcp;
mod tray;
mod window;

use commands::{
    app_version, archive_note_command, claim_due_reminders_command, clear_trash_command,
    close_note_command, complete_due_reminders_command, complete_reminder_command,
    complete_startup_command, create_note_command, delete_note_command,
    export_all_notes_zip_command, get_shortcut_warnings_command, get_ui_config_command,
    get_workspace_command, hide_window_command, list_archived_notes_command,
    list_library_notes_command, list_notes_command, list_recent_notes_command,
    list_reminders_command, list_trashed_notes_command, open_external_markdown_command,
    open_external_url_command, open_note_command, open_trash_command, quit_app_command,
    read_external_markdown_command, read_note_command, rename_note_command,
    reopen_reminder_command, restore_note_from_trash_command, save_clipboard_command,
    save_markdown_file_command, save_ui_config_command, search_notes_command,
    set_autostart_command, set_close_to_minimize_command, set_note_color_command,
    set_snap_to_edges_command, set_start_hidden_command, set_window_opacity_command,
    show_window_command, toggle_always_on_top_command, toggle_main_window_maximize_command,
    toggle_window_command, unarchive_note_command, update_clipboard_shortcut_command,
    update_toggle_shortcut_command, write_external_markdown_command, write_note_command, AppState,
};
use neopad_core::{init_workspace, load_config};
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
use tauri_plugin_window_state::StateFlags;

fn build_state() -> anyhow::Result<AppState> {
    let workspace_path = std::env::var_os("NEOPAD_WORKSPACE").map(std::path::PathBuf::from);
    let workspace = init_workspace(workspace_path)?;
    let startup_hidden = std::env::args_os().any(|arg| arg == "--hidden")
        || load_config(&workspace)
            .map(|config| config.ui.start_hidden)
            .unwrap_or(false);
    Ok(AppState {
        workspace,
        shortcut_warnings: Mutex::new(Vec::new()),
        mcp_process: Mutex::new(None),
        mcp_error: Arc::new(Mutex::new(None)),
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
        startup_hidden,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = match build_state() {
        Ok(state) => state,
        Err(error) => {
            let message = commands::display_error(error);
            eprintln!("failed to initialize NeoPad workspace: {message}");
            let _ = rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("NeoPad failed to start")
                .set_description(format!(
                    "NeoPad could not initialize its workspace.\n\n{message}"
                ))
                .show();
            return;
        }
    };

    let mut builder = tauri::Builder::default();
    if std::env::var_os("NEOPAD_E2E").is_none() {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = window::show_main_window(app);
        }));
    }

    builder
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::POSITION)
                .build(),
        )
        .manage(state)
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
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
            if let Err(error) = mcp::start_if_enabled(app.state::<AppState>().inner()) {
                eprintln!("failed to start MCP service: {error:#}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            complete_startup_command,
            get_workspace_command,
            get_ui_config_command,
            save_ui_config_command,
            list_notes_command,
            list_library_notes_command,
            list_archived_notes_command,
            list_trashed_notes_command,
            read_note_command,
            create_note_command,
            write_note_command,
            rename_note_command,
            delete_note_command,
            restore_note_from_trash_command,
            clear_trash_command,
            close_note_command,
            open_note_command,
            open_external_markdown_command,
            read_external_markdown_command,
            write_external_markdown_command,
            archive_note_command,
            unarchive_note_command,
            set_note_color_command,
            list_recent_notes_command,
            search_notes_command,
            list_reminders_command,
            claim_due_reminders_command,
            complete_reminder_command,
            reopen_reminder_command,
            complete_due_reminders_command,
            save_markdown_file_command,
            export_all_notes_zip_command,
            show_window_command,
            hide_window_command,
            set_autostart_command,
            set_start_hidden_command,
            set_close_to_minimize_command,
            set_snap_to_edges_command,
            set_window_opacity_command,
            update_toggle_shortcut_command,
            update_clipboard_shortcut_command,
            toggle_main_window_maximize_command,
            open_trash_command,
            open_external_url_command,
            quit_app_command,
            toggle_window_command,
            toggle_always_on_top_command,
            save_clipboard_command,
            get_shortcut_warnings_command,
            mcp::get_mcp_status_command,
            mcp::set_mcp_enabled_command,
            mcp::regenerate_mcp_token_command,
            tray::set_tray_language_command
        ])
        .run(tauri::generate_context!())
        .expect("failed to run NeoPad");
}
