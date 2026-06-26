use crate::commands::{save_clipboard_text, AppState};
use crate::window::{hide_main_window, toggle_main_window};
use tauri::{App, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn register_global_shortcuts(app: &App) {
    register_shortcut(app, toggle_window_shortcut(), "Alt+Z");
    register_shortcut(app, save_clipboard_shortcut(), "Ctrl+Shift+V");
    register_shortcut(app, hide_window_shortcut(), "Escape");
}

pub fn handle_global_shortcut(
    app: &tauri::AppHandle,
    shortcut: &Shortcut,
    event: tauri_plugin_global_shortcut::ShortcutEvent,
) {
    if event.state != ShortcutState::Pressed {
        return;
    }

    if shortcut == &toggle_window_shortcut() {
        let _ = toggle_main_window(app);
    } else if shortcut == &save_clipboard_shortcut() {
        let state = app.state::<AppState>();
        if save_clipboard_text(app, &state).is_ok() {
            let _ = app.emit("neopad://notes-changed", ());
        }
    } else if shortcut == &hide_window_shortcut() {
        let _ = hide_main_window(app);
    }
}

fn register_shortcut(app: &App, shortcut: Shortcut, label: &'static str) {
    let result = app.global_shortcut().register(shortcut);

    if let Err(error) = result {
        let state = app.state::<AppState>();
        if let Ok(mut warnings) = state.shortcut_warnings.lock() {
            warnings.push(format!("failed to register {label}: {error}"));
        };
    }
}

fn toggle_window_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::ALT), Code::KeyZ)
}

fn save_clipboard_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV)
}

fn hide_window_shortcut() -> Shortcut {
    Shortcut::new(None, Code::Escape)
}
