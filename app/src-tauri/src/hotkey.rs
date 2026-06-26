use crate::commands::{save_clipboard_text, AppState};
use crate::window::{hide_main_window, toggle_main_window};
use anyhow::{bail, Result};
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

    let state = app.state::<AppState>();
    let toggle_shortcut = state
        .toggle_shortcut
        .lock()
        .map(|shortcut| *shortcut)
        .unwrap_or_else(|_| toggle_window_shortcut());

    if shortcut == &toggle_shortcut {
        let _ = toggle_main_window(app);
    } else if shortcut == &save_clipboard_shortcut() {
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

pub fn update_toggle_window_shortcut(
    app: &tauri::AppHandle,
    state: &AppState,
    base_key: &str,
    modifiers: &[String],
) -> Result<()> {
    let shortcut = shortcut_from_parts(base_key, modifiers)?;
    let previous = *state
        .toggle_shortcut
        .lock()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    if previous == shortcut {
        return Ok(());
    }

    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister(previous);
    shortcuts.register(shortcut)?;

    let mut current = state
        .toggle_shortcut
        .lock()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    *current = shortcut;
    Ok(())
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

fn shortcut_from_parts(base_key: &str, modifiers: &[String]) -> Result<Shortcut> {
    let code = code_from_base_key(base_key)?;
    let mut parsed_modifiers = Modifiers::empty();

    for modifier in modifiers {
        match modifier.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => parsed_modifiers |= Modifiers::CONTROL,
            "alt" => parsed_modifiers |= Modifiers::ALT,
            "shift" => parsed_modifiers |= Modifiers::SHIFT,
            "win" | "super" | "meta" => parsed_modifiers |= Modifiers::SUPER,
            _ => bail!("unsupported modifier: {modifier}"),
        }
    }

    let modifiers = if parsed_modifiers.is_empty() {
        None
    } else {
        Some(parsed_modifiers)
    };

    Ok(Shortcut::new(modifiers, code))
}

fn code_from_base_key(base_key: &str) -> Result<Code> {
    let normalized = base_key.trim().to_ascii_uppercase();
    let code = match normalized.as_str() {
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyW,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,
        "0" => Code::Digit0,
        "1" => Code::Digit1,
        "2" => Code::Digit2,
        "3" => Code::Digit3,
        "4" => Code::Digit4,
        "5" => Code::Digit5,
        "6" => Code::Digit6,
        "7" => Code::Digit7,
        "8" => Code::Digit8,
        "9" => Code::Digit9,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        _ => bail!("unsupported base key: {base_key}"),
    };
    Ok(code)
}
