use crate::commands::AppState;
use std::sync::atomic::Ordering;
use tauri::{image::Image, App, AppHandle, Manager, WebviewWindow, WindowEvent};

const APP_ICON: &[u8] = include_bytes!("../../src/assets/neopad-logo-small.png");

pub fn install_main_window_icon(app: &App) {
    let Some(window) = main_window(app.handle()) else {
        return;
    };

    if let Ok(icon) = Image::from_bytes(APP_ICON) {
        let _ = window.set_icon(icon);
    }
}

pub fn install_close_to_hide_handler(app: &App) {
    let Some(window) = main_window(app.handle()) else {
        return;
    };
    let window_for_event = window.clone();
    let app_handle = app.handle().clone();

    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            let state = app_handle.state::<AppState>();
            if !state.is_quitting.load(Ordering::SeqCst) {
                api.prevent_close();
                let _ = window_for_event.hide();
            }
        }
    });
}

pub fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = main_window(app) {
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

pub fn hide_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = main_window(app) {
        window.hide()?;
    }
    Ok(())
}

pub fn toggle_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = main_window(app) {
        if window.is_visible()? {
            window.hide()?;
        } else {
            window.show()?;
            window.set_focus()?;
        }
    }
    Ok(())
}

pub fn set_main_window_always_on_top(app: &AppHandle, enabled: bool) -> tauri::Result<()> {
    if let Some(window) = main_window(app) {
        window.set_always_on_top(enabled)?;
    }
    Ok(())
}

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("main")
}
