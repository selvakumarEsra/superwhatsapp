//! Runs the core on its own thread (REQ-WEBUI-001, 002). The core's state
//! machine still lives in `zapfast::app::App`, which steps once per wake with
//! an egui context that draws nothing; the page gets a new [`view::State`]
//! whenever what it shows changes.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};
use zapfast::app::App;
use zapfast::model::Action;

use crate::view;

/// The fastest the core steps, so a busy animation cannot spin the thread.
const MIN_STEP: Duration = Duration::from_millis(16);
/// The core also steps this often with nothing to do (timers, presence).
const IDLE_STEP: Duration = Duration::from_millis(500);

pub enum Input {
    Action(Action),
    /// Changes the core's state directly, for settings the egui build
    /// changes in place rather than through an action.
    Edit(Box<dyn FnOnce(&mut App) + Send>),
    Focused(bool),
    /// The person used the page; the app lock's idle timer restarts.
    Activity,
    /// Saves what is unsaved and ends the app.
    Quit,
    /// The page asks for the current state (on load).
    Refresh,
    Wake,
}

#[derive(Clone)]
pub struct Engine {
    tx: Sender<Input>,
    shared: Arc<Shared>,
}

/// What the main thread reads from the core without waiting on it, as of
/// the last step.
#[derive(Default)]
pub struct Shared {
    /// Settings › Keep running when the window closes.
    keep_running: AtomicBool,
    /// The interface language, for translating the page's text.
    locale: Mutex<zapfast::i18n::Locale>,
}

impl Engine {
    pub fn send(&self, input: Input) {
        let _ = self.tx.send(input);
    }

    /// Whether closing the window keeps SuperWhatsapp running in the tray.
    pub fn keeps_running(&self) -> bool {
        self.shared.keep_running.load(Ordering::Relaxed)
    }

    /// Translates the page's English text into the interface language, from
    /// the catalogues the egui build uses; untranslated text stays English.
    pub fn translate(&self, texts: &[String]) -> Vec<String> {
        use fastframe_i18n::Locale as _;
        let locale = self.shared.locale.lock().map(|l| *l).unwrap_or_default();
        let catalog = locale.catalog();
        texts
            .iter()
            .map(|text| {
                catalog.map_or_else(|| text.clone(), |c| c.translate(text, None).into_owned())
            })
            .collect()
    }
}

/// How the core starts.
pub struct Start {
    pub demo: bool,
    pub demo_page: Option<String>,
    /// The account's single-instance slot, held for the whole run; demo
    /// runs take none.
    pub instance: Option<Instance>,
}

/// The slot that keeps one SuperWhatsapp, this interface or the egui one, on an
/// account's archive and linked device at a time.
pub struct Instance {
    waker: zapfast::backend::Waker,
    guard: zapfast::single_instance::Guard,
}

/// Becomes the running SuperWhatsapp, or asks the copy already running to show
/// its window and returns `None`, as the egui build's launch does.
pub fn claim() -> Option<Instance> {
    use zapfast::single_instance::{Outcome, acquire};
    let dirs = zapfast::paths::AppDirs::discover();
    let waker = zapfast::backend::Waker::default();
    match acquire(&dirs.runtime, &waker, "show") {
        Outcome::Only(guard) => Some(Instance { waker, guard }),
        _ => {
            eprintln!("SuperWhatsapp is already running; asked it to show its window");
            None
        }
    }
}

impl Instance {
    /// Whether a later launch asked for the window, before the core takes
    /// the request: the core's own answer only reaches an egui window.
    fn wants_window(&self) -> bool {
        use zapfast::single_instance::ControlCommand;
        self.guard.commands().lock().is_ok_and(|queue| {
            queue.iter().any(|command| {
                matches!(
                    command,
                    ControlCommand::Show
                        | ControlCommand::OpenChat(_)
                        | ControlCommand::OpenInvite(_)
                )
            })
        })
    }
}

pub fn spawn(handle: AppHandle, start: Start) -> Engine {
    let (tx, rx) = channel();
    let wake = tx.clone();
    let shared = Arc::new(Shared::default());
    let core = shared.clone();
    std::thread::Builder::new()
        .name("zapfast-core".into())
        .spawn(move || run(handle, start, rx, wake, core))
        .expect("core thread");
    Engine { tx, shared }
}

fn make_app(start: &Start) -> App {
    use zapfast::paths::AppDirs;
    use zapfast::settings::Settings;
    if start.demo {
        let root = std::env::temp_dir().join(format!("zapfast-tauri-demo-{}", std::process::id()));
        let dirs = AppDirs::under(&root);
        dirs.ensure().expect("demo folders");
        let settings = Settings::load(&dirs.settings_file());
        let mut app = App::headless(dirs, settings).0;
        zapfast::demo::populate(&mut app);
        zapfast::demo::apply_flags(&mut app, start.demo_page.as_deref());
        return app;
    }
    let dirs = AppDirs::discover();
    dirs.adopt_previous_names().expect("app folders");
    dirs.ensure().expect("app folders");
    dirs.adopt_single_account().expect("account folder");
    let settings = Settings::load(&dirs.settings_file());
    let instance = start
        .instance
        .as_ref()
        .expect("a linked run holds the slot");
    let mut app = App::new(
        &instance.waker,
        dirs,
        settings,
        zapfast::app::AppOptions { tray: false },
    )
    .expect("core");
    app.set_remote_control(&instance.guard);
    app
}

fn run(
    handle: AppHandle,
    start: Start,
    rx: Receiver<Input>,
    wake: Sender<Input>,
    shared: Arc<Shared>,
) {
    let mut app = make_app(&start);
    let ctx = egui::Context::default();
    ctx.set_request_repaint_callback(move |_| {
        let _ = wake.send(Input::Wake);
    });
    app.attach(&ctx);
    let mut focused = true;
    let mut last: Option<view::State> = None;
    // The unread count last put on the Dock or taskbar. Demo runs show it
    // too: their tile is the development build's, not the installed app's.
    let mut badge: Option<u32> = None;
    let mut last_step = Instant::now() - MIN_STEP;
    loop {
        let first = match rx.recv_timeout(IDLE_STEP) {
            Ok(input) => Some(input),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => None,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
        };
        let mut force = false;
        for input in first.into_iter().chain(rx.try_iter()) {
            match input {
                Input::Action(action) => app.actions.push(action),
                Input::Edit(edit) => edit(&mut app),
                Input::Focused(now) => focused = now,
                Input::Activity => app.app_lock.note_input(),
                Input::Quit => {
                    app.shutdown();
                    handle.exit(0);
                    return;
                }
                Input::Refresh => force = true,
                Input::Wake => {}
            }
        }
        if start.instance.as_ref().is_some_and(Instance::wants_window) {
            let shown = handle.clone();
            let _ = handle.run_on_main_thread(move || crate::desktop::show(&shown));
        }
        let since = last_step.elapsed();
        if since < MIN_STEP {
            std::thread::sleep(MIN_STEP - since);
        }
        last_step = Instant::now();
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1440.0, 900.0),
            )),
            ..Default::default()
        };
        if let Some(viewport) = input.viewports.get_mut(&egui::ViewportId::ROOT) {
            viewport.focused = Some(focused);
        }
        let mut output = ctx.run_ui(input, |ui| {
            let ctx = ui.ctx().clone();
            app.background_frame(&ctx);
        });
        // Nothing is drawn, so texture uploads are discarded.
        output.textures_delta.clear();
        shared
            .keep_running
            .store(app.settings.keep_running_in_background, Ordering::Relaxed);
        if let Ok(mut locale) = shared.locale.lock() {
            *locale = app.locale;
        }
        let count = if app.app_lock.is_locked() {
            0
        } else {
            app.unread_chat_count_everywhere()
        };
        if badge != Some(count) {
            badge = Some(count);
            crate::desktop::set_badge(&handle, count, app.locale);
        }
        let state = view::build(&mut app);
        if force || last.as_ref() != Some(&state) {
            if let Err(error) = handle.emit("zapfast://state", &state) {
                log::warn!("could not send the state to the page: {error}");
            }
            last = Some(state);
        }
    }
}

/// The demo's state after a few steps, without a window (`--dump-state`).
pub fn dump(page: Option<String>) -> view::State {
    let mut app = make_app(&Start {
        demo: true,
        demo_page: page,
        instance: None,
    });
    let ctx = egui::Context::default();
    app.attach(&ctx);
    for _ in 0..3 {
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let ctx = ui.ctx().clone();
            app.background_frame(&ctx);
        });
        output.textures_delta.clear();
    }
    view::build(&mut app)
}

#[cfg(test)]
mod translation_tests {
    use super::*;

    #[test]
    fn page_text_follows_the_interface_language() {
        let (tx, _rx) = channel();
        let engine = Engine {
            tx,
            shared: Arc::new(Shared::default()),
        };
        let texts = ["Search".to_owned(), "Not in any catalogue".to_owned()];
        assert_eq!(engine.translate(&texts), texts);
        *engine.shared.locale.lock().unwrap() = zapfast::i18n::Locale::Dutch;
        assert_eq!(
            engine.translate(&texts),
            ["Zoeken".to_owned(), "Not in any catalogue".to_owned()]
        );
    }
}
