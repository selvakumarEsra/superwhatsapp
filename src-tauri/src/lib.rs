//! SuperWhatsapp's Tauri shell (specs/tauri-ui.md): the window, the commands the
//! page calls, and the media protocol. The core runs in [`engine`].

mod desktop;
mod engine;
mod frames;
mod media;
mod view;
mod znga;

use tauri::{Manager, State};
use zapfast::model::{Action, ChatFilter, Page};

use engine::{Engine, Input, Start};

#[tauri::command]
fn refresh(engine: State<'_, Engine>) {
    engine.send(Input::Refresh);
}

#[tauri::command]
fn open_chat(engine: State<'_, Engine>, id: String) {
    if id == zapfast::znga::thread::ZNGA_CHAT {
        engine.send(Input::Edit(Box::new(znga::open)));
        return;
    }
    engine.send(Input::Edit(Box::new(move |app| {
        app.znga.chat_open = false;
        app.actions.push(Action::OpenChat(id));
    })));
}

#[tauri::command]
fn close_chat(engine: State<'_, Engine>) {
    engine.send(Input::Edit(Box::new(|app| {
        app.znga.chat_open = false;
        app.actions.push(Action::CloseChat);
    })));
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
        // The Locked plate, as egui's rail: the folder opens, or the code
        // prompt comes up; any other place locks it again.
        actions.push(if section == "locked" {
            Action::OpenLockedFolder
        } else {
            Action::CloseLockedFolder
        });
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

/// The info panel's overview, or its media view on one tab (egui's
/// `Action::ShowInfo`).
#[tauri::command]
fn info_page(engine: State<'_, Engine>, page: String) {
    use zapfast::model::{InfoView, MediaTab};
    let view = match page.as_str() {
        "media" => InfoView::Media(MediaTab::Media),
        "docs" => InfoView::Media(MediaTab::Docs),
        "links" => InfoView::Media(MediaTab::Links),
        _ => InfoView::Overview,
    };
    engine.send(Input::Action(Action::ShowInfo(view)));
}

/// Opens or closes the Calls tab's "New call" list.
#[tauri::command]
fn show_new_call(engine: State<'_, Engine>, open: bool) {
    engine.send(Input::Action(Action::ShowNewCall(open)));
}

/// The "New call" list's search, by name or number.
#[tauri::command]
fn search_new_call(engine: State<'_, Engine>, text: String) {
    engine.send(Input::Edit(Box::new(move |app| app.new_call_search = text)));
}

/// Asks again for the chat's media listing after it failed.
#[tauri::command]
fn retry_info_media(engine: State<'_, Engine>) {
    engine.send(Input::Action(Action::RetryInfoMedia));
}

/// The page scrolled to the jumped-to message and flashed it.
#[tauri::command]
fn jumped(engine: State<'_, Engine>) {
    engine.send(Input::Edit(Box::new(|app| app.jump_highlight = None)));
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

/// Changes one setting (REQ-WEBUI-012). Each key maps to the field or
/// action the egui build changes, so the value is stored the same way.
#[tauri::command]
fn set_setting(engine: State<'_, Engine>, key: String, value: serde_json::Value) {
    use zapfast::settings::{NotificationSound, ThemeChoice};
    let flag = value.as_bool();
    let text = value.as_str().map(str::to_owned);
    let global: Option<fn(&mut zapfast::settings::Settings) -> &mut bool> = match key.as_str() {
        "enterSends" => Some(|s| &mut s.enter_sends),
        "notificationContent" => Some(|s| &mut s.notification_content),
        "groupSounds" => Some(|s| &mut s.group_sounds),
        "keepRunning" => Some(|s| &mut s.keep_running_in_background),
        "checkForUpdates" => Some(|s| &mut s.check_for_updates),
        "downloadUpdates" => Some(|s| &mut s.download_updates_automatically),
        "showWallpaper" => Some(|s| &mut s.show_wallpaper),
        "pauseOtherMedia" => Some(|s| &mut s.pause_other_media),
        _ => None,
    };
    let account: Option<fn(&mut zapfast::settings::AccountSettings) -> &mut bool> =
        match key.as_str() {
            "notifications" => Some(|s| &mut s.notifications),
            "autoDownload" => Some(|s| &mut s.auto_download),
            "historyPrefetch" => Some(|s| &mut s.history_prefetch),
            "sendReadReceipts" => Some(|s| &mut s.send_read_receipts),
            "sendTyping" => Some(|s| &mut s.send_typing),
            _ => None,
        };
    let input = match (key.as_str(), global, account, flag) {
        (_, Some(field), _, Some(on)) => Input::Edit(Box::new(move |app| {
            *field(&mut app.settings) = on;
            app.actions.push(Action::SettingsChanged);
        })),
        (_, _, Some(field), Some(on)) => Input::Edit(Box::new(move |app| {
            *field(&mut app.account_mut().settings) = on;
            app.account_mut().mark_settings_dirty();
        })),
        ("keepChatsArchived", _, _, Some(on)) => Input::Action(Action::SetKeepChatsArchived(on)),
        ("startWithSystem", _, _, Some(on)) => Input::Action(Action::SetStartWithSystem(on)),
        ("showShortcutHints", _, _, Some(on)) => Input::Action(Action::SetShortcutHints(on)),
        ("font", ..) => Input::Action(Action::SetFont(match text.as_deref() {
            Some("inter") => zapfast::settings::FontChoice::Inter,
            _ => zapfast::settings::FontChoice::System,
        })),
        ("theme", ..) => Input::Action(Action::SetTheme(match text.as_deref() {
            Some("dark") => ThemeChoice::Dark,
            Some("light") => ThemeChoice::Light,
            _ => ThemeChoice::System,
        })),
        ("zoom", ..) => match value.as_f64() {
            Some(delta) if delta != 0.0 => Input::Action(Action::ZoomBy(delta as f32)),
            _ => Input::Action(Action::ResetZoom),
        },
        ("autoLock", ..) => Input::Action(Action::SetAutoLock(match text.as_deref() {
            Some("oneMinute") => zapfast::settings::AutoLock::OneMinute,
            Some("oneHour") => zapfast::settings::AutoLock::OneHour,
            _ => zapfast::settings::AutoLock::FifteenMinutes,
        })),
        ("proxy", ..) => Input::Action(Action::SetProxy(text.unwrap_or_default())),
        ("messageSound" | "mentionSound", ..) => {
            let sound = match text.as_deref() {
                Some("alert") => NotificationSound::Alert,
                Some("system") => NotificationSound::System,
                Some("none") => NotificationSound::None,
                _ => NotificationSound::Receive,
            };
            Input::Action(Action::SetNotificationSound {
                mention: key == "mentionSound",
                sound,
            })
        }
        ("name", ..) => Input::Action(Action::SetProfile {
            name: text,
            about: None,
        }),
        ("about", ..) => Input::Action(Action::SetProfile {
            name: None,
            about: text,
        }),
        _ => {
            log::warn!("unknown setting {key}");
            return;
        }
    };
    engine.send(input);
}

/// Tries a password on the lock screen. The password goes straight to the
/// core's check and is never logged.
#[tauri::command]
fn unlock(engine: State<'_, Engine>, password: String) {
    engine.send(Input::Edit(Box::new(move |app| {
        app.app_lock.entry = password;
        app.actions.push(Action::UnlockApp);
    })));
}

#[tauri::command]
fn lock_now(engine: State<'_, Engine>) {
    engine.send(Input::Action(Action::LockApp));
}

/// "Forgot password?": `open` asks whether to unlink; `unlink` unlinks
/// every account, which is the only way past a forgotten password.
#[tauri::command]
fn forgot_password(engine: State<'_, Engine>, open: bool, unlink: bool) {
    engine.send(Input::Action(if unlink {
        Action::UnlinkLockedApp
    } else {
        Action::ForgotAppPassword(open)
    }));
}

/// Sets, changes or turns off the app lock's password (Settings ›
/// Privacy). `mode` is set, change or turnOff; `None` closes the form.
#[tauri::command]
fn app_lock_form(
    engine: State<'_, Engine>,
    mode: Option<String>,
    current: String,
    new: String,
    confirm: String,
) {
    use zapfast::app_lock::{Form, FormMode};
    let mode = match mode.as_deref() {
        Some("set") => FormMode::Set,
        Some("change") => FormMode::Change,
        Some("turnOff") => FormMode::TurnOff,
        _ => {
            engine.send(Input::Action(Action::AppLockForm(None)));
            return;
        }
    };
    engine.send(Input::Edit(Box::new(move |app| {
        if app.app_lock.checking() {
            return;
        }
        app.app_lock.form = Some(Form {
            current,
            new,
            confirm,
            ..Form::new(mode)
        });
        app.actions.push(Action::SubmitAppLockForm);
    })));
}

/// The secret code that shows locked chats; `None` removes it.
#[tauri::command]
fn set_chat_lock_code(engine: State<'_, Engine>, code: Option<String>) {
    let code = code
        .map(|code| code.trim().to_owned())
        .filter(|code| !code.is_empty());
    engine.send(Input::Action(Action::SetChatLockCode(code)));
}

/// Opens a web page in the browser, as egui's Action::OpenUrl does. Only
/// http(s) links: the page never asks the system to open anything else.
#[tauri::command]
fn open_url(engine: State<'_, Engine>, url: String) {
    if url.starts_with("https://") || url.starts_with("http://") {
        engine.send(Input::Action(Action::OpenUrl(url)));
    }
}

/// Starts a voice or video call with a chat, as the egui call buttons do.
#[tauri::command]
fn start_call(engine: State<'_, Engine>, chat: String, video: bool) {
    engine.send(Input::Action(if video {
        Action::StartVideoCall(chat)
    } else {
        Action::StartCall(chat)
    }));
}

/// The call screen's buttons: answer, decline, hangup, mute, camera.
#[tauri::command]
fn call_control(engine: State<'_, Engine>, control: String, on: Option<bool>) {
    let action = match control.as_str() {
        "answer" => Action::AnswerCall,
        "decline" => Action::DeclineCall,
        "hangup" => Action::HangupCall,
        "mute" => Action::SetCallMuted(on.unwrap_or(true)),
        "camera" => Action::SetCallCamera(on.unwrap_or(false)),
        "hide" => Action::LeaveCallSurface,
        "return" => Action::ReturnToCall,
        _ => return,
    };
    engine.send(Input::Action(action));
}

/// The contact editor's Save: the name goes through WhatsApp contact sync.
#[tauri::command]
fn save_contact(engine: State<'_, Engine>, id: String, first: String, last: String) {
    engine.send(Input::Action(Action::SaveContact { id, first, last }));
}

/// Settings › Files: the page opened, so measure the media cache again,
/// as egui does whenever Settings opens.
#[tauri::command]
fn measure_storage(engine: State<'_, Engine>) {
    engine.send(Input::Edit(Box::new(|app| {
        app.storage_token += 1;
        let token = app.storage_token;
        app.backend
            .send(zapfast::backend::Command::LoadStorageStats { token });
    })));
}

/// Settings › Files' buttons: open the archive's or the downloads' folder,
/// open the log, pick or reset the downloads folder, clear the media cache.
#[tauri::command]
fn files_action(engine: State<'_, Engine>, action: String) {
    engine.send(Input::Edit(Box::new(move |app| {
        let push = match action.as_str() {
            "archive" => Action::OpenFolder(app.account().dirs.state.clone()),
            "downloads" => {
                let dir = app
                    .settings
                    .download_folder
                    .clone()
                    .unwrap_or_else(|| app.account().dirs.media_cache_dir());
                let _ = std::fs::create_dir_all(&dir);
                Action::OpenFolder(dir)
            }
            "log" => Action::OpenLog(app.dirs.log_file()),
            "pick" => Action::PickDownloadFolder,
            "default" => Action::SetDownloadFolder(None),
            "clear" => Action::ClearMediaCache(app.account().id.clone()),
            _ => return,
        };
        app.actions.push(push);
    })));
}

/// The call screen's device pickers; `None` is the system default.
#[tauri::command]
fn call_device(engine: State<'_, Engine>, kind: String, id: Option<String>) {
    let action = match kind.as_str() {
        "microphone" => Action::SetCallMicrophone(id),
        "speaker" => Action::SetCallSpeaker(id),
        "camera" => Action::SetCallCameraDevice(id),
        _ => return,
    };
    engine.send(Input::Action(action));
}

/// The call screen's Full screen corner button: the window goes full
/// screen, or leaves it, as egui's ToggleCallFullscreen does.
#[tauri::command]
fn call_fullscreen(window: tauri::WebviewWindow) -> bool {
    let on = !window.is_fullscreen().unwrap_or(false);
    let _ = window.set_fullscreen(on);
    on
}

/// The Calls page's chip: all or missed.
#[tauri::command]
fn set_call_filter(engine: State<'_, Engine>, filter: String) {
    let filter = if filter == "missed" {
        zapfast::model::CallFilter::Missed
    } else {
        zapfast::model::CallFilter::All
    };
    engine.send(Input::Action(Action::SetCallFilter(filter)));
}

/// The Calls page's "Search name or number" field.
#[tauri::command]
fn search_calls(engine: State<'_, Engine>, text: String) {
    engine.send(Input::Edit(Box::new(move |app| app.calls_search = text)));
}

/// The locked-chats prompt's answer: a new code the first time, else the
/// code that opens the folder.
#[tauri::command]
fn unlock_locked(engine: State<'_, Engine>, code: String) {
    let code = code.trim().to_owned();
    if code.is_empty() {
        return;
    }
    engine.send(Input::Edit(Box::new(move |app| {
        let setup = app.settings.chat_lock_code_hash.is_none();
        app.actions.push(if setup {
            Action::CreateChatLockCode(code)
        } else {
            Action::UnlockLockedFolder(code)
        });
    })));
}

#[tauri::command]
fn close_dialog(engine: State<'_, Engine>) {
    engine.send(Input::Action(Action::CloseDialog));
}

#[tauri::command]
fn fetch_blocklist(engine: State<'_, Engine>) {
    engine.send(Input::Action(Action::FetchBlocklist));
}

#[tauri::command]
fn set_blocked(engine: State<'_, Engine>, chat: String, blocked: bool) {
    engine.send(Input::Action(Action::SetBlocked { chat, blocked }));
}

/// The wallpaper colour for the light or the dark theme, by its stored
/// name. Set directly: the core's action picks the theme from a palette the
/// headless core does not keep in step with the page.
#[tauri::command]
fn set_wallpaper_color(engine: State<'_, Engine>, color: String, dark: bool) {
    let Ok(color) = serde_json::from_value::<zapfast::settings::WallpaperColor>(
        serde_json::Value::String(color),
    ) else {
        return;
    };
    engine.send(Input::Edit(Box::new(move |app| {
        if dark {
            app.settings.dark_wallpaper_color = color;
        } else {
            app.settings.wallpaper_color = color;
        }
        app.actions.push(Action::SettingsChanged);
    })));
}

/// Settings › Appearance: a theme file from the themes folder, opening
/// that folder, reading it again, and the chat background image.
#[tauri::command]
fn set_custom_theme(engine: State<'_, Engine>, file: String) {
    engine.send(Input::Action(Action::SetCustomTheme(file)));
}

#[tauri::command]
fn themes_folder(engine: State<'_, Engine>, reload: bool) {
    engine.send(Input::Action(if reload {
        Action::ReloadThemes
    } else {
        Action::OpenThemesFolder
    }));
}

#[tauri::command]
fn wallpaper_image(engine: State<'_, Engine>, pick: bool) {
    engine.send(Input::Action(if pick {
        Action::PickWallpaperImage
    } else {
        Action::RemoveWallpaperImage
    }));
}

/// The page's English text in the interface language (REQ-WEBUI-015).
#[tauri::command]
fn translate(engine: State<'_, Engine>, texts: Vec<String>) -> Vec<String> {
    engine.translate(&texts)
}

/// Settings › Appearance › Language: a locale code, or none to follow the
/// system.
#[tauri::command]
fn set_language(engine: State<'_, Engine>, code: Option<String>) {
    let choice = match code {
        None => None,
        Some(code) => match serde_json::from_value(serde_json::Value::String(code)) {
            Ok(locale) => Some(locale),
            Err(_) => return,
        },
    };
    engine.send(Input::Action(Action::SetInterfaceLanguage(choice)));
}

/// The person used the page: keyboard or pointer. Restarts the app
/// lock's idle timer, which egui's input used to restart.
#[tauri::command]
fn activity(engine: State<'_, Engine>) {
    engine.send(Input::Activity);
}

/// Unlinks this computer; the page asks first.
#[tauri::command]
fn unlink(engine: State<'_, Engine>) {
    engine.send(Input::Action(Action::Unlink));
}

fn start(arguments: &[std::ffi::OsString]) -> Start {
    let mut args = arguments
        .iter()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned());
    let mut start = Start {
        demo: false,
        demo_page: None,
        instance: None,
        receipt: None,
        update_error: None,
        hidden: false,
        target: None,
        invite: None,
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--demo" => start.demo = true,
            "--demo-page" => {
                start.demo = true;
                start.demo_page = args.next();
            }
            // A number, a `wa.me` link, the `whatsapp://` URI the desktop
            // entry passes, or a group invite link.
            argument if !argument.starts_with('-') => {
                start.target = zapfast::target::parse(argument);
                start.invite = zapfast::safety::group_invite_code(argument);
            }
            _ => {}
        }
    }
    start
}

/// `superwhatsapp quit` and `superwhatsapp reload-themes` only talk to the
/// running copy, as in the egui build: they never start SuperWhatsapp.
/// Returns the exit code when the command line was one of them.
fn control(arguments: &[std::ffi::OsString]) -> Option<i32> {
    use zapfast::single_instance::{Sent, send};
    let verb = match arguments.get(1)?.to_str()? {
        "quit" => "quit",
        "reload-themes" => "reload-themes",
        _ => return None,
    };
    let dirs = zapfast::paths::AppDirs::discover();
    let (code, message) = match (verb, send(&dirs.runtime, verb)) {
        (_, Sent::Taken) | ("quit", Sent::NotRunning) => (0, None),
        (_, Sent::NotRunning) => (
            1,
            Some("SuperWhatsapp is not running, so there are no themes to reload.".to_owned()),
        ),
        ("quit", Sent::Older) => (
            1,
            Some("The running SuperWhatsapp is too old to quit from the command line; use Quit in its tray menu.".to_owned()),
        ),
        (_, Sent::Older) => (
            1,
            Some("The running SuperWhatsapp is too old to reload themes from the command line.".to_owned()),
        ),
        (_, Sent::Failed(error)) => (1, Some(format!("Could not reach the running SuperWhatsapp: {error}"))),
    };
    if let Some(message) = message {
        eprintln!("{message}");
    }
    Some(code)
}

pub fn run() {
    // First, before anything else: run the update helper when asked
    // (`--apply-update <job>`, then exit), and take the update flags off
    // the command line, as the egui build does.
    let launch = fastframe_update::intercept(&zapfast::updates::CONFIG);
    // The updater checks a download with `--version`: `<slug> <version>`.
    if launch
        .arguments
        .iter()
        .skip(1)
        .any(|arg| arg == "--version" || arg == "-V")
    {
        println!(
            "{} {}",
            zapfast::updates::CONFIG.slug,
            env!("CARGO_PKG_VERSION")
        );
        return;
    }
    // `--dump-state <page>`: print the demo's state as the page receives it
    // and exit, for screenshots and front-end tests (REQ-WEBUI-018).
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() == Some("--dump-state") {
        let state = engine::dump(args.next());
        let mut value = serde_json::to_value(&state).expect("state");
        media::inline(&mut value);
        println!("{}", serde_json::to_string_pretty(&value).expect("state"));
        return;
    }
    if let Some(code) = control(&launch.arguments) {
        std::process::exit(code);
    }
    let mut start = start(&launch.arguments);
    if let Some(argument) = launch.arguments.get(1).and_then(|arg| arg.to_str())
        && !argument.starts_with('-')
        && start.target.is_none()
        && start.invite.is_none()
    {
        eprintln!("SuperWhatsapp cannot read a phone number or group invite link from {argument}");
        std::process::exit(2);
    }
    // One SuperWhatsapp per account: a copy already running, this interface or
    // the egui one, comes forward instead (REQ-WEBUI-012.A2).
    // A relaunch after an update shows its window, so the page loads and
    // confirms the update.
    let start_hidden = launch.receipt.is_none()
        && launch
            .arguments
            .iter()
            .any(|arg| arg == desktop::START_HIDDEN);
    start.hidden = start_hidden;
    if !start.demo {
        match engine::claim(&start) {
            Some(instance) => start.instance = Some(instance),
            None => return,
        }
    }
    start.receipt = launch.receipt;
    start.update_error = launch.error;
    tauri::Builder::default()
        // A second start brings the first window forward (REQ-WEBUI-012.A2).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            desktop::show(app);
        }))
        .register_uri_scheme_protocol("zapmedia", media::serve)
        .register_uri_scheme_protocol("zapframe", frames::serve)
        .setup(move |app| {
            let engine = engine::spawn(app.handle().clone(), start);
            app.manage(engine);
            desktop::setup(app, start_hidden)?;
            Ok(())
        })
        .on_window_event(desktop::on_window_event)
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
            info_page,
            show_new_call,
            search_new_call,
            retry_info_media,
            jumped,
            leave_group,
            set_star,
            go_to_message,
            set_setting,
            unlink,
            activity,
            unlock,
            lock_now,
            forgot_password,
            app_lock_form,
            set_chat_lock_code,
            unlock_locked,
            set_call_filter,
            open_url,
            start_call,
            call_control,
            call_device,
            measure_storage,
            files_action,
            save_contact,
            call_fullscreen,
            search_calls,
            close_dialog,
            fetch_blocklist,
            set_blocked,
            set_wallpaper_color,
            set_custom_theme,
            themes_folder,
            wallpaper_image,
            translate,
            set_language,
            znga::znga_send,
            znga::znga_stop,
            znga::znga_card,
            znga::znga_tool,
            znga::znga_offer_done,
            znga::znga_key,
            znga::znga_test,
            znga::znga_chat_list,
            znga::znga_fact,
            znga::znga_promise,
            znga::znga_profile,
            znga::znga_backfill,
            znga::znga_set
        ])
        .build(tauri::generate_context!())
        .expect("SuperWhatsapp could not start")
        .run(|app, event| match event {
            // ⌘Q and the app menu's Quit save through the core first; the
            // core's own exit carries a code and goes through.
            tauri::RunEvent::ExitRequested {
                api, code: None, ..
            } => {
                api.prevent_exit();
                desktop::quit(app);
            }
            // Clicking the Dock icon with the window hidden shows it.
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => desktop::show(app),
            _ => {}
        });
}
