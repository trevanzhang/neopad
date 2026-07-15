use crate::window::show_main_window;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
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
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if should_show_main_window(button, button_state) {
                    let _ = show_main_window(tray.app_handle());
                }
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                let _ = show_main_window(app);
            }
            "hide" => {
                let _ = app.emit("neopad://hide-requested", ());
            }
            "new_note" => {
                let _ = app.emit("neopad://new-note-requested", ());
                let _ = show_main_window(app);
            }
            "save_clipboard" => {
                let _ = app.emit("neopad://save-clipboard-requested", ());
            }
            "settings" => {
                let _ = app.emit("neopad://open-settings", ());
                let _ = show_main_window(app);
            }
            "quit" => {
                let _ = app.emit("neopad://quit-requested", ());
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

fn should_show_main_window(button: MouseButton, button_state: MouseButtonState) -> bool {
    button == MouseButton::Left && button_state == MouseButtonState::Up
}

#[cfg(test)]
mod tests {
    use super::should_show_main_window;
    use tauri::tray::{MouseButton, MouseButtonState};

    #[test]
    fn left_click_release_shows_the_main_window() {
        assert!(should_show_main_window(
            MouseButton::Left,
            MouseButtonState::Up
        ));
    }

    #[test]
    fn other_tray_mouse_events_do_not_show_the_main_window() {
        assert!(!should_show_main_window(
            MouseButton::Left,
            MouseButtonState::Down
        ));
        assert!(!should_show_main_window(
            MouseButton::Right,
            MouseButtonState::Up
        ));
    }
}
