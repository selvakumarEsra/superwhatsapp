//! Desktop integration (REQ-WEBUI-012): the tray icon and its menu, the
//! window hiding to the tray when ZapFast keeps running, the hidden start at
//! login, and bringing the first copy forward when ZapFast starts again.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime, WebviewWindow, WindowEvent};

use crate::engine::{Engine, Input};

/// The flag the login item starts ZapFast with (src/autostart.rs).
pub const START_HIDDEN: &str = "--start-hidden";

fn main_window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window("main")
}

/// Shows, unminimises and focuses the window.
pub fn show<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = main_window(app) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Saves through the core, then ends the app.
pub fn quit<R: Runtime>(app: &AppHandle<R>) {
    match app.try_state::<Engine>() {
        Some(engine) => engine.send(Input::Quit),
        None => app.exit(0),
    }
}

/// The tray icon: a click shows the window; its menu shows it or quits.
pub fn setup(app: &tauri::App, start_hidden: bool) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "Show ZapFast", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit ZapFast", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("ZapFast")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show(app),
            "quit" => quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    // The window starts hidden (tauri.conf.json) so a login start never
    // flashes it; every other start shows it now.
    if !start_hidden {
        show(app.handle());
    }
    Ok(())
}

/// Closing the window hides it to the tray while ZapFast keeps running in
/// the background, and quits otherwise.
pub fn on_window_event<R: Runtime>(window: &tauri::Window<R>, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(focused) => {
            window.state::<Engine>().send(Input::Focused(*focused));
        }
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            if window.state::<Engine>().keeps_running() {
                let _ = window.hide();
            } else {
                quit(window.app_handle());
            }
        }
        _ => {}
    }
}
