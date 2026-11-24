//! ZapFast's Tauri shell (specs/tauri-ui.md): the window, the commands the
//! page calls, and the media protocol. The core runs in [`engine`].

mod engine;
mod media;
mod view;

use tauri::{Manager, State, WindowEvent};
use zapfast::model::Action;

use engine::{Engine, Input, Start};

#[tauri::command]
fn refresh(engine: State<'_, Engine>) {
    engine.send(Input::Refresh);
}

#[tauri::command]
fn open_chat(engine: State<'_, Engine>, id: String) {
    engine.send(Input::Action(Action::OpenChat(id)));
}

#[tauri::command]
fn close_chat(engine: State<'_, Engine>) {
    engine.send(Input::Action(Action::CloseChat));
}

#[tauri::command]
fn send_text(engine: State<'_, Engine>, chat: String, text: String) {
    let text = text.trim().to_owned();
    if !text.is_empty() {
        engine.send(Input::Action(Action::SendText {
            chat,
            text,
            quoting: None,
        }));
    }
}

#[tauri::command]
fn set_filter(engine: State<'_, Engine>, filter: String) {
    if let Some(filter) = view::parse_filter(&filter) {
        engine.send(Input::Action(Action::SetChatFilter(filter)));
    }
}

#[tauri::command]
fn search(engine: State<'_, Engine>, text: String) {
    engine.send(Input::Action(Action::Search(text)));
}

#[tauri::command]
fn set_pinned(engine: State<'_, Engine>, chat: String, pinned: bool) {
    engine.send(Input::Action(Action::SetPinned(chat, pinned)));
}

#[tauri::command]
fn set_favorite(engine: State<'_, Engine>, chat: String, favorite: bool) {
    engine.send(Input::Action(Action::SetFavorite(chat, favorite)));
}

fn start() -> Start {
    let mut args = std::env::args().skip(1);
    let mut start = Start {
        demo: false,
        demo_page: None,
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--demo" => start.demo = true,
            "--demo-page" => {
                start.demo = true;
                start.demo_page = args.next();
            }
            _ => {}
        }
    }
    start
}

pub fn run() {
    // `--dump-state <page>`: print the demo's state as the page receives it
    // and exit, for screenshots and front-end tests (REQ-WEBUI-018).
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() == Some("--dump-state") {
        let state = engine::dump(args.next());
        println!("{}", serde_json::to_string_pretty(&state).expect("state"));
        return;
    }
    let start = start();
    tauri::Builder::default()
        .register_uri_scheme_protocol("zapmedia", media::serve)
        .setup(move |app| {
            let engine = engine::spawn(app.handle().clone(), start);
            app.manage(engine);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(focused) = event {
                window.state::<Engine>().send(Input::Focused(*focused));
            }
        })
        .invoke_handler(tauri::generate_handler![
            refresh,
            open_chat,
            close_chat,
            send_text,
            set_filter,
            search,
            set_pinned,
            set_favorite
        ])
        .run(tauri::generate_context!())
        .expect("ZapFast could not start");
}
