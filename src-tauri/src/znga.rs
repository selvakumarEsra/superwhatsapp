//! Znga in the Tauri interface (specs/znga-chat.md): the Znga chat's row
//! and messages, the composer's offer, Settings › Znga, and the commands
//! the page calls for them. The logic lives in `zapfast::znga::chat`.

use serde::Serialize;
use tauri::State;
use zapfast::app::App;
use zapfast::znga::config::{Engine, chat_key};
use zapfast::znga::thread::{Card, ZNGA_CHAT};
use zapfast::znga::{chat, keys};

use crate::engine::{Engine as Core, Input};
use crate::view::ChatRow;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZngaChat {
    pub entries: Vec<ZEntry>,
    /// The answer being written: `Some("")` while Znga is thinking.
    pub writing: Option<String>,
    /// A question waits for the person's pick.
    pub asking: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZEntry {
    pub id: u64,
    pub at: i64,
    pub from_me: bool,
    pub text: String,
    pub unasked: bool,
    pub cards: Vec<ZCard>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZCard {
    /// needsYou, briefing, draft, catchUp, plan, voice, promise or drift.
    pub kind: &'static str,
    pub title: String,
    pub body: String,
    pub meta: String,
    pub actions: Vec<ZAction>,
    /// What happened once acted on; the buttons are then disabled.
    pub outcome: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ZAction {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferView {
    pub id: u64,
    pub chat: String,
    pub text: String,
    pub rewrite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZFact {
    pub chat: String,
    pub chat_title: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZPromise {
    pub chat: String,
    pub chat_title: String,
    pub text: String,
    pub due: Option<i64>,
    pub owes_me: bool,
}

/// One Sentinel decision: never message text (REQ-ZCHAT-007.A3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZAudit {
    pub at: i64,
    pub action: String,
    pub origin: &'static str,
    pub allowed: bool,
    pub reason: String,
}

/// Settings › Znga (REQ-ZCHAT-007).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZngaSettings {
    pub enabled: bool,
    pub engine: &'static str,
    /// (id, label) of every engine.
    pub engines: Vec<(&'static str, &'static str)>,
    pub model: String,
    pub endpoint: String,
    pub default_model: &'static str,
    pub default_endpoint: &'static str,
    pub needs_key: bool,
    /// Where the key comes from, when there is one.
    pub key_source: Option<String>,
    pub cloud: bool,
    pub cloud_consent: bool,
    pub alerts: bool,
    pub briefings: bool,
    pub briefing_min_messages: usize,
    pub briefing_interval_minutes: u64,
    pub digests: bool,
    pub digest_morning: String,
    pub digest_evening: String,
    pub reminders: bool,
    pub quiet_hours: bool,
    pub quiet_from: String,
    pub quiet_to: String,
    pub self_notes: bool,
    pub self_chat: bool,
    pub include_groups: bool,
    pub promises: bool,
    pub memory: bool,
    pub checkin_days: u32,
    pub your_name: String,
    pub voice_notes: String,
    pub tone: String,
    pub voice_endpoint: String,
    pub voice_model: String,
    /// (chat id, title).
    pub twin: Vec<(String, String)>,
    pub excluded: Vec<(String, String)>,
    pub facts: Vec<ZFact>,
    pub open_promises: Vec<ZPromise>,
    pub audit: Vec<ZAudit>,
    /// The result of Test connection, or the last setup problem.
    pub status: Option<(String, bool)>,
}

fn engine_id(engine: Engine) -> &'static str {
    match engine {
        Engine::Local => "local",
        Engine::Claude => "claude",
        Engine::Gemini => "gemini",
        Engine::OpenAi => "openai",
        Engine::ClaudeCode => "claude-code",
    }
}

fn engine_of(id: &str) -> Option<Engine> {
    Engine::ALL.into_iter().find(|e| engine_id(*e) == id)
}

fn card(card: &Card) -> ZCard {
    let (kind, title, body, meta) = card.view();
    ZCard {
        kind,
        title,
        body,
        meta,
        actions: card
            .actions()
            .iter()
            .map(|(id, label)| ZAction { id, label })
            .collect(),
        outcome: None,
    }
}

/// The Znga row at the top of the chat list, with Znga on
/// (REQ-ZCHAT-001.A1).
pub fn row(app: &App) -> Option<ChatRow> {
    if !chat::enabled(app) {
        return None;
    }
    let account = app.account().id.as_str();
    let thread = app.znga.threads.get(account);
    let last = thread.and_then(|t| t.entries().last());
    Some(ChatRow {
        id: ZNGA_CHAT.to_owned(),
        title: "Znga".into(),
        avatar: None,
        group: false,
        time: last.map_or(0, |e| e.at),
        unread: thread.map_or(0, |t| t.unread),
        marked_unread: false,
        muted: false,
        pinned: false,
        favorite: false,
        favorite_position: 0,
        last: Some(crate::view::Last {
            from_me: last.is_some_and(|e| e.from_person),
            sender: None,
            text: last
                .map(|e| e.text.lines().next().unwrap_or("").to_owned())
                .unwrap_or_else(|| "Ask me anything about your chats.".into()),
            status: "none",
        }),
    })
}

/// The Znga chat's messages while it is the open conversation.
pub fn chat_view(app: &mut App) -> Option<ZngaChat> {
    if app.open_chat.is_some() {
        app.znga.chat_open = false;
    }
    if !chat::enabled(app) || !app.znga.chat_open {
        return None;
    }
    let writing = chat::writing(app);
    let account = app.account().id.as_str().to_owned();
    let thread = app.znga.threads.entry(account).or_default();
    let entries = thread
        .entries()
        .iter()
        .map(|entry| ZEntry {
            id: entry.id,
            at: entry.at,
            from_me: entry.from_person,
            text: entry.text.clone(),
            unasked: entry.unasked,
            cards: entry
                .items
                .iter()
                .map(|item| ZCard {
                    outcome: item.outcome.clone(),
                    ..card(&item.card)
                })
                .collect(),
        })
        .collect();
    Some(ZngaChat {
        entries,
        writing,
        asking: thread.question.is_some(),
    })
}

pub fn offer(app: &App) -> Option<OfferView> {
    app.znga.offer.as_ref().map(|offer| OfferView {
        id: offer.id,
        chat: offer.chat.clone(),
        text: offer.text.clone(),
        rewrite: offer.rewrite,
    })
}

pub fn settings(app: &App) -> ZngaSettings {
    let config = &app.znga.config;
    let account = app.account().id.as_str().to_owned();
    let titled = |keys: &std::collections::BTreeSet<String>| -> Vec<(String, String)> {
        keys.iter()
            .filter_map(|key| key.strip_prefix(&format!("{account}/")))
            .map(|chat| (chat.to_owned(), app.chat_title_of(chat)))
            .collect()
    };
    let persona = config.personas.get(&account).cloned().unwrap_or_default();
    ZngaSettings {
        enabled: config.enabled,
        engine: engine_id(config.engine),
        engines: Engine::ALL
            .iter()
            .map(|e| (engine_id(*e), e.label()))
            .collect(),
        model: config.model.clone(),
        endpoint: config.endpoint.clone(),
        default_model: config.engine.default_model(),
        default_endpoint: config.engine.default_endpoint(),
        needs_key: config.engine.needs_key(),
        key_source: config
            .engine
            .needs_key()
            .then(|| keys::source(config.engine))
            .flatten(),
        cloud: config.is_cloud(),
        cloud_consent: config.cloud_consent,
        alerts: config.alerts,
        briefings: config.briefings,
        briefing_min_messages: config.briefing_min_messages,
        briefing_interval_minutes: config.briefing_interval_minutes,
        digests: config.digests,
        digest_morning: config.digest_morning.clone(),
        digest_evening: config.digest_evening.clone(),
        reminders: config.reminders,
        quiet_hours: config.quiet_hours,
        quiet_from: config.quiet_from.clone(),
        quiet_to: config.quiet_to.clone(),
        self_notes: config.self_notes,
        self_chat: config.self_chat,
        include_groups: config.include_groups,
        promises: config.promises,
        memory: config.memory,
        checkin_days: config.checkin_days,
        your_name: if persona.your_name.is_empty() {
            config.your_name.clone()
        } else {
            persona.your_name.clone()
        },
        voice_notes: if persona.voice_notes.is_empty() {
            config.voice_notes.clone()
        } else {
            persona.voice_notes.clone()
        },
        tone: persona.tone,
        voice_endpoint: config.voice_endpoint.clone(),
        voice_model: config.voice_model.clone(),
        twin: titled(&config.twin_chats),
        excluded: titled(&config.excluded_chats),
        facts: app
            .znga
            .people
            .for_account(&account)
            .into_iter()
            .map(|fact| ZFact {
                chat: fact.chat.clone(),
                chat_title: fact.chat_title.clone(),
                text: fact.text.clone(),
            })
            .collect(),
        open_promises: app
            .znga
            .promises
            .open()
            .into_iter()
            .filter(|(_, p)| p.account == account)
            .map(|(_, p)| ZPromise {
                chat: p.chat.clone(),
                chat_title: p.chat_title.clone(),
                text: p.text.clone(),
                due: p.due,
                owes_me: p.owes == zapfast::znga::promises::Owes::Me,
            })
            .collect(),
        audit: app
            .znga
            .audit
            .entries()
            .rev()
            .take(100)
            .map(|e| ZAudit {
                at: e.at,
                action: e.action.clone(),
                origin: e.origin.label(),
                allowed: e.allowed,
                reason: e.reason.clone(),
            })
            .collect(),
        status: app.znga.status.as_ref().map(|s| (s.text.clone(), s.error)),
    }
}

// ---------------------------------------------------------------------
// Commands.

/// Opens the Znga chat, closing any other (REQ-ZCHAT-001.A6).
pub fn open(app: &mut App) {
    if app.open_chat.is_some() {
        app.actions.push(zapfast::model::Action::CloseChat);
        app.open_chat = None;
    }
    app.znga.chat_open = true;
    let account = app.account().id.as_str().to_owned();
    app.znga.threads.entry(account).or_default().mark_read();
}

#[tauri::command]
pub fn znga_send(engine: State<'_, Core>, text: String) {
    engine.send(Input::Edit(Box::new(move |app| {
        chat::send_message(app, &text)
    })));
}

#[tauri::command]
pub fn znga_stop(engine: State<'_, Core>) {
    engine.send(Input::Edit(Box::new(chat::stop)));
}

#[tauri::command]
pub fn znga_card(engine: State<'_, Core>, entry: u64, index: usize, action: String) {
    engine.send(Input::Edit(Box::new(move |app| {
        chat::card_action(app, entry, index, &action)
    })));
}

#[tauri::command]
pub fn znga_tool(
    engine: State<'_, Core>,
    chat: String,
    tool: String,
    text: String,
    language: String,
) {
    engine.send(Input::Edit(Box::new(move |app| {
        chat::tool(app, &chat, &tool, &text, &language)
    })));
}

/// The page took or dismissed the composer's offer.
#[tauri::command]
pub fn znga_offer_done(engine: State<'_, Core>, id: u64) {
    engine.send(Input::Edit(Box::new(move |app| {
        if app.znga.offer.as_ref().is_some_and(|o| o.id == id) {
            app.znga.offer = None;
        }
    })));
}

/// Saves or forgets the engine's API key; it goes to the system keychain
/// and is never logged.
#[tauri::command]
pub fn znga_key(engine: State<'_, Core>, key: Option<String>) {
    engine.send(Input::Edit(Box::new(move |app| {
        let which = app.znga.config.engine;
        match key {
            Some(key) => match keys::store(which, key.trim()) {
                Ok(()) => app
                    .znga
                    .set_status("Key saved in the system keychain.", false),
                Err(error) => app.znga.set_status(error, true),
            },
            None => keys::forget(which),
        }
        app.znga.refresh_key();
    })));
}

#[tauri::command]
pub fn znga_test(engine: State<'_, Core>) {
    engine.send(Input::Edit(Box::new(|app| {
        use zapfast::znga::provider::{Effort, Prompt, Turn};
        let prompt = Prompt {
            system: "Reply with the single word: ready".into(),
            turns: vec![Turn::user("Are you there?")],
            max_tokens: 1000,
            effort: Effort::Low,
        };
        match app.znga.submit(prompt, zapfast::znga::Purpose::Test) {
            Ok(()) => app.znga.set_status("Testing…", false),
            Err(error) => app.znga.set_status(error, true),
        }
    })));
}

/// Twin and excluded chats: `list` is twin or excluded.
#[tauri::command]
pub fn znga_chat_list(engine: State<'_, Core>, list: String, chat: String, on: bool) {
    engine.send(Input::Edit(Box::new(move |app| {
        let key = chat_key(app.account().id.as_str(), &chat);
        let config = &mut app.znga.config;
        let (this, other) = if list == "twin" {
            (&mut config.twin_chats, &mut config.excluded_chats)
        } else {
            (&mut config.excluded_chats, &mut config.twin_chats)
        };
        if on {
            this.insert(key.clone());
            other.remove(&key);
        } else {
            this.remove(&key);
        }
        app.znga.mark_dirty();
    })));
}

/// Adds a fact about a chat, or forgets one (REQ-ZCHAT-007.A2).
#[tauri::command]
pub fn znga_fact(engine: State<'_, Core>, chat: String, text: String, forget: bool) {
    engine.send(Input::Edit(Box::new(move |app| {
        let account = app.account().id.as_str().to_owned();
        if forget {
            app.znga.people.forget(&account, &chat, &text);
        } else if !text.trim().is_empty() {
            let title = app.chat_title_of(&chat);
            let now = zapfast::util::now();
            app.znga
                .people
                .add(&account, &chat, &title, &[text.trim().to_owned()], now);
        }
    })));
}

/// Marks an open promise done or not a promise.
#[tauri::command]
pub fn znga_promise(engine: State<'_, Core>, chat: String, text: String, done: bool) {
    engine.send(Input::Edit(Box::new(move |app| {
        use zapfast::znga::promises::State as PromiseState;
        let account = app.account().id.as_str().to_owned();
        let index = app
            .znga
            .promises
            .all()
            .iter()
            .position(|p| p.account == account && p.chat == chat && p.text == text);
        if let Some(index) = index {
            let state = if done {
                PromiseState::Done
            } else {
                PromiseState::Dismissed
            };
            app.znga.promises.set_state(index, state);
        }
    })));
}

/// Changes one Znga setting; it takes effect at once (REQ-ZCHAT-007.A1).
#[tauri::command]
pub fn znga_set(engine: State<'_, Core>, key: String, value: serde_json::Value) {
    engine.send(Input::Edit(Box::new(move |app| set(app, &key, &value))));
}

pub fn set(app: &mut App, key: &str, value: &serde_json::Value) {
    let flag = value.as_bool();
    let text = value.as_str().map(|t| t.trim().to_owned());
    let number = value.as_u64();
    let account = app.account().id.as_str().to_owned();
    let config = &mut app.znga.config;
    let hhmm = |t: &Option<String>| {
        t.clone()
            .filter(|t| zapfast::znga::thread::parse_hhmm(t).is_some())
    };
    match (key, flag) {
        ("enabled", Some(on)) => config.enabled = on,
        ("cloudConsent", Some(on)) => config.cloud_consent = on,
        ("alerts", Some(on)) => config.alerts = on,
        ("briefings", Some(on)) => config.briefings = on,
        ("digests", Some(on)) => config.digests = on,
        ("reminders", Some(on)) => config.reminders = on,
        ("quietHours", Some(on)) => config.quiet_hours = on,
        ("selfNotes", Some(on)) => config.self_notes = on,
        ("includeGroups", Some(on)) => config.include_groups = on,
        ("promises", Some(on)) => config.promises = on,
        ("memory", Some(on)) => config.memory = on,
        ("selfChat", Some(on)) => {
            if on && !config.self_chat {
                config.self_chat_since = zapfast::util::now();
            }
            config.self_chat = on;
        }
        ("engine", _) => {
            let Some(engine) = text.as_deref().and_then(engine_of) else {
                return;
            };
            if engine != config.engine {
                config.engine = engine;
                config.model.clear();
                config.endpoint.clear();
                config.cloud_consent = false;
                app.znga.refresh_key();
            }
        }
        ("model", _) => config.model = text.unwrap_or_default(),
        ("endpoint", _) => config.endpoint = text.unwrap_or_default(),
        ("voiceEndpoint", _) => config.voice_endpoint = text.unwrap_or_default(),
        ("voiceModel", _) => config.voice_model = text.unwrap_or_default(),
        ("digestMorning", _) => match hhmm(&text) {
            Some(t) => config.digest_morning = t,
            None => return,
        },
        ("digestEvening", _) => match hhmm(&text) {
            Some(t) => config.digest_evening = t,
            None => return,
        },
        ("quietFrom", _) => match hhmm(&text) {
            Some(t) => config.quiet_from = t,
            None => return,
        },
        ("quietTo", _) => match hhmm(&text) {
            Some(t) => config.quiet_to = t,
            None => return,
        },
        ("briefingMinMessages", _) => match number {
            Some(n) => config.briefing_min_messages = n.clamp(1, 50) as usize,
            None => return,
        },
        ("briefingIntervalMinutes", _) => match number {
            Some(n) => config.briefing_interval_minutes = n.clamp(5, 240),
            None => return,
        },
        ("checkinDays", _) => match number {
            Some(n) => config.checkin_days = n.clamp(1, 90) as u32,
            None => return,
        },
        ("yourName" | "voiceNotes" | "tone", _) => {
            let persona = config.personas.entry(account).or_default();
            let field = match key {
                "yourName" => &mut persona.your_name,
                "voiceNotes" => &mut persona.voice_notes,
                _ => &mut persona.tone,
            };
            *field = value.as_str().unwrap_or_default().to_owned();
        }
        _ => {
            log::warn!("unknown Znga setting {key}");
            return;
        }
    }
    app.znga.mark_dirty();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_engine_has_an_id_the_page_can_send_back() {
        for engine in Engine::ALL {
            assert_eq!(engine_of(engine_id(engine)), Some(engine));
        }
    }
}
