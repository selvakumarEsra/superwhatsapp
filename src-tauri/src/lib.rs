//! ZapFast's Tauri shell (specs/tauri-ui.md): the window, the commands the
//! page calls, and the media protocol. The core runs in [`engine`].

mod engine;
mod media;
mod view;

use tauri::{Manager, State, WindowEvent};
use zapfast::model::{Action, ChatFilter, Page};

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

/// The sidebar's destinations (REQ-WEBUI-006): chats, favourites,
/// archived, starred and calls. Each lands from wherever the window is.
#[tauri::command]
fn show_section(engine: State<'_, Engine>, section: String) {
    let mut actions = Vec::new();
    let calls = section == "calls";
    actions.push(Action::Open(if calls { Page::Calls } else { Page::Chats }));
    if !calls {
        actions.push(Action::ShowStarred(section == "starred"));
        actions.push(Action::ShowArchived(section == "archived"));
        match section.as_str() {
            "favourites" => actions.push(Action::SetChatFilter(ChatFilter::Favorites)),
            "chats" => actions.push(Action::SetChatFilter(ChatFilter::All)),
            _ => {}
        }
    }
    for action in actions {
        engine.send(Input::Action(action));
    }
}

/// Mutes until Unix time, for ever with 0, or unmutes with `None`.
#[tauri::command]
fn set_muted(engine: State<'_, Engine>, chat: String, until: Option<i64>) {
    engine.send(Input::Action(Action::SetMuted(chat, until)));
}

#[tauri::command]
fn set_archived(engine: State<'_, Engine>, chat: String, archived: bool) {
    engine.send(Input::Action(Action::SetArchived(chat, archived)));
}

#[tauri::command]
fn set_read(engine: State<'_, Engine>, chat: String, read: bool) {
    engine.send(Input::Action(if read {
        Action::MarkRead(chat)
    } else {
        Action::MarkUnread(chat)
    }));
}

/// Deleting and clearing reach the phone first; the page asks before.
#[tauri::command]
fn delete_chat(engine: State<'_, Engine>, chat: String) {
    engine.send(Input::Action(Action::DeleteChat(chat)));
}

#[tauri::command]
fn clear_chat(engine: State<'_, Engine>, chat: String) {
    engine.send(Input::Action(Action::ClearChat(chat)));
}

#[tauri::command]
fn open_info(engine: State<'_, Engine>, chat: String) {
    engine.send(Input::Action(Action::OpenInfo(chat)));
}

#[tauri::command]
fn close_info(engine: State<'_, Engine>) {
    engine.send(Input::Action(Action::CloseInfo));
}

#[tauri::command]
fn leave_group(engine: State<'_, Engine>, chat: String) {
    engine.send(Input::Action(Action::LeaveGroup {
        chat,
        archive: false,
    }));
}

#[tauri::command]
fn set_star(engine: State<'_, Engine>, chat: String, message: String, starred: bool) {
    engine.send(Input::Action(Action::SetStar {
        chat,
        message,
        starred,
    }));
}

#[tauri::command]
fn go_to_message(engine: State<'_, Engine>, chat: String, message: String) {
    engine.send(Input::Action(Action::OpenMessage { chat, message }));
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
            set_favorite,
            show_section,
            set_muted,
            set_archived,
            set_read,
            delete_chat,
            clear_chat,
            open_info,
            close_info,
            leave_group,
            set_star,
            go_to_message
        ])
        .run(tauri::generate_context!())
        .expect("ZapFast could not start");
}
