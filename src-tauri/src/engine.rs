//! Runs the core on its own thread (REQ-WEBUI-001, 002). The core's state
//! machine still lives in `zapfast::app::App`, which steps once per wake with
//! an egui context that draws nothing; the page gets a new [`view::State`]
//! whenever what it shows changes.

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
    Focused(bool),
    /// The page asks for the current state (on load).
    Refresh,
    Wake,
}

#[derive(Clone)]
pub struct Engine {
    tx: Sender<Input>,
}

impl Engine {
    pub fn send(&self, input: Input) {
        let _ = self.tx.send(input);
    }
}

/// How the core starts.
pub struct Start {
    pub demo: bool,
    pub demo_page: Option<String>,
}

pub fn spawn(handle: AppHandle, start: Start) -> Engine {
    let (tx, rx) = channel();
    let wake = tx.clone();
    std::thread::Builder::new()
        .name("zapfast-core".into())
        .spawn(move || run(handle, start, rx, wake))
        .expect("core thread");
    Engine { tx }
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
    App::new(
        &zapfast::backend::Waker::default(),
        dirs,
        settings,
        zapfast::app::AppOptions { tray: false },
    )
    .expect("core")
}

fn run(handle: AppHandle, start: Start, rx: Receiver<Input>, wake: Sender<Input>) {
    let mut app = make_app(&start);
    let ctx = egui::Context::default();
    ctx.set_request_repaint_callback(move |_| {
        let _ = wake.send(Input::Wake);
    });
    app.attach(&ctx);
    let mut focused = true;
    let mut last: Option<view::State> = None;
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
                Input::Focused(now) => focused = now,
                Input::Refresh => force = true,
                Input::Wake => {}
            }
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
