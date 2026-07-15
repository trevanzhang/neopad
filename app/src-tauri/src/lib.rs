mod ai;
mod commands;
mod hotkey;
mod installer;
mod mcp;
mod tray;
mod window;

use commands::{
    app_version, archive_note_command, archive_note_to_directory_command,
    claim_due_reminders_command, clear_trash_command, close_note_command,
    complete_due_reminders_command, complete_reminder_command, complete_startup_command,
    copy_external_markdown_path_to_clipboard_command, copy_note_path_to_clipboard_command,
    create_archive_directory_command, create_note_command, create_note_with_body_command,
    delete_archive_directory_command, delete_note_command, export_all_notes_zip_command,
    get_shortcut_warnings_command, get_ui_config_command, get_workspace_command,
    hide_window_command, list_archive_directories_command, list_archived_notes_command,
    list_library_notes_command, list_notes_command, list_recent_notes_command,
    list_recoverable_note_writes_command, list_reminders_command, list_trashed_notes_command,
    move_archive_directory_command, move_archived_note_command, open_external_markdown_command,
    open_external_markdown_paths_command, open_external_url_command, open_note_command,
    open_trash_command, open_workspace_in_file_manager_command, quit_app_command,
    read_external_markdown_command, read_note_command, rename_archive_directory_command,
    rename_note_command, rename_note_with_heading_command, reopen_reminder_command,
    reorder_open_notes_command, restore_note_from_trash_command,
    restore_recoverable_note_write_command, reveal_external_markdown_in_file_manager_command,
    reveal_note_in_file_manager_command, save_clipboard_command, save_markdown_file_command,
    save_note_export_command, save_ui_config_command, search_notes_command, set_autostart_command,
    set_close_to_minimize_command, set_note_color_command, set_snap_to_edges_command,
    set_start_hidden_command, set_window_opacity_command, show_window_command,
    take_pending_external_markdown_paths_command, toggle_always_on_top_command,
    toggle_main_window_maximize_command, toggle_window_command, unarchive_note_command,
    update_clipboard_shortcut_command, update_toggle_shortcut_command,
    write_external_markdown_command, write_note_command, AppState,
};
use neopad_core::{default_workspace_dir, init_workspace, load_config, save_config};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
use tauri_plugin_window_state::StateFlags;

fn build_state() -> anyhow::Result<AppState> {
    let workspace_path = std::env::var_os("NEOPAD_WORKSPACE").map(std::path::PathBuf::from);
    let should_apply_installer_language =
        workspace_path.is_none() && !default_workspace_dir()?.join("config.json").is_file();
    let workspace = init_workspace(workspace_path)?;
    if should_apply_installer_language {
        if let Some(language) = installer::selected_language() {
            let mut config = load_config(&workspace)?;
            config.ui.language = language.to_owned();
            save_config(&workspace, &config)?;
        }
    }
    let startup_args = std::env::args_os().collect::<Vec<_>>();
    let startup_hidden = startup_args.iter().any(|arg| arg == "--hidden")
        || load_config(&workspace)
            .map(|config| config.ui.start_hidden)
            .unwrap_or(false);
    let startup_cwd = std::env::current_dir().unwrap_or_default();
    Ok(AppState {
        workspace,
        pending_external_markdown_paths: Mutex::new(external_markdown_paths_from_args(
            startup_args,
            &startup_cwd,
        )),
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

const EXTERNAL_MARKDOWN_OPEN_REQUEST: &str = "neopad://external-markdown-open-requested";

fn external_markdown_paths_from_args(
    args: impl IntoIterator<Item = OsString>,
    cwd: &Path,
) -> Vec<String> {
    args.into_iter()
        .skip(1)
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            }
        })
        .filter(|path| {
            matches!(
                path.extension()
                    .and_then(|extension| extension.to_str())
                    .map(|extension| extension.to_ascii_lowercase())
                    .as_deref(),
                Some("md" | "markdown")
            )
        })
        .map(|path| path.to_string_lossy().to_string())
        .collect()
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
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            let paths = external_markdown_paths_from_args(
                args.into_iter().map(OsString::from),
                Path::new(&cwd),
            );
            if !paths.is_empty() {
                if let Ok(mut pending) = app
                    .state::<AppState>()
                    .pending_external_markdown_paths
                    .lock()
                {
                    pending.extend(paths.iter().cloned());
                }
                let _ = app.emit(EXTERNAL_MARKDOWN_OPEN_REQUEST, paths);
            }
            let _ = window::show_main_window(app);
        }));
    }

    let app = builder
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
            ai::get_ai_config_command,
            ai::save_ai_config_command,
            ai::save_ai_api_key_command,
            ai::clear_ai_api_key_command,
            ai::test_ai_connection_command,
            ai::generate_ai_text_command,
            ai::list_ai_prompts_command,
            ai::list_ai_prompt_files_command,
            ai::list_ai_prompt_directories_command,
            ai::list_ai_trashed_prompts_command,
            ai::read_ai_prompt_command,
            ai::create_ai_prompt_command,
            ai::create_ai_prompt_directory_command,
            ai::move_ai_prompt_directory_command,
            ai::rename_ai_prompt_directory_command,
            ai::delete_ai_prompt_directory_command,
            ai::move_ai_prompt_command,
            ai::write_ai_prompt_command,
            ai::rename_ai_prompt_command,
            ai::trash_ai_prompt_command,
            ai::restore_ai_prompt_command,
            ai::reveal_ai_prompt_command,
            ai::copy_ai_prompt_path_command,
            ai::open_ai_prompts_folder_command,
            app_version,
            complete_startup_command,
            get_workspace_command,
            get_ui_config_command,
            save_ui_config_command,
            list_notes_command,
            list_library_notes_command,
            list_archived_notes_command,
            list_archive_directories_command,
            list_trashed_notes_command,
            list_recoverable_note_writes_command,
            read_note_command,
            create_note_command,
            create_note_with_body_command,
            write_note_command,
            rename_note_command,
            rename_note_with_heading_command,
            delete_note_command,
            restore_note_from_trash_command,
            restore_recoverable_note_write_command,
            clear_trash_command,
            close_note_command,
            open_note_command,
            open_external_markdown_command,
            open_external_markdown_paths_command,
            take_pending_external_markdown_paths_command,
            read_external_markdown_command,
            write_external_markdown_command,
            archive_note_command,
            archive_note_to_directory_command,
            move_archived_note_command,
            create_archive_directory_command,
            move_archive_directory_command,
            rename_archive_directory_command,
            delete_archive_directory_command,
            reorder_open_notes_command,
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
            save_note_export_command,
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
            reveal_note_in_file_manager_command,
            reveal_external_markdown_in_file_manager_command,
            copy_note_path_to_clipboard_command,
            copy_external_markdown_path_to_clipboard_command,
            open_workspace_in_file_manager_command,
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
        .build(tauri::generate_context!())
        .expect("failed to build NeoPad");

    app.run(|app_handle, event| {
        if matches!(
            event,
            tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. }
        ) {
            if let Err(error) = mcp::stop_owned_process(app_handle.state::<AppState>().inner()) {
                eprintln!("failed to stop MCP service during app exit: {error:#}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::external_markdown_paths_from_args;
    use std::ffi::OsString;

    #[test]
    fn startup_arguments_keep_only_markdown_paths() {
        let cwd = std::env::temp_dir().join("neopad-startup-arguments");
        let paths = external_markdown_paths_from_args(
            [
                OsString::from("neopad.exe"),
                OsString::from("readme.md"),
                OsString::from("--hidden"),
                OsString::from("notes.txt"),
                OsString::from("guide.MARKDOWN"),
            ],
            &cwd,
        );

        assert_eq!(
            paths,
            vec![
                cwd.join("readme.md").to_string_lossy().to_string(),
                cwd.join("guide.MARKDOWN").to_string_lossy().to_string(),
            ]
        );
    }
}
