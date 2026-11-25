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

/// Puts the unread chat count on the Dock tile (macOS), the launcher
/// (Linux, where the core also publishes it) or the taskbar button
/// (Windows), on the main thread as AppKit and the taskbar require.
pub fn set_badge<R: Runtime>(app: &AppHandle<R>, count: u32, locale: zapfast::i18n::Locale) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Some(window) = main_window(&handle) else {
            return;
        };
        #[cfg(target_os = "windows")]
        {
            // A COM object, so it stays on the main thread this runs on.
            thread_local! {
                static TASKBAR: std::cell::RefCell<Option<fastframe_dock::Taskbar>> =
                    const { std::cell::RefCell::new(None) };
            }
            let description = zapfast::notify::badge_description(count, locale);
            TASKBAR.with_borrow_mut(|taskbar| {
                let taskbar = taskbar.get_or_insert_with(fastframe_dock::Taskbar::default);
                let _ = taskbar.show(&window, count, &description);
            });
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = locale;
            let _ = window.set_badge_count((count > 0).then_some(i64::from(count)));
        }
    });
}
