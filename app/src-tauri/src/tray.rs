use crate::commands::{save_clipboard_text, set_quitting, AppState};
use crate::window::{hide_main_window, show_main_window};
use neopad_core::create_note;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Emitter, Manager, Wry,
};

const APP_ICON: &[u8] = include_bytes!("../../src/assets/neopad-logo-small.png");

pub struct TrayMenuItems {
    show: MenuItem<Wry>,
    hide: MenuItem<Wry>,
    new_note: MenuItem<Wry>,
    save_clipboard: MenuItem<Wry>,
    settings: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

pub fn create_tray(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let new_note = MenuItem::with_id(app, "new_note", "New Note", true, None::<&str>)?;
    let save_clipboard =
        MenuItem::with_id(app, "save_clipboard", "Save Clipboard", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&show, &hide, &new_note, &save_clipboard, &settings, &quit],
    )?;
    let icon = Image::from_bytes(APP_ICON)?;

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("NeoPad")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                let _ = show_main_window(app);
            }
            "hide" => {
                let _ = hide_main_window(app);
            }
            "new_note" => {
                let state = app.state::<AppState>();
                if create_note(&state.workspace, None).is_ok() {
                    let _ = app.emit("neopad://notes-changed", ());
                    let _ = show_main_window(app);
                }
            }
            "save_clipboard" => {
                let state = app.state::<AppState>();
                if save_clipboard_text(app, &state).is_ok() {
                    let _ = app.emit("neopad://notes-changed", ());
                }
            }
            "settings" => {
                let _ = app.emit("neopad://open-settings", ());
                let _ = show_main_window(app);
            }
            "quit" => {
                let state = app.state::<AppState>();
                set_quitting(&state);
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    app.manage(TrayMenuItems {
        show,
        hide,
        new_note,
        save_clipboard,
        settings,
        quit,
    });

    Ok(())
}

#[tauri::command]
pub fn set_tray_language_command(app: AppHandle, language: String) -> Result<(), String> {
    let items = app.state::<TrayMenuItems>();
    let labels = if language == "zh" {
        ["显示", "隐藏", "新建笔记", "保存剪贴板", "设置", "退出"]
    } else {
        [
            "Show",
            "Hide",
            "New Note",
            "Save Clipboard",
            "Settings",
            "Quit",
        ]
    };

    items.show.set_text(labels[0]).map_err(display_error)?;
    items.hide.set_text(labels[1]).map_err(display_error)?;
    items.new_note.set_text(labels[2]).map_err(display_error)?;
    items
        .save_clipboard
        .set_text(labels[3])
        .map_err(display_error)?;
    items.settings.set_text(labels[4]).map_err(display_error)?;
    items.quit.set_text(labels[5]).map_err(display_error)?;
    Ok(())
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
