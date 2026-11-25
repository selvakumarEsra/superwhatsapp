//! The bridge between Znga and the app. Upstream files call these from
//! lines marked `znga hook`; everything else about Znga stays in this
//! folder.

use crate::app::App;
use crate::model::{Action, Content, Message};

use std::collections::HashMap;
use std::path::PathBuf;

use super::archive_ask::{AskOrigin, Asking, Visible};
use super::config::chat_key;
use super::drafts::{Draft, Kind};
use super::privacy::{ChatFacts, visible};
use super::proactive::{self, Context, Seen};
use super::prompt::{self, Line, Twin};
use super::provider::Turn;
use super::selfchat::{self, Allowance, Question};
use super::sentinel::{Action as Act, Origin};
use super::store::{Job, Reply};
use super::{Effect, Purpose, VoiceNote, VoiceState};

/// How often our own sent messages are re-read for the twin's style.
const STYLE_SCAN: std::time::Duration = std::time::Duration::from_secs(60);
const CHECKIN_SCAN: std::time::Duration = std::time::Duration::from_secs(10 * 60);
/// How often twin chats are mined for promises and facts.
const EXTRACT_SCAN: std::time::Duration = std::time::Duration::from_secs(15 * 60);
const PROMISE_CHECK: std::time::Duration = std::time::Duration::from_secs(60);
const RHYTHM_SCAN: std::time::Duration = std::time::Duration::from_secs(6 * 3600);

fn away(app: &App) -> bool {
    app.window_hidden || !app.window_focused
}

fn user_part(id: &str) -> &str {
    id.split(['@', ':']).next().unwrap_or(id)
}

/// Called for every live incoming message (`Event::Incoming`), on the
/// account it arrived on (the app makes that account active meanwhile).
pub fn observe_incoming(app: &mut App, chat_id: &str, message: &Message) {
    if !app.znga.config.enabled || message.from_me {
        return;
    }
    let account = app.account().id.as_str().to_owned();
    let now = crate::util::now();
    let Some(chat) = app.chat(chat_id) else {
        return;
    };
    if !visible(&app.znga.config, ChatFacts::of(&account, chat)) {
        return;
    }
    let muted = chat.muted(now);
    let group = chat.is_group();
    let chat_title = app.chat_title(chat);
    let sender_name = app.display_name_or(&message.sender, message.sender_name.as_deref());
    let text = app.message_text(message);
    if let Content::Audio {
        voice_note: true,
        seconds,
        ..
    } = &message.content
    {
        app.znga.add_voice_note(VoiceNote {
            id: 0,
            account: account.clone(),
            chat: chat_id.to_owned(),
            chat_title: chat_title.clone(),
            sender_name: sender_name.clone(),
            message: message.id.clone(),
            seconds: *seconds,
            at: message.timestamp,
            state: VoiceState::New,
        });
    }
    let mentions_me = {
        let me: Vec<&str> = [app.account().me.as_deref(), app.account().me_lid.as_deref()]
            .into_iter()
            .flatten()
            .map(user_part)
            .collect();
        message
            .mentions
            .iter()
            .any(|mention| me.contains(&user_part(&mention.id)))
    };
    let seen = Seen {
        account: account.clone(),
        chat: chat_id.to_owned(),
        chat_title: chat_title.clone(),
        group,
        sender: message.sender.clone(),
        sender_name: sender_name.clone(),
        text: text.clone(),
        at: message.timestamp,
        score: Default::default(),
    };
    let alert = app
        .znga
        .activity
        .observe(seen, Context { mentions_me, muted });
    if let Some(alert) = alert
        && app.znga.config.alerts
    {
        let reason = alert.reason.clone();
        app.znga.push_alert(alert);
        let reading = !away(app) && app.open_chat.as_deref() == Some(chat_id);
        if !reading && app.znga.audit.check(&Act::Notify, Origin::OthersText, now) {
            notify(
                app,
                format!("Znga · {chat_title} needs you"),
                format!("{sender_name}: {} ({reason})", clip(&text, 140)),
                &account,
                chat_id,
                Some(message.id.clone()),
            );
        }
    }
    if app.znga.config.is_twin(&account, chat_id) && !muted {
        request_draft(app, &account, chat_id, Kind::AutoReply);
    }
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_owned()
    } else {
        text.chars().take(max).collect::<String>() + "…"
    }
}

/// A Znga notification, unless notifications are off. Behind the app lock
/// it says nothing about the content.
fn notify(
    app: &mut App,
    title: String,
    body: String,
    account: &str,
    chat: &str,
    message: Option<String>,
) {
    if !app.settings.notifications {
        return;
    }
    let (title, body) = if app.app_lock.is_locked() {
        (
            "Znga".to_owned(),
            "Open SuperWhatsapp to see it.".to_owned(),
        )
    } else {
        (title, body)
    };
    let account = app
        .accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .unwrap_or_else(|| app.account())
        .id
        .clone();
    let waker = app.waker.clone();
    let sound = app.settings.mention_sound.clone();
    let opens = std::sync::Arc::clone(&app.notification_opens);
    app.notifications.show(
        title,
        body,
        None,
        sound,
        crate::notify::NotificationTarget {
            account,
            chat: chat.to_owned(),
            message,
        },
        opens,
        move || waker.wake(),
        false,
    );
}

/// The chat's recent lines: the loaded conversation when there is one,
/// else what Znga has seen.
fn lines_of(app: &App, account: &str, chat: &str) -> Vec<Line> {
    lines_n(app, account, chat, 30)
}

fn lines_n(app: &App, account: &str, chat: &str, n: usize) -> Vec<Line> {
    let index = app
        .accounts
        .iter()
        .position(|known| known.id.as_str() == account);
    if let Some(index) = index
        && let Some(conversation) = app.accounts[index].conversations.get(chat)
        && !conversation.messages.is_empty()
    {
        let skip = conversation.messages.len().saturating_sub(n);
        return conversation.messages[skip..]
            .iter()
            .filter(|message| !matches!(message.content, Content::Call { .. }))
            .map(|message| Line {
                speaker: app.display_name_or(&message.sender, message.sender_name.as_deref()),
                text: app.message_text(message),
                from_me: message.from_me,
            })
            .collect();
    }
    app.znga
        .activity
        .recent_in(account, chat, n)
        .into_iter()
        .map(|seen| Line {
            speaker: seen.sender_name,
            text: seen.text,
            from_me: false,
        })
        .collect()
}

fn your_name(app: &App, account: &str) -> String {
    let configured = app.znga.config.persona(account).your_name;
    if !configured.is_empty() {
        return configured;
    }
    app.accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .and_then(|known| known.me_name.clone())
        .unwrap_or_default()
}

fn title_of(app: &App, account: &str, chat: &str) -> String {
    app.accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .and_then(|known| known.chats.iter().find(|c| c.id == chat))
        .map(|found| app.chat_title(found))
        .unwrap_or_else(|| chat.to_owned())
}

/// Asks the twin for a reply or check-in for `chat`. An automatic request
/// is skipped while one for the same chat is still running.
pub fn request_draft(app: &mut App, account: &str, chat: &str, kind: Kind) {
    request_draft_with(app, account, chat, kind, None);
}

/// The twin as it writes on `account` to `chat`: style, persona, examples
/// and what Znga remembers about the person (REQ-ZNGA-019, REQ-ZNGA-025).
fn twin_for(app: &App, account: &str, chat: &str) -> Twin {
    let persona = app.znga.config.persona(account);
    Twin {
        name: your_name(app, account),
        style: app.znga.style_for(account).describe(),
        voice_notes: persona.voice_notes,
        tone: persona.tone,
        examples: app.znga.examples.pick(&chat_key(account, chat), 12),
        facts: if chat.is_empty() {
            Vec::new()
        } else {
            app.znga.people.about(account, chat)
        },
    }
}

fn request_draft_with(app: &mut App, account: &str, chat: &str, kind: Kind, coach: Option<String>) {
    let same = |purpose: &Purpose| {
        matches!(purpose, Purpose::Draft { account: a, chat: c, kind: k, .. }
            if a == account && c == chat && *k == kind)
    };
    if app.znga.waiting_for(same) {
        return;
    }
    let lines = lines_of(app, account, chat);
    let chat_title = title_of(app, account, chat);
    let twin = twin_for(app, account, chat);
    let prompt = match kind {
        Kind::CheckIn => prompt::checkin(
            &twin,
            &chat_title,
            app.znga.config.checkin_days,
            &lines,
            coach.as_deref(),
        ),
        _ => prompt::twin_reply(&twin, &chat_title, &lines),
    };
    let purpose = Purpose::Draft {
        kind,
        account: account.to_owned(),
        chat: chat.to_owned(),
        chat_title,
    };
    if let Err(error) = app.znga.submit(prompt, purpose)
        && (coach.is_some() || !matches!(kind, Kind::AutoReply | Kind::CheckIn))
    {
        app.znga.set_status(error, true);
    }
}

/// An opener for a drifting chat, with the coach's reading of why
/// (REQ-ZNGA-024).
pub fn request_opener(app: &mut App, account: &str, chat: &str) {
    let reason = app
        .znga
        .rhythms
        .iter()
        .find(|r| r.account == account && r.chat == chat)
        .map(|r| r.reason());
    request_draft_with(
        app,
        account,
        chat,
        Kind::CheckIn,
        Some(reason.unwrap_or_default()),
    );
    app.znga.set_status("Znga is drafting an opener…", false);
}

/// "Brief me": a briefing from everything since the last one, or, with
/// nothing new, from the most recent activity.
pub fn request_briefing(app: &mut App, proactive: bool) {
    let mut items = app.znga.activity.since_last_briefing();
    if items.is_empty() && !proactive {
        items = app.znga.activity.recent(60);
    }
    if proactive {
        // Accounts whose persona turned briefings off are left out
        // (REQ-ZNGA-025.A3).
        let config = &app.znga.config;
        items.retain(|seen| config.persona(&seen.account).briefings);
        if items.is_empty() {
            return;
        }
    }
    if items.is_empty() {
        app.znga.set_status(
            "Nothing to brief yet: Znga reports on messages that arrive while it is on.",
            false,
        );
        return;
    }
    let account = app.account().id.as_str().to_owned();
    let prompt = prompt::briefing(&your_name(app, &account), &items);
    match app.znga.submit(prompt, Purpose::Briefing { proactive }) {
        Ok(()) => {
            app.znga.activity.mark_briefed(crate::util::now());
            if !proactive {
                app.znga.set_status("Znga is writing your briefing…", false);
            }
        }
        Err(error) => app.znga.set_status(error, true),
    }
}

/// "Update my people": one instruction, a draft per named person.
pub fn request_fan_out(app: &mut App, instruction: &str) {
    let account = app.account().id.as_str().to_owned();
    let config = app.znga.config.clone();
    let mut chats: Vec<(String, String, i64)> = app
        .account()
        .chats
        .iter()
        .filter(|chat| visible(&config, ChatFacts::of(&account, chat)))
        .map(|chat| (chat.id.clone(), app.chat_title(chat), chat.last_activity))
        .collect();
    // Twin chats first, then the most recently active.
    chats.sort_by_key(|(id, _, last)| (!config.is_twin(&account, id), -last));
    chats.truncate(120);
    let names: Vec<String> = chats.iter().map(|(_, title, _)| title.clone()).collect();
    let pairs: Vec<(String, String)> = chats
        .into_iter()
        .map(|(id, title, _)| (id, title))
        .collect();
    let twin = twin_for(app, &account, "");
    let prompt = prompt::fan_out(&twin, instruction, &names);
    match app.znga.submit(
        prompt,
        Purpose::FanOut {
            account,
            chats: pairs,
        },
    ) {
        Ok(()) => app
            .znga
            .set_status("Znga is drafting your messages…", false),
        Err(error) => app.znga.set_status(error, true),
    }
}

/// "Ask Znga": plan a search, search the archive, then answer
/// (REQ-ZNGA-020).
pub fn request_ask(app: &mut App, question: String) {
    let question = question.trim().to_owned();
    if question.is_empty() {
        return;
    }
    let history = app.znga.ask.clone();
    app.znga.ask.push(Turn::user(question.clone()));
    app.znga.ask_dirty = true;
    let ask = app.znga.begin_ask(Asking {
        origin: AskOrigin::Panel,
        question: question.clone(),
        history: history.clone(),
        hits: Vec::new(),
    });
    if let Err(error) = app.znga.submit(
        prompt::plan_search(&question, &history),
        Purpose::Plan { ask },
    ) {
        app.znga.asking.remove(&ask);
        app.znga.ask.push(Turn::assistant(format!("⚠ {error}")));
    }
}

/// The archive and the chats Znga may read on `account`.
fn archive_of(app: &App, account: &str) -> Option<PathBuf> {
    app.accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .map(|known| known.dirs.archive_db())
}

fn visible_of(app: &App, account: &str) -> (Visible, HashMap<String, String>) {
    let mut readable = Visible::new();
    let mut names = HashMap::new();
    let config = &app.znga.config;
    if let Some(known) = app
        .accounts
        .iter()
        .find(|known| known.id.as_str() == account)
    {
        for chat in &known.chats {
            if !visible(config, ChatFacts::of(account, chat)) {
                continue;
            }
            let title = app.chat_title(chat);
            if !chat.is_group() {
                names.insert(chat.id.clone(), title.clone());
            }
            readable.insert(chat.id.clone(), (title, chat.is_group()));
        }
    }
    (readable, names)
}

fn origin_of(asking: &Asking) -> Origin {
    match asking.origin {
        AskOrigin::Panel => Origin::Person,
        AskOrigin::OwnChat { .. } => Origin::OwnChat,
    }
}

/// Searches the archive for an ask, when the Sentinel allows it, or goes
/// straight to the answer.
fn search_for(app: &mut App, ask: u64, terms: Vec<String>) {
    let Some(asking) = app.znga.asking.get(&ask).cloned() else {
        return;
    };
    let account = match &asking.origin {
        AskOrigin::OwnChat { account, .. } => account.clone(),
        AskOrigin::Panel => app.account().id.as_str().to_owned(),
    };
    let now = crate::util::now();
    let allowed = !terms.is_empty()
        && app
            .znga
            .audit
            .check(&Act::SearchArchive, origin_of(&asking), now);
    match archive_of(app, &account).filter(|_| allowed) {
        Some(archive) => {
            let (visible, names) = visible_of(app, &account);
            app.znga.store.send(Job::Search {
                id: ask,
                archive,
                terms,
                visible,
                names,
            });
            if asking.origin == AskOrigin::Panel {
                app.znga.set_status("Searching your chats…", false);
            }
        }
        None => answer(app, ask),
    }
}

/// Step 3 of an ask: the answer from recent activity and the hits.
fn answer(app: &mut App, ask: u64) {
    let Some(asking) = app.znga.asking.get(&ask).cloned() else {
        return;
    };
    let account = match &asking.origin {
        AskOrigin::OwnChat { account, .. } => account.clone(),
        AskOrigin::Panel => app.account().id.as_str().to_owned(),
    };
    let context = app.znga.activity.recent(120);
    let prompt = prompt::answer_with_hits(
        &your_name(app, &account),
        &context,
        &asking.hits,
        &asking.history,
        &asking.question,
    );
    if let Err(error) = app.znga.submit(prompt, Purpose::Answer { ask }) {
        let now = crate::util::now();
        let effects = app.znga.handle(Purpose::Answer { ask }, Err(error), now);
        apply(app, effects);
    }
}

/// Stops the running Ask answer, keeping what was written.
pub fn stop_ask(app: &mut App) {
    let running = app.znga.runner.partial(|purpose| {
        matches!(
            purpose,
            Purpose::Answer { .. } | Purpose::Plan { .. } | Purpose::Ask
        )
    });
    if let Some((id, _)) = running {
        app.znga.runner.cancel(id);
    }
}

/// Called for messages that reach a chat (`Event::Messages`), before the
/// conversation takes them: questions in the person's own chat
/// (REQ-ZNGA-012..014).
pub fn observe_messages(app: &mut App, chat: &str, messages: &[Message], older: bool) {
    let config = &app.znga.config;
    if !config.enabled || !config.self_chat || messages.is_empty() {
        return;
    }
    let account = app.account();
    let Some(own) = account.me.as_deref().and_then(proactive::own_chat) else {
        return;
    };
    if own != chat {
        return;
    }
    if account
        .chats
        .iter()
        .any(|known| known.id == chat && known.locked)
        || config.is_excluded(account.id.as_str(), chat)
    {
        return;
    }
    let account = account.id.as_str().to_owned();
    let since = config.self_chat_since;
    let now = crate::util::now();
    for message in messages {
        let text = match &message.content {
            Content::Text { text, .. } => Some(text.as_str()),
            _ => None,
        };
        if app.znga.selfchat.accept(
            &message.id,
            text,
            message.from_me,
            message.timestamp,
            since,
            now,
            older,
        ) {
            app.znga.selfchat.queue.push_back(Question {
                account: account.clone(),
                chat: chat.to_owned(),
                message: message.id.clone(),
                text: text.unwrap_or_default().to_owned(),
            });
        }
    }
    pump_self_chat(app);
}

/// Starts the next self-chat question when none is being answered.
fn pump_self_chat(app: &mut App) {
    if app.znga.selfchat.busy {
        return;
    }
    let Some(question) = app.znga.selfchat.queue.pop_front() else {
        return;
    };
    let now = crate::util::now();
    let reply = |text: String| Effect::SendOwn {
        account: question.account.clone(),
        chat: question.chat.clone(),
        text,
    };
    match app.znga.selfchat.take(now) {
        Allowance::Refuse => {
            pump_self_chat(app);
            return;
        }
        Allowance::Notice => {
            let notice = selfchat::marked(&format!(
                "That's {} answers today, my limit. I'll answer again tomorrow.",
                selfchat::PER_DAY
            ));
            apply(app, vec![reply(notice)]);
            return;
        }
        Allowance::Answer => {}
    }
    if let Some(reason) = app.znga.blocked() {
        apply(app, vec![reply(selfchat::marked(&format!("⚠ {reason}")))]);
        return;
    }
    let Some(archive) = archive_of(app, &question.account) else {
        return;
    };
    app.znga.selfchat.busy = true;
    let ask = app.znga.begin_ask(Asking {
        origin: AskOrigin::OwnChat {
            account: question.account.clone(),
            chat: question.chat.clone(),
        },
        question: question.text.clone(),
        history: Vec::new(),
        hits: Vec::new(),
    });
    app.znga.store.send(Job::History {
        ask,
        archive,
        chat: question.chat.clone(),
    });
}

/// The self-chat thread arrived: plan the search.
fn self_chat_history(app: &mut App, ask: u64, messages: Vec<(bool, String)>) {
    let Some(asking) = app.znga.asking.get_mut(&ask) else {
        return;
    };
    let mut turns = selfchat::turns(&messages);
    // The question itself is the newest message; it is asked separately.
    if turns
        .last()
        .is_some_and(|turn| turn.from_user && turn.text == asking.question.trim())
    {
        turns.pop();
    }
    asking.history = turns;
    let prompt = prompt::plan_search(&asking.question, &asking.history);
    if let Err(error) = app.znga.submit(prompt, Purpose::Plan { ask }) {
        let now = crate::util::now();
        let effects = app.znga.handle(Purpose::Answer { ask }, Err(error), now);
        apply(app, effects);
    }
}

/// "Catch me up" on the open chat (REQ-ZNGA-021).
pub fn request_catch_up(app: &mut App) {
    let Some(chat) = app.open_chat.clone() else {
        app.znga.set_status("Open a chat first.", true);
        return;
    };
    let account = app.account().id.as_str().to_owned();
    let unread = app.chat(&chat).map_or(0, |known| known.unread as usize);
    let lines = lines_n(app, &account, &chat, unread.clamp(20, 80));
    let chat_title = title_of(app, &account, &chat);
    let twin = twin_for(app, &account, &chat);
    let prompt = prompt::catch_up(&twin, &chat_title, &lines, unread);
    let purpose = Purpose::CatchUp {
        account,
        chat,
        chat_title,
    };
    match app.znga.submit(prompt, purpose) {
        Ok(()) => app.znga.set_status("Znga is catching you up…", false),
        Err(error) => app.znga.set_status(error, true),
    }
}

/// "Find plans" in the open chat (REQ-ZNGA-022).
pub fn request_events(app: &mut App) {
    let Some(chat) = app.open_chat.clone() else {
        app.znga.set_status("Open a chat first.", true);
        return;
    };
    let account = app.account().id.as_str().to_owned();
    let lines = lines_n(app, &account, &chat, 60);
    let chat_title = title_of(app, &account, &chat);
    let prompt = prompt::events(&chat_title, &lines, crate::util::now());
    let purpose = Purpose::Events {
        account,
        chat,
        chat_title,
    };
    match app.znga.submit(prompt, purpose) {
        Ok(()) => app.znga.set_status("Znga is looking for plans…", false),
        Err(error) => app.znga.set_status(error, true),
    }
}

/// Writes a found event as a calendar file and opens it, after the
/// Sentinel allows it (REQ-ZNGA-022.A4).
pub fn add_to_calendar(app: &mut App, index: usize) {
    let Some(found) = app.znga.events.get(index).cloned() else {
        return;
    };
    let name = super::calendar::file_name(&found.event);
    let now = crate::util::now();
    if !app
        .znga
        .audit
        .check(&Act::WriteFile { name: &name }, Origin::Person, now)
    {
        return;
    }
    let path = app.dirs.cache.join("znga-calendar").join(&name);
    let text = super::calendar::ics(&found.event, now, &format!("{now}-{index}"));
    std::thread::Builder::new()
        .name("znga-calendar".into())
        .spawn(move || {
            let written = path
                .parent()
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|()| std::fs::write(&path, text));
            match written {
                Ok(()) => {
                    if let Err(error) = crate::opener::open_or_reveal(&path) {
                        log::warn!(target: "zapfast::znga", "{error}");
                    }
                }
                Err(error) => log::warn!(target: "zapfast::znga", "calendar file: {error}"),
            }
        })
        .ok();
    app.znga.set_status(
        format!("Opening \"{}\" in your calendar…", found.event.title),
        false,
    );
}

/// Transcribes and summarises a voice note (REQ-ZNGA-023).
pub fn request_voice(app: &mut App, note: u64) {
    let Some(entry) = app.znga.voice_notes.iter().find(|n| n.id == note).cloned() else {
        return;
    };
    let set = |app: &mut App, state: VoiceState| {
        if let Some(entry) = app.znga.voice_notes.iter_mut().find(|n| n.id == note) {
            entry.state = state;
        }
    };
    if let Some(reason) = app.znga.blocked() {
        set(app, VoiceState::Failed(reason.into()));
        return;
    }
    let endpoint = app.znga.config.voice_endpoint();
    if !super::voice::is_local(&endpoint) && !app.znga.config.cloud_consent {
        set(
            app,
            VoiceState::Failed(
                "Confirm in Setup that the transcription service may receive voice notes.".into(),
            ),
        );
        return;
    }
    let media = app
        .accounts
        .iter()
        .find(|known| known.id.as_str() == entry.account)
        .and_then(|known| known.conversations.get(&entry.chat))
        .and_then(|conversation| conversation.messages.iter().find(|m| m.id == entry.message))
        .and_then(|message| message.content.media().cloned());
    let Some((path, mime)) = media.and_then(|media| Some((media.path?, media.mime))) else {
        set(
            app,
            VoiceState::Failed(
                "Open the chat and play or download the voice note first, then try again.".into(),
            ),
        );
        return;
    };
    let now = crate::util::now();
    if !app.znga.audit.check(&Act::Transcribe, Origin::Person, now) {
        return;
    }
    set(app, VoiceState::Working);
    let model = app.znga.config.voice_model();
    let key = if super::voice::is_local(&endpoint) {
        None
    } else {
        super::keys::resolve_key(super::config::Engine::OpenAi)
    };
    let sender = app.znga.voice_results.0.clone();
    let waker = app.waker.clone();
    std::thread::Builder::new()
        .name("znga-voice".into())
        .spawn(move || {
            let result = super::voice::transcribe(&endpoint, &model, key.as_deref(), &path, &mime);
            let _ = sender.send((note, result));
            waker.wake();
        })
        .ok();
}

fn voice_transcribed(app: &mut App, note: u64, result: Result<String, String>) {
    let Some(entry) = app.znga.voice_notes.iter().find(|n| n.id == note).cloned() else {
        return;
    };
    match result {
        Ok(transcript) => {
            let twin = twin_for(app, &entry.account, &entry.chat);
            let prompt =
                prompt::voice_summary(&twin, &entry.chat_title, &entry.sender_name, &transcript);
            if let Some(found) = app.znga.voice_notes.iter_mut().find(|n| n.id == note) {
                found.state = VoiceState::Done {
                    transcript,
                    summary: String::new(),
                };
            }
            if let Err(error) = app.znga.submit(prompt, Purpose::Voice { note }) {
                app.znga.set_status(error, true);
            }
        }
        Err(error) => {
            if let Some(found) = app.znga.voice_notes.iter_mut().find(|n| n.id == note) {
                found.state = VoiceState::Failed(error);
            }
        }
    }
}

/// "Explain" for an edited kept copy (REQ-ZNGA-026).
pub fn request_explain_edit(app: &mut App, key: String, before: &str, after: &str) {
    if app.znga.explanations.contains_key(&key)
        || app
            .znga
            .waiting_for(|p| matches!(p, Purpose::ExplainEdit { key: k } if *k == key))
    {
        return;
    }
    let prompt = prompt::explain_edit(before, after);
    if let Err(error) = app
        .znga
        .submit(prompt, Purpose::ExplainEdit { key: key.clone() })
    {
        app.znga.explanations.insert(key, Err(error));
    }
}

/// Rewrite or translate the composer text of the open chat.
pub fn request_rewrite(app: &mut App, prompt: super::provider::Prompt) {
    let Some(chat) = app.open_chat.clone() else {
        app.znga.set_status("Open a chat first.", true);
        return;
    };
    let account = app.account().id.as_str().to_owned();
    let purpose = Purpose::Draft {
        kind: Kind::Rewrite,
        chat_title: title_of(app, &account, &chat),
        account,
        chat,
    };
    match app.znga.submit(prompt, purpose) {
        Ok(()) => app.znga.set_status("Znga is rewriting…", false),
        Err(error) => app.znga.set_status(error, true),
    }
}

/// Sends an approved draft. The only way Znga's text reaches anyone else,
/// and only from the person's click.
pub fn send_draft(app: &mut App, draft: Draft) {
    let Some(account) = app
        .accounts
        .iter()
        .find(|known| known.id.as_str() == draft.account)
        .map(|known| known.id.clone())
    else {
        app.znga
            .set_status("That account is no longer linked.", true);
        return;
    };
    app.deferred_account_actions.push((
        account,
        Action::SendText {
            chat: draft.chat,
            text: draft.text,
            quoting: None,
        },
    ));
}

/// Puts a draft in the composer of its chat, opening the chat when needed.
pub fn use_in_composer(app: &mut App, draft: Draft) {
    if app.account().id.as_str() != draft.account {
        app.toast("Switch to that account to edit this draft.");
        return;
    }
    if app.open_chat.as_deref() == Some(draft.chat.as_str()) {
        app.composer = draft.text;
        app.focus_composer = true;
    } else {
        app.actions.push(Action::OpenChat(draft.chat.clone()));
        app.actions.push(Action::PrefillComposer {
            chat: draft.chat,
            text: draft.text,
        });
    }
}

/// Our own chat for the active account (or the first linked one).
fn own_chat(app: &App) -> Option<(crate::model::AccountId, String)> {
    std::iter::once(app.account())
        .chain(app.accounts.iter())
        .find_map(|account| {
            let chat = proactive::own_chat(account.me.as_deref()?)?;
            Some((account.id.clone(), chat))
        })
}

/// Our own chat's id on `account`.
fn own_chat_of(app: &App, account: &str) -> Option<String> {
    app.accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .and_then(|known| proactive::own_chat(known.me.as_deref()?))
}

/// Carries out what Znga asked for, each action through the Sentinel
/// (REQ-ZNGA-017).
fn apply(app: &mut App, effects: Vec<Effect>) {
    let now = crate::util::now();
    for effect in effects {
        match effect {
            Effect::Notify {
                title,
                body,
                account,
                chat,
                message,
                only_when_away,
            } => {
                if only_when_away && !away(app) {
                    continue;
                }
                if !app.znga.audit.check(&Act::Notify, Origin::Schedule, now) {
                    continue;
                }
                let account = if account.is_empty() {
                    app.account().id.as_str().to_owned()
                } else {
                    account
                };
                notify(app, title, body, &account, &chat, message);
            }
            Effect::SelfNote(text) => {
                if let Some((account, chat)) = own_chat(app) {
                    let own = own_chat_of(app, account.as_str());
                    let allowed = app.znga.audit.check(
                        &Act::Send {
                            chat: &chat,
                            own_chat: own.as_deref(),
                        },
                        Origin::Schedule,
                        now,
                    );
                    if allowed {
                        app.deferred_account_actions.push((
                            account,
                            Action::SendText {
                                chat,
                                text,
                                quoting: None,
                            },
                        ));
                    }
                }
            }
            Effect::SendOwn {
                account,
                chat,
                text,
            } => {
                app.znga.selfchat.busy = false;
                let own = own_chat_of(app, &account);
                let allowed = app.znga.audit.check(
                    &Act::Send {
                        chat: &chat,
                        own_chat: own.as_deref(),
                    },
                    Origin::OwnChat,
                    now,
                );
                let known = app
                    .accounts
                    .iter()
                    .find(|known| known.id.as_str() == account)
                    .map(|known| known.id.clone());
                if allowed && let Some(account) = known {
                    app.deferred_account_actions.push((
                        account,
                        Action::SendText {
                            chat,
                            text,
                            quoting: None,
                        },
                    ));
                }
                pump_self_chat(app);
            }
            Effect::Search { ask, terms } => search_for(app, ask, terms),
        }
    }
}

/// Re-reads our own sent text messages from the loaded conversations, for
/// the twin's style.
fn scan_style(app: &mut App) {
    let mut learned: Vec<(String, String, String)> = Vec::new();
    for account in &app.accounts {
        let id = account.id.as_str().to_owned();
        for (chat, conversation) in &account.conversations {
            if app.znga.config.is_excluded(&id, chat) {
                continue;
            }
            if account
                .chats
                .iter()
                .any(|known| known.id == *chat && known.locked)
            {
                continue;
            }
            let skip = conversation.messages.len().saturating_sub(40);
            for message in &conversation.messages[skip..] {
                if message.from_me
                    && let Content::Text { text, .. } = &message.content
                {
                    learned.push((id.clone(), chat.clone(), text.clone()));
                }
            }
        }
    }
    for (account, chat, text) in &learned {
        app.znga.learn(account, chat, text);
    }
    let accounts: Vec<String> = app
        .accounts
        .iter()
        .map(|account| account.id.as_str().to_owned())
        .collect();
    if app.znga.remeasure(&accounts) {
        app.znga.save_styles(&app.files);
    }
}

fn scan_checkins(app: &mut App) {
    if app.znga.config.twin_chats.is_empty() {
        return;
    }
    let chats: Vec<(String, String, i64)> = app
        .accounts
        .iter()
        .flat_map(|account| {
            account.chats.iter().map(move |chat| {
                (
                    account.id.as_str().to_owned(),
                    chat.id.clone(),
                    chat.last_activity,
                )
            })
        })
        .collect();
    let now = crate::util::now();
    let due = proactive::checkins_due(&app.znga.config, &chats, &app.znga.checkins_suggested, now);
    for (account, chat) in due.into_iter().take(2) {
        app.znga
            .checkins_suggested
            .insert(chat_key(&account, &chat), now);
        request_draft(app, &account, &chat, Kind::CheckIn);
    }
}

/// Asks for the active account's `znga.db` and seeds activity from its
/// archive, once (REQ-ZNGA-016).
fn ensure_loaded(app: &mut App) {
    let account = app.account().id.as_str().to_owned();
    if app.znga.ask_account.as_deref() != Some(account.as_str()) {
        if app.znga.ask_dirty {
            save_ask(app);
        }
        app.znga.ask.clear();
        app.znga.ask_account = Some(account.clone());
        if app.znga.loaded.contains(&account)
            && let Some(archive) = archive_of(app, &account)
        {
            app.znga.store.send(Job::Load {
                account: account.clone(),
                archive,
            });
        }
    }
    if app.znga.loaded.contains(&account) {
        return;
    }
    let Some(archive) = archive_of(app, &account) else {
        return;
    };
    if !archive.is_file() {
        return;
    }
    app.znga.loaded.insert(account.clone());
    app.znga.store.send(Job::Load {
        account: account.clone(),
        archive: archive.clone(),
    });
    let (visible, names) = visible_of(app, &account);
    app.znga.store.send(Job::Seed {
        account,
        archive,
        visible,
        names,
    });
}

fn save_ask(app: &mut App) {
    app.znga.ask_dirty = false;
    let Some(account) = app.znga.ask_account.clone() else {
        return;
    };
    if let Some(archive) = archive_of(app, &account) {
        app.znga.store.send(Job::SaveAsk {
            archive,
            turns: app.znga.ask.clone(),
        });
    }
}

/// Hands changed memory to the store, per account.
fn save_memory(app: &mut App) {
    if app.znga.ask_dirty {
        save_ask(app);
    }
    let facts = app.znga.people.take_dirty();
    let promises = app.znga.promises.take_dirty();
    if facts || promises {
        let accounts: Vec<String> = app
            .accounts
            .iter()
            .map(|a| a.id.as_str().to_owned())
            .collect();
        for account in accounts {
            let Some(archive) = archive_of(app, &account) else {
                continue;
            };
            if !app.znga.restored.contains(&account) {
                // Never overwrite a znga.db that was not read yet; its
                // changes are saved, merged, once it loads.
                continue;
            }
            if facts {
                let list = app
                    .znga
                    .people
                    .all()
                    .iter()
                    .filter(|f| f.account == account)
                    .cloned()
                    .collect();
                app.znga.store.send(Job::SaveFacts {
                    archive: archive.clone(),
                    facts: list,
                });
            }
            if promises {
                let items = app
                    .znga
                    .promises
                    .all()
                    .iter()
                    .filter(|p| p.account == account)
                    .cloned()
                    .collect();
                app.znga.store.send(Job::SavePromises { archive, items });
            }
        }
    }
    let entries = app.znga.audit.take_unsaved();
    if !entries.is_empty() {
        let account = app.account().id.as_str().to_owned();
        if let Some(archive) = archive_of(app, &account).filter(|a| a.is_file()) {
            app.znga.store.send(Job::Audit { archive, entries });
        }
    }
}

fn store_replies(app: &mut App) {
    for reply in app.znga.store.poll() {
        match reply {
            Reply::Loaded { account, loaded } => {
                if app.znga.ask_account.as_deref() == Some(account.as_str())
                    && app.znga.ask.is_empty()
                {
                    app.znga.ask = loaded.ask;
                }
                // Merged with anything found before the load finished.
                let first = app.znga.restored.insert(account.clone());
                if first {
                    app.znga.people.restore(loaded.facts);
                    app.znga.promises.restore(loaded.promises);
                    app.znga.people.mark_dirty();
                    app.znga.promises.mark_dirty();
                }
                if app.account().id.as_str() == account {
                    app.znga.audit.restore(loaded.audit);
                }
            }
            Reply::Seeded { seen, .. } => app.znga.activity.seed(seen),
            Reply::Found { id, hits } => {
                if let Some(asking) = app.znga.asking.get_mut(&id) {
                    asking.hits = hits.unwrap_or_default();
                }
                answer(app, id);
            }
            Reply::Rhythms { account, rhythms } => {
                app.znga.rhythms.retain(|r| r.account != account);
                app.znga.rhythms.extend(rhythms);
            }
            Reply::History { ask, messages } => self_chat_history(app, ask, messages),
            Reply::Failed(error) => log::warn!(target: "zapfast::znga", "store: {error}"),
        }
    }
}

/// Mines twin chats with new messages for promises and facts
/// (REQ-ZNGA-018, REQ-ZNGA-019). At most two chats per scan.
fn scan_extract(app: &mut App) {
    let config = &app.znga.config;
    if !config.promises && !config.memory {
        return;
    }
    let mut due: Vec<(String, String, i64)> = Vec::new();
    for account in &app.accounts {
        let id = account.id.as_str();
        for (chat, conversation) in &account.conversations {
            if !config.is_twin(id, chat) || config.is_excluded(id, chat) {
                continue;
            }
            if account.chats.iter().any(|c| c.id == *chat && c.locked) {
                continue;
            }
            let Some(newest) = conversation.messages.last().map(|m| m.timestamp) else {
                continue;
            };
            let key = chat_key(id, chat);
            if app
                .znga
                .extracted
                .get(&key)
                .is_some_and(|seen| *seen >= newest)
            {
                continue;
            }
            due.push((id.to_owned(), chat.clone(), newest));
        }
    }
    due.sort_by_key(|(_, _, newest)| -newest);
    for (account, chat, newest) in due.into_iter().take(2) {
        app.znga.extracted.insert(chat_key(&account, &chat), newest);
        let lines = lines_of(app, &account, &chat);
        let chat_title = title_of(app, &account, &chat);
        let name = your_name(app, &account);
        let zone = jiff::tz::TimeZone::system();
        let now = jiff::Timestamp::now()
            .to_zoned(zone)
            .strftime("%A %Y-%m-%d %H:%M")
            .to_string();
        if app.znga.config.promises {
            let prompt = prompt::promises(&name, &chat_title, &lines, &now);
            let _ = app.znga.submit(
                prompt,
                Purpose::Promises {
                    account: account.clone(),
                    chat: chat.clone(),
                    chat_title: chat_title.clone(),
                },
            );
        }
        if app.znga.config.memory {
            let known = app.znga.people.about(&account, &chat);
            let prompt = prompt::facts(&name, &chat_title, &lines, &known);
            let _ = app.znga.submit(
                prompt,
                Purpose::Facts {
                    account,
                    chat,
                    chat_title,
                },
            );
        }
    }
}

/// Reminds the person of promises coming due (REQ-ZNGA-018.A3).
fn check_promises(app: &mut App) {
    let now = crate::util::now();
    for promise in app.znga.promises.due(now) {
        let title = match promise.owes {
            super::promises::Owes::Me => format!("Znga · you promised {}", promise.chat_title),
            super::promises::Owes::Them => {
                format!("Znga · {} is waiting on you", promise.chat_title)
            }
        };
        apply(
            app,
            vec![Effect::Notify {
                title,
                body: promise.text.clone(),
                account: promise.account.clone(),
                chat: promise.chat.clone(),
                message: None,
                only_when_away: false,
            }],
        );
        request_draft(app, &promise.account, &promise.chat, Kind::Suggestion);
    }
}

/// Reads twin chats' rhythms from the archive (REQ-ZNGA-024).
pub fn scan_rhythms(app: &mut App) {
    let now = crate::util::now();
    let accounts: Vec<String> = app
        .accounts
        .iter()
        .map(|a| a.id.as_str().to_owned())
        .collect();
    for account in accounts {
        let chats: Vec<(String, String)> = app
            .znga
            .config
            .twin_chats
            .iter()
            .filter_map(|key| key.strip_prefix(&format!("{account}/")))
            .filter(|chat| !app.znga.config.is_excluded(&account, chat))
            .map(|chat| (chat.to_owned(), title_of(app, &account, chat)))
            .collect();
        if chats.is_empty() {
            continue;
        }
        if let Some(archive) = archive_of(app, &account).filter(|a| a.is_file()) {
            app.znga.store.send(Job::Rhythms {
                account,
                archive,
                chats,
                now,
            });
        }
    }
}

/// Drops memory about chats Znga may no longer see (REQ-ZNGA-019.A4).
fn forget_hidden(app: &mut App) {
    let hidden: std::collections::HashSet<String> = app
        .accounts
        .iter()
        .flat_map(|account| {
            account
                .chats
                .iter()
                .filter(|chat| chat.locked)
                .map(move |chat| chat_key(account.id.as_str(), &chat.id))
        })
        .collect();
    let config = app.znga.config.clone();
    let keep = |account: &str, chat: &str| {
        !config.is_excluded(account, chat) && !hidden.contains(&chat_key(account, chat))
    };
    app.znga.people.retain(|f| keep(&f.account, &f.chat));
    app.znga.promises.retain(|p| keep(&p.account, &p.chat));
}

/// Once a frame, after the app handled its events.
pub fn tick(app: &mut App) {
    let now = crate::util::now();
    for (purpose, answer) in app.znga.runner.poll() {
        let effects = app.znga.handle(purpose, answer, now);
        apply(app, effects);
    }
    store_replies(app);
    while let Ok((note, result)) = app.znga.voice_results.1.try_recv() {
        voice_transcribed(app, note, result);
    }
    app.znga.save_if_dirty(&app.files);
    if !app.znga.config.enabled {
        return;
    }
    ensure_loaded(app);
    save_memory(app);
    let instant = std::time::Instant::now();
    if app
        .znga
        .last_scan
        .is_none_or(|last| instant.duration_since(last) >= STYLE_SCAN)
    {
        app.znga.last_scan = Some(instant);
        scan_style(app);
        forget_hidden(app);
    }
    if app
        .znga
        .last_promise_check
        .is_none_or(|last| instant.duration_since(last) >= PROMISE_CHECK)
    {
        app.znga.last_promise_check = Some(instant);
        check_promises(app);
    }
    if app.znga.blocked().is_some() {
        return;
    }
    let is_briefing = |purpose: &Purpose| matches!(purpose, Purpose::Briefing { .. });
    if app.znga.activity.due(&app.znga.config, now, away(app)) && !app.znga.waiting_for(is_briefing)
    {
        request_briefing(app, true);
    }
    if app
        .znga
        .last_checkin_scan
        .is_none_or(|last| instant.duration_since(last) >= CHECKIN_SCAN)
    {
        app.znga.last_checkin_scan = Some(instant);
        scan_checkins(app);
    }
    if app
        .znga
        .last_extract
        .is_none_or(|last| instant.duration_since(last) >= EXTRACT_SCAN)
    {
        app.znga.last_extract = Some(instant);
        scan_extract(app);
    }
    if app
        .znga
        .last_rhythms
        .is_none_or(|last| instant.duration_since(last) >= RHYTHM_SCAN)
    {
        app.znga.last_rhythms = Some(instant);
        scan_rhythms(app);
    }
}
