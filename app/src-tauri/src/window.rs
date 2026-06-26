use crate::commands::AppState;
use std::sync::atomic::Ordering;
use tauri::{image::Image, App, AppHandle, Manager, PhysicalPosition, WebviewWindow, WindowEvent};

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
            if !state.is_quitting.load(Ordering::SeqCst)
                && state.close_to_minimize.load(Ordering::SeqCst)
            {
                api.prevent_close();
                let _ = window_for_event.hide();
            }
        }

        if let WindowEvent::Moved(_) = event {
            let state = app_handle.state::<AppState>();
            if state.snap_to_edges.load(Ordering::SeqCst) {
                let _ = snap_window_to_edges(&window_for_event);
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

fn snap_window_to_edges(window: &WebviewWindow) -> tauri::Result<()> {
    const SNAP_DISTANCE: i32 = 16;

    let Some(monitor) = window.current_monitor()? else {
        return Ok(());
    };
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let window_position = window.outer_position()?;
    let window_size = window.outer_size()?;

    let left = monitor_position.x;
    let top = monitor_position.y;
    let right = monitor_position.x + monitor_size.width as i32 - window_size.width as i32;
    let bottom = monitor_position.y + monitor_size.height as i32 - window_size.height as i32;

    let mut next_x = window_position.x;
    let mut next_y = window_position.y;

    if (window_position.x - left).abs() <= SNAP_DISTANCE {
        next_x = left;
    } else if (window_position.x - right).abs() <= SNAP_DISTANCE {
        next_x = right;
    }

    if (window_position.y - top).abs() <= SNAP_DISTANCE {
        next_y = top;
    } else if (window_position.y - bottom).abs() <= SNAP_DISTANCE {
        next_y = bottom;
    }

    if next_x != window_position.x || next_y != window_position.y {
        window.set_position(PhysicalPosition::new(next_x, next_y))?;
    }

    Ok(())
}
