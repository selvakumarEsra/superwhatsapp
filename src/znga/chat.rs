//! The Znga chat's bridge to the app (specs/znga-chat.md): what the person
//! writes to Znga, the buttons on its messages, the composer's Znga tools,
//! and everything Znga posts on its own. Only the Tauri interface turns
//! this on ([`super::Znga::chat_mode`]); the egui panel is unchanged.
//!
//! Nothing here sends a WhatsApp message except [`card_action`]'s Send,
//! which runs only from the person's click (REQ-ZCHAT-006.A4).

use crate::app::App;
use crate::model::Action;

use super::calendar;
use super::config::{Config, chat_key};
use super::drafts::{Draft, Kind as DraftKind};
use super::hooks;
use super::privacy::{ChatFacts, visible};
use super::promises::{Owes, State as PromiseState};
use super::prompt;
use super::sentinel::{Action as Act, Entry as AuditEntry, Origin};
use super::thread::{self, Card, Intent, Item, Kind, Posted, Question, Thread};
use super::{Offer, Purpose, VoiceState};

fn active(app: &App) -> String {
    app.account().id.as_str().to_owned()
}

/// Whether the Znga chat is in use: the Tauri interface with Znga on.
pub fn enabled(app: &App) -> bool {
    app.znga.chat_mode && app.znga.config.enabled
}

/// The person is looking at the Znga chat now.
pub fn reading(app: &App) -> bool {
    app.znga.chat_open && app.open_chat.is_none() && !hooks::away(app)
}

fn thread_of<'a>(app: &'a mut App, account: &str) -> &'a mut Thread {
    app.znga.threads.entry(account.to_owned()).or_default()
}

fn kind_on(config: &Config, kind: Kind) -> bool {
    match kind {
        Kind::NeedsYou => config.alerts,
        Kind::WelcomeBack => config.briefings,
        Kind::Digest => config.digests,
        Kind::Reminder => config.reminders,
    }
}

fn local_now() -> jiff::Zoned {
    jiff::Timestamp::now().to_zoned(jiff::tz::TimeZone::system())
}

fn minute_of(zoned: &jiff::Zoned) -> u32 {
    zoned.hour() as u32 * 60 + zoned.minute() as u32
}

fn day_of(zoned: &jiff::Zoned) -> i64 {
    (zoned.timestamp().as_second() + i64::from(zoned.offset().seconds())).div_euclid(86_400)
}

/// Inside the person's quiet hours now (REQ-ZCHAT-005.A4).
pub fn quiet(config: &Config) -> bool {
    let (Some(from), Some(to)) = (
        thread::parse_hhmm(&config.quiet_from),
        thread::parse_hhmm(&config.quiet_to),
    ) else {
        return false;
    };
    config.quiet_hours && thread::in_quiet(minute_of(&local_now()), from, to)
}

/// Every chat of the active account as (id, title), locked and excluded
/// ones included so naming one can be refused.
fn all_chats(app: &App) -> Vec<(String, String)> {
    app.account()
        .chats
        .iter()
        .map(|chat| (chat.id.clone(), app.chat_title(chat)))
        .collect()
}

/// Whether Znga may read `chat` on `account`. Unknown chats are not.
fn chat_visible(app: &App, account: &str, chat: &str) -> bool {
    app.accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .and_then(|known| known.chats.iter().find(|c| c.id == chat))
        .is_some_and(|found| visible(&app.znga.config, ChatFacts::of(account, found)))
}

/// Znga's answer to something the person asked (REQ-ZCHAT-002).
pub fn answer(app: &mut App, account: &str, text: impl Into<String>, items: Vec<Item>) {
    let reading = reading(app) && account == active(app);
    let now = crate::util::now();
    thread_of(app, account).post(text, items, now, reading);
}

/// A line saying what the person asked for, written as theirs.
fn asked_line(app: &mut App, text: impl Into<String>) {
    let account = active(app);
    let now = crate::util::now();
    thread_of(app, &account).post_person(text, now);
}

/// Something Znga says on its own (REQ-ZCHAT-005): only when its kind is
/// on, never naming a locked or excluded chat, folded and paced by the
/// thread, with a "Znga" notification for a new message.
pub fn unasked(app: &mut App, account: &str, kind: Kind, line: String, items: Vec<Item>) {
    if !enabled(app) || !kind_on(&app.znga.config, kind) {
        return;
    }
    let hidden = items
        .iter()
        .filter_map(|item| item.card.chat())
        .any(|chat| !chat_visible(app, account, chat));
    if hidden {
        return;
    }
    let reading = reading(app) && account == active(app);
    let quiet = quiet(&app.znga.config);
    let now = crate::util::now();
    let posted = thread_of(app, account).unasked(kind, line.clone(), items, now, reading, quiet);
    if matches!(posted, Posted::New(_)) && !reading {
        notify(app, account, &line);
    }
}

fn notify(app: &mut App, account: &str, line: &str) {
    let now = crate::util::now();
    if app.znga.audit.check(&Act::Notify, Origin::Schedule, now) {
        hooks::notify(
            app,
            "Znga".into(),
            hooks::clip(line, 140),
            account,
            "",
            None,
        );
    }
}

/// Puts text in front of a chat's composer.
fn offer(app: &mut App, chat: &str, text: &str, rewrite: bool) {
    app.znga.next_offer += 1;
    app.znga.offer = Some(Offer {
        id: app.znga.next_offer,
        chat: chat.to_owned(),
        text: text.to_owned(),
        rewrite,
    });
}

/// Says an error from a request the person made in the chat, and clears
/// the panel's status line, which the chat replaces.
fn say_status_error(app: &mut App) {
    if let Some(status) = app.znga.status.take()
        && status.error
    {
        let account = active(app);
        let text = thread::explain_error(&status.text, app.znga.config.engine);
        answer(app, &account, text, Vec::new());
    }
}

// ---------------------------------------------------------------------
// What the person writes to Znga (REQ-ZCHAT-002, REQ-ZCHAT-006).

/// A message the person wrote in the Znga chat.
/// Opens the Znga chat in place of the open conversation (REQ-ZCHAT-001.A6).
pub fn open_thread(app: &mut App) {
    if app.open_chat.is_some() {
        app.actions.push(crate::model::Action::CloseChat);
    }
    if app.page != crate::model::Page::Chats {
        app.actions
            .push(crate::model::Action::Open(crate::model::Page::Chats));
    }
    app.znga.chat_open = true;
    let account = active(app);
    app.znga.threads.entry(account).or_default().mark_read();
}

pub fn send_message(app: &mut App, text: &str) {
    let text = text.trim();
    if text.is_empty() || !enabled(app) {
        return;
    }
    let account = active(app);
    let now = crate::util::now();
    let thread = thread_of(app, &account);
    thread.post_person(text, now);
    if let Some(question) = thread.question.take() {
        if let Some(choice) = thread::pick(text, &question.choices).cloned() {
            resolve(app, question.intent, choice);
            return;
        }
        if matches!(thread::intent(text), Intent::Ask(_)) {
            // Neither a pick nor a new instruction: ask again.
            let again = Question {
                intent: question.intent,
                choices: question.choices,
            };
            ask_which(app, again);
            return;
        }
    }
    match thread::intent(text) {
        Intent::Done => {
            let found = thread_of(app, &account).latest_needs_you();
            match found {
                Some((entry, index)) => {
                    thread_of(app, &account).settle(entry, index, "Done");
                    answer(app, &account, "Marked done.", Vec::new());
                }
                None => answer(app, &account, "Nothing is waiting for you.", Vec::new()),
            }
        }
        Intent::Tell { rest } => tell(app, &rest),
        Intent::Remind { what } => remind(app, &what),
        Intent::CatchUp { name } => catch_up_named(app, &name),
        Intent::Ask(question) => ask(app, question),
    }
}

fn ask_which(app: &mut App, question: Question) {
    let account = active(app);
    let list = question
        .choices
        .iter()
        .enumerate()
        .map(|(n, (_, title))| format!("{}. {title}", n + 1))
        .collect::<Vec<_>>()
        .join("\n");
    answer(
        app,
        &account,
        format!("Which one do you mean?\n{list}\nAnswer with the number or the full name."),
        Vec::new(),
    );
    thread_of(app, &account).question = Some(question);
}

/// Narrows matches to chats Znga may read; one left is the one meant, as
/// the hidden ones do not exist for Znga.
fn readable_choices(app: &App, found: Vec<&(String, String)>) -> Vec<(String, String)> {
    let account = active(app);
    let all: Vec<(String, String)> = found.into_iter().cloned().collect();
    let readable: Vec<(String, String)> = all
        .iter()
        .filter(|(id, _)| chat_visible(app, &account, id))
        .cloned()
        .collect();
    if readable.is_empty() { all } else { readable }
}

fn resolve(app: &mut App, intent: Intent, (chat, title): (String, String)) {
    match intent {
        Intent::Tell { rest } => {
            let chats = vec![(chat.clone(), title.clone())];
            let message = thread::addressee(&rest, &all_chats(app))
                .map(|(_, message)| message)
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| {
                    // The name was only partly given; the rest is the message.
                    rest.split_once(' ')
                        .map_or(rest.clone(), |(_, m)| m.to_owned())
                });
            let _ = chats;
            draft_to(app, &chat, &title, &message);
        }
        Intent::Remind { what } => remind_for(app, &what, Some((chat, title))),
        Intent::CatchUp { .. } => catch_up_chat(app, &chat),
        Intent::Done | Intent::Ask(_) => {}
    }
}

/// The Sentinel's check before Znga reads a chat the person named
/// (REQ-ZCHAT-006.A3). A refusal is said in the chat and logged.
fn may_read(app: &mut App, chat: &str) -> bool {
    let account = active(app);
    let hidden = !chat_visible(app, &account, chat);
    let now = crate::util::now();
    let allowed = app
        .znga
        .audit
        .check(&Act::Read { hidden }, Origin::Person, now);
    if !allowed {
        answer(
            app,
            &account,
            "I can't do that: that chat is locked or excluded, and Znga never reads those.",
            Vec::new(),
        );
    }
    allowed
}

fn tell(app: &mut App, rest: &str) {
    let account = active(app);
    let chats = all_chats(app);
    match thread::addressee(rest, &chats) {
        None => answer(
            app,
            &account,
            "I couldn't tell who you mean. Use their name as it appears in your chats, like \
             \"tell Ada I'll call at 5\".",
            Vec::new(),
        ),
        Some((found, message)) => {
            let choices = readable_choices(app, found);
            if choices.len() > 1 {
                ask_which(
                    app,
                    Question {
                        intent: Intent::Tell {
                            rest: rest.to_owned(),
                        },
                        choices,
                    },
                );
            } else if message.is_empty() {
                answer(
                    app,
                    &account,
                    format!("What should I tell {}?", choices[0].1),
                    Vec::new(),
                );
            } else {
                let (chat, title) = choices[0].clone();
                draft_to(app, &chat, &title, &message);
            }
        }
    }
}

/// A draft to `chat` saying `message`, posted with Send, Edit in chat and
/// Discard. Nothing is sent (REQ-ZCHAT-006.A1).
fn draft_to(app: &mut App, chat: &str, title: &str, message: &str) {
    if !may_read(app, chat) {
        return;
    }
    let account = active(app);
    let now = crate::util::now();
    app.znga
        .audit
        .check(&Act::Draft { chat }, Origin::Person, now);
    app.znga.asked.insert(chat_key(&account, chat));
    let lines = hooks::lines_n(app, &account, chat, 20);
    let twin = hooks::twin_for(app, &account, chat);
    let prompt = prompt::tell(&twin, title, message, &lines);
    let purpose = Purpose::Draft {
        kind: DraftKind::Update,
        account: account.clone(),
        chat: chat.to_owned(),
        chat_title: title.to_owned(),
    };
    if let Err(error) = app.znga.submit(prompt, purpose) {
        app.znga.asked.remove(&chat_key(&account, chat));
        let text = thread::explain_error(&error, app.znga.config.engine);
        answer(app, &account, text, Vec::new());
    }
}

fn remind(app: &mut App, what: &str) {
    let chats = all_chats(app);
    let named = thread::mentioned(what, &chats).cloned();
    remind_for(app, what, named);
}

/// Keeps a promise to remind the person (REQ-ZCHAT-006).
fn remind_for(app: &mut App, what: &str, chat: Option<(String, String)>) {
    if let Some((id, _)) = &chat
        && !may_read(app, id)
    {
        return;
    }
    let account = active(app);
    let now = crate::util::now();
    app.znga.audit.check(&Act::Remind, Origin::Person, now);
    let zone = jiff::tz::TimeZone::system();
    let local = local_now();
    let due = thread::due_in(what, local.datetime())
        .and_then(|at| at.to_zoned(zone).ok())
        .map(|at| at.timestamp().as_second());
    let (chat, title) = chat.unwrap_or_default();
    app.znga.promises.add(
        &account,
        &chat,
        &title,
        vec![(Owes::Me, what.to_owned(), due)],
        now,
    );
    let when = match due {
        Some(at) => format!(
            ", {}",
            jiff::Timestamp::from_second(at)
                .map(|t| t
                    .to_zoned(jiff::tz::TimeZone::system())
                    .strftime("%A %-d %B at %H:%M")
                    .to_string())
                .unwrap_or_default()
        ),
        None => " (no time given: it is in your open promises)".into(),
    };
    answer(
        app,
        &account,
        format!("I'll remind you to {what}{when}."),
        Vec::new(),
    );
}

fn catch_up_named(app: &mut App, name: &str) {
    let account = active(app);
    let chats = all_chats(app);
    let mut found = thread::candidates(name, &chats);
    for suffix in [" group", " chat"] {
        if found.is_empty()
            && let Some(short) = name.to_lowercase().strip_suffix(suffix)
        {
            found = thread::candidates(short, &chats);
        }
    }
    if found.is_empty() {
        answer(
            app,
            &account,
            format!("I couldn't find a chat called \"{name}\"."),
            Vec::new(),
        );
        return;
    }
    let choices = readable_choices(app, found);
    if choices.len() > 1 {
        ask_which(
            app,
            Question {
                intent: Intent::CatchUp {
                    name: name.to_owned(),
                },
                choices,
            },
        );
    } else {
        catch_up_chat(app, &choices[0].0.clone());
    }
}

fn catch_up_chat(app: &mut App, chat: &str) {
    if !may_read(app, chat) {
        return;
    }
    hooks::request_catch_up_for(app, chat);
    say_status_error(app);
}

/// A question for the archive, answered in the chat (REQ-ZCHAT-002.A1).
fn ask(app: &mut App, question: String) {
    let account = active(app);
    let history: Vec<super::provider::Turn> = {
        let thread = thread_of(app, &account);
        let entries = thread.entries();
        // The question itself is the last entry.
        let skip = entries.len().saturating_sub(21);
        entries[skip..entries.len().saturating_sub(1)]
            .iter()
            .filter(|e| e.items.is_empty() && !e.text.starts_with('⚠'))
            .map(|e| super::provider::Turn {
                from_user: e.from_person,
                text: e.text.clone(),
            })
            .collect()
    };
    let ask = app.znga.begin_ask(super::archive_ask::Asking {
        origin: super::archive_ask::AskOrigin::Thread {
            account: account.clone(),
        },
        question: question.clone(),
        history: history.clone(),
        hits: Vec::new(),
    });
    if let Err(error) = app.znga.submit(
        prompt::plan_search(&question, &history),
        Purpose::Plan { ask },
    ) {
        app.znga.asking.remove(&ask);
        let text = thread::explain_error(&error, app.znga.config.engine);
        answer(app, &account, text, Vec::new());
    }
}

/// The answer that is being written now, for the chat: `Some("")` while
/// Znga is still searching or thinking.
pub fn writing(app: &App) -> Option<String> {
    let account = active(app);
    let thread_ask = |ask: &u64| {
        app.znga.asking.get(ask).is_some_and(|a| {
            matches!(&a.origin, super::archive_ask::AskOrigin::Thread { account: a } if *a == account)
        })
    };
    if let Some((_, text)) = app
        .znga
        .runner
        .partial(|p| matches!(p, Purpose::Answer { ask } if thread_ask(ask)))
    {
        return Some(text.to_owned());
    }
    let busy = app.znga.runner.is_waiting(|p| match p {
        Purpose::Plan { ask } => thread_ask(ask),
        Purpose::Draft {
            kind,
            account: a,
            chat,
            ..
        } => {
            *a == account
                && (!matches!(kind, DraftKind::AutoReply | DraftKind::CheckIn)
                    || app.znga.asked.contains(&chat_key(a, chat)))
        }
        Purpose::CatchUp { account: a, .. } | Purpose::Events { account: a, .. } => *a == account,
        Purpose::Voice { .. } | Purpose::Briefing { proactive: false } => true,
        _ => false,
    });
    busy.then(String::new)
}

/// Stop: ends the answer being written, keeping what was written.
pub fn stop(app: &mut App) {
    hooks::stop_ask(app);
}

// ---------------------------------------------------------------------
// The buttons on Znga's messages (REQ-ZCHAT-003).

fn open_chat(app: &mut App, chat: &str) {
    app.znga.chat_open = false;
    app.actions.push(Action::OpenChat(chat.to_owned()));
}

/// The person pressed `action` on card `index` of message `entry`. A card
/// already acted on does nothing (REQ-ZCHAT-003.A2).
pub fn card_action(app: &mut App, entry: u64, index: usize, action: &str) {
    if !enabled(app) {
        return;
    }
    let account = active(app);
    let Some(card) = app
        .znga
        .threads
        .get(&account)
        .and_then(|t| t.open_card(entry, index))
        .cloned()
    else {
        return;
    };
    if !card.actions().iter().any(|(id, _)| *id == action) {
        return;
    }
    if let Some(chat) = card.chat()
        && app.account().chats.iter().all(|c| c.id != chat)
    {
        // The card's chat is on another linked account (REQ-ZCHAT-003.A4).
        answer(
            app,
            &account,
            "Switch to that account to do this.",
            Vec::new(),
        );
        return;
    }
    let now = crate::util::now();
    let mut outcome = Some(Card::outcome(action).to_owned());
    match (&card, action) {
        (Card::NeedsYou { chat, .. } | Card::CatchUp { chat, .. }, "open") => open_chat(app, chat),
        (Card::NeedsYou { chat, text, .. }, "draft") => {
            app.znga
                .alerts
                .retain(|a| !(a.chat == *chat && a.text == *text));
            app.znga.asked.insert(chat_key(&account, chat));
            hooks::request_draft(app, &account, chat, DraftKind::Suggestion);
            say_status_error(app);
        }
        (Card::NeedsYou { chat, text, .. }, "done") => {
            app.znga
                .alerts
                .retain(|a| !(a.chat == *chat && a.text == *text));
        }
        (Card::Briefing { .. }, "copy") => {}
        (
            Card::Draft {
                draft,
                chat,
                chat_title,
                text,
                label: _,
            },
            "send" | "edit" | "discard",
        ) => {
            app.znga.drafts.take(*draft);
            let draft = Draft {
                id: *draft,
                kind: DraftKind::Suggestion,
                account: account.clone(),
                chat: chat.clone(),
                chat_title: chat_title.clone(),
                text: text.clone(),
                created: now,
            };
            match action {
                "send" => {
                    // The one path from Znga to another person, and only
                    // from this click.
                    app.znga.audit.log(AuditEntry {
                        at: now,
                        action: "send a draft you approved".into(),
                        origin: Origin::Person,
                        allowed: true,
                        reason: "you pressed Send".into(),
                    });
                    hooks::send_draft(app, draft);
                }
                "edit" => {
                    open_chat(app, chat);
                    offer(app, chat, text, false);
                }
                _ => {}
            }
        }
        (Card::Plan { .. }, "calendar") => {
            if let Card::Plan {
                title,
                start,
                end,
                place,
                notes,
                ..
            } = &card
            {
                let event = calendar::Event {
                    title: title.clone(),
                    start: *start,
                    end: *end,
                    place: place.clone(),
                    notes: notes.clone(),
                };
                if !hooks::write_event(app, &event, &format!("{now}-{entry}-{index}")) {
                    outcome = Some("Not allowed".into());
                }
            }
        }
        (Card::Plan { title, start, .. }, "dismiss") => {
            app.znga
                .events
                .retain(|e| !(e.event.title == *title && e.event.start == *start));
        }
        (Card::Voice { note, .. }, "summarise") => {
            if app.znga.voice_notes.iter().any(|n| n.id == *note) {
                app.znga.asked.insert(format!("voice:{note}"));
                hooks::request_voice(app, *note);
                voice_failures(app);
            } else {
                answer(
                    app,
                    &account,
                    "That voice note is no longer here. Open the chat, play it, and try again.",
                    Vec::new(),
                );
                outcome = None;
            }
        }
        (Card::Promise { chat, text, .. }, "done" | "notPromise") => {
            let index = app
                .znga
                .promises
                .all()
                .iter()
                .position(|p| p.account == account && p.chat == *chat && p.text == *text);
            if let Some(index) = index {
                let state = if action == "done" {
                    PromiseState::Done
                } else {
                    PromiseState::Dismissed
                };
                app.znga.promises.set_state(index, state);
            }
        }
        (Card::Drift { chat, .. }, "opener") => {
            app.znga.asked.insert(chat_key(&account, chat));
            hooks::request_opener(app, &account, chat);
            say_status_error(app);
        }
        _ => outcome = None,
    }
    if let Some(outcome) = outcome
        && Card::closes(action)
    {
        thread_of(app, &account).settle(entry, index, outcome);
    }
}

fn voice_failures(app: &mut App) {
    let failed: Vec<(u64, String, String)> = app
        .znga
        .voice_notes
        .iter()
        .filter(|n| app.znga.asked.contains(&format!("voice:{}", n.id)))
        .filter_map(|n| match &n.state {
            VoiceState::Failed(error) => Some((n.id, n.account.clone(), error.clone())),
            _ => None,
        })
        .collect();
    for (id, account, error) in failed {
        app.znga.asked.remove(&format!("voice:{id}"));
        let text = thread::explain_error(&error, app.znga.config.engine);
        answer(app, &account, text, Vec::new());
    }
}

// ---------------------------------------------------------------------
// The composer's Znga tools (REQ-ZCHAT-004).

/// Whether the composer of `chat` offers Znga's tools.
pub fn tools_for(app: &App, chat: &str) -> bool {
    enabled(app) && chat != thread::ZNGA_CHAT && chat_visible(app, &active(app), chat)
}

/// One of the composer's Znga tools on the open chat. `text` is the
/// composer's text; rewrites need some.
pub fn tool(app: &mut App, chat: &str, tool: &str, text: &str, language: &str) {
    if !tools_for(app, chat) || app.open_chat.as_deref() != Some(chat) {
        return;
    }
    let account = active(app);
    let title = app.chat_title_of(chat);
    let key = chat_key(&account, chat);
    let rewrite = match tool {
        "friendlier" => Some((prompt::Rewrite::Friendlier, "friendlier")),
        "shorter" => Some((prompt::Rewrite::Shorter, "shorter")),
        "formal" => Some((prompt::Rewrite::Formal, "more formal")),
        "spelling" => Some((prompt::Rewrite::FixGrammar, "free of spelling mistakes")),
        _ => None,
    };
    match tool {
        "suggest" => {
            asked_line(app, format!("Suggest a reply to {title}"));
            app.znga.asked.insert(key.clone());
            app.znga.composer_asks.insert(key);
            hooks::request_draft(app, &account, chat, DraftKind::Suggestion);
        }
        "catchUp" => {
            asked_line(app, format!("Catch me up on {title}"));
            hooks::request_catch_up_for(app, chat);
        }
        "plans" => {
            asked_line(app, format!("Find plans in {title}"));
            hooks::request_events_for(app, chat);
        }
        "translate" if !text.trim().is_empty() && !language.trim().is_empty() => {
            asked_line(
                app,
                format!("Translate my message to {title} into {}", language.trim()),
            );
            hooks::request_rewrite(app, prompt::translate(text, language.trim()));
        }
        _ => match rewrite {
            Some((how, label)) if !text.trim().is_empty() => {
                asked_line(app, format!("Make my message to {title} {label}"));
                hooks::request_rewrite(app, prompt::rewrite(text, how));
            }
            _ => return,
        },
    }
    say_status_error(app);
}

// ---------------------------------------------------------------------
// Answers arriving, and what Znga does on its own (REQ-ZCHAT-002, 005).

/// What was there before an answer was handled.
pub struct Before {
    max_draft: u64,
}

pub fn before(app: &App) -> Before {
    Before {
        max_draft: app
            .znga
            .drafts
            .items()
            .iter()
            .map(|d| d.id)
            .max()
            .unwrap_or(0),
    }
}

/// Posts what an answer produced, after [`super::Znga::handle`] stored it.
pub fn after(app: &mut App, purpose: &Purpose, result: &Result<String, String>, before: Before) {
    if !enabled(app) {
        return;
    }
    let new: Vec<Draft> = app
        .znga
        .drafts
        .items()
        .iter()
        .filter(|d| d.id > before.max_draft)
        .cloned()
        .collect();
    for draft in new {
        place_draft(app, draft);
    }
    let account = active(app);
    match (purpose, result) {
        (Purpose::Briefing { proactive: true }, Ok(text)) => unasked(
            app,
            &account,
            Kind::WelcomeBack,
            "Welcome back. Here is what happened while you were away.".into(),
            vec![Item::new(Card::Briefing { text: text.clone() })],
        ),
        (Purpose::Briefing { proactive: false }, Ok(text)) => answer(
            app,
            &account,
            "Your briefing:",
            vec![Item::new(Card::Briefing { text: text.clone() })],
        ),
        (Purpose::Digest { slot }, Ok(text)) => {
            let name = if *slot == "morning" {
                "Morning"
            } else {
                "Evening"
            };
            unasked(
                app,
                &account,
                Kind::Digest,
                format!("{name} digest"),
                vec![Item::new(Card::Briefing { text: text.clone() })],
            );
        }
        (
            Purpose::CatchUp {
                account: on,
                chat,
                chat_title,
            },
            Ok(_),
        ) => {
            let summary = app
                .znga
                .catch_ups
                .iter()
                .find(|c| c.account == *on && c.chat == *chat)
                .map(|c| c.summary.clone())
                .unwrap_or_default();
            answer(
                app,
                on,
                format!("Catch-up on {chat_title}:\n{summary}"),
                vec![Item::new(Card::CatchUp {
                    chat: chat.clone(),
                    chat_title: chat_title.clone(),
                })],
            );
        }
        (
            Purpose::Events {
                account: on,
                chat_title,
                ..
            },
            Ok(_),
        ) => {
            let cards: Vec<Item> = app
                .znga
                .events
                .iter()
                .filter(|e| e.chat_title == *chat_title)
                .map(|e| plan_card(&e.chat_title, &e.event))
                .collect();
            let text = if cards.is_empty() {
                format!("No plans with a date found in {chat_title}.")
            } else {
                format!("Plans found in {chat_title}:")
            };
            answer(app, on, text, cards);
        }
        (Purpose::Voice { note }, Ok(_)) => {
            app.znga.asked.remove(&format!("voice:{note}"));
            let found = app.znga.voice_notes.iter().find(|n| n.id == *note).cloned();
            if let Some(note) = found
                && let VoiceState::Done {
                    transcript,
                    summary,
                } = &note.state
            {
                answer(
                    app,
                    &note.account,
                    format!(
                        "{}'s voice note in {}:\n{summary}\n\nTranscript:\n{transcript}",
                        note.sender_name, note.chat_title
                    ),
                    Vec::new(),
                );
            }
        }
        (purpose, Err(error)) if asked_purpose(app, purpose) => {
            if let Purpose::Draft { account, chat, .. } = purpose {
                app.znga.asked.remove(&chat_key(account, chat));
                app.znga.composer_asks.remove(&chat_key(account, chat));
            }
            if let Purpose::Voice { note } = purpose {
                app.znga.asked.remove(&format!("voice:{note}"));
            }
            let on = match purpose {
                Purpose::Draft { account, .. }
                | Purpose::CatchUp { account, .. }
                | Purpose::Events { account, .. } => account.clone(),
                _ => account.clone(),
            };
            let text = thread::explain_error(error, app.znga.config.engine);
            answer(app, &on, text, Vec::new());
        }
        _ => {}
    }
    // The chat says what the panel's status line said.
    if app.znga.status.as_ref().is_some_and(|s| !s.error) {
        app.znga.status = None;
    }
    if matches!(purpose, Purpose::Draft { .. } | Purpose::Events { .. }) {
        app.znga.status = None;
    }
}

fn asked_purpose(app: &App, purpose: &Purpose) -> bool {
    match purpose {
        Purpose::Draft {
            kind,
            account,
            chat,
            ..
        } => {
            !matches!(kind, DraftKind::AutoReply | DraftKind::CheckIn)
                || app.znga.asked.contains(&chat_key(account, chat))
        }
        Purpose::CatchUp { .. }
        | Purpose::Events { .. }
        | Purpose::FanOut { .. }
        | Purpose::Voice { .. }
        | Purpose::Briefing { proactive: false } => true,
        _ => false,
    }
}

fn plan_card(chat_title: &str, event: &calendar::Event) -> Item {
    Item::new(Card::Plan {
        chat_title: chat_title.to_owned(),
        title: event.title.clone(),
        start: event.start,
        end: event.end,
        place: event.place.clone(),
        notes: event.notes.clone(),
    })
}

fn place_draft(app: &mut App, draft: Draft) {
    let key = chat_key(&draft.account, &draft.chat);
    let title = draft.chat_title.clone();
    if draft.kind == DraftKind::Rewrite {
        app.znga.drafts.take(draft.id);
        offer(app, &draft.chat, &draft.text, true);
        answer(app, &draft.account, draft.text, Vec::new());
        return;
    }
    if app.znga.composer_asks.remove(&key) {
        offer(app, &draft.chat, &draft.text, false);
    }
    let card = Item::new(Card::Draft {
        draft: draft.id,
        chat: draft.chat.clone(),
        chat_title: title.clone(),
        text: draft.text.clone(),
        label: draft.kind.label().to_owned(),
    });
    if app.znga.asked.remove(&key) || draft.kind == DraftKind::Update {
        answer(
            app,
            &draft.account,
            format!("A draft for {title}:"),
            vec![card],
        );
        return;
    }
    let (kind, line) = match draft.kind {
        DraftKind::AutoReply => (Kind::NeedsYou, format!("I drafted a reply to {title}.")),
        DraftKind::CheckIn => (
            Kind::Reminder,
            format!("It has been a while since you talked to {title}. A check-in?"),
        ),
        _ => (Kind::Reminder, format!("A reply for {title}.")),
    };
    unasked(app, &draft.account, kind, line, vec![card]);
}

/// A message that needs the person, posted at once with no AI call
/// (REQ-ZCHAT-005.A1).
pub fn needs_you(app: &mut App, alert: &super::proactive::Alert) {
    unasked(
        app,
        &alert.account.clone(),
        Kind::NeedsYou,
        format!("{} needs you: {}", alert.chat_title, alert.reason),
        vec![Item::new(Card::NeedsYou {
            chat: alert.chat.clone(),
            chat_title: alert.chat_title.clone(),
            sender: alert.sender_name.clone(),
            text: alert.text.clone(),
            reason: alert.reason.clone(),
        })],
    );
}

/// Long voice notes get a Summarise button.
pub fn voice_note(app: &mut App, note: &super::VoiceNote) {
    let Some(seconds) = note.seconds.filter(|s| *s >= 30) else {
        return;
    };
    unasked(
        app,
        &note.account.clone(),
        Kind::Reminder,
        format!(
            "{} sent a {}-second voice note in {}.",
            note.sender_name, seconds, note.chat_title
        ),
        vec![Item::new(Card::Voice {
            note: note.id,
            chat: note.chat.clone(),
            chat_title: note.chat_title.clone(),
            sender: note.sender_name.clone(),
            seconds,
        })],
    );
}

/// A promise coming due (REQ-ZCHAT-005 reminders).
pub fn promise_due(app: &mut App, promise: &super::promises::Promise) {
    let line = match (promise.owes, promise.chat_title.is_empty()) {
        (_, true) => format!("Reminder: {}", promise.text),
        (Owes::Me, false) => format!("You promised {}: {}", promise.chat_title, promise.text),
        (Owes::Them, false) => {
            format!("{} is waiting on you: {}", promise.chat_title, promise.text)
        }
    };
    unasked(
        app,
        &promise.account.clone(),
        Kind::Reminder,
        line,
        vec![Item::new(Card::Promise {
            chat: promise.chat.clone(),
            chat_title: promise.chat_title.clone(),
            text: promise.text.clone(),
            owes_me: promise.owes == Owes::Me,
        })],
    );
}

/// Close friends drifting away, once while their card is open.
pub fn rhythms(app: &mut App, account: &str) {
    let drifting: Vec<super::coach::Rhythm> = app
        .znga
        .rhythms
        .iter()
        .filter(|r| r.account == account && r.drifting)
        .cloned()
        .collect();
    for rhythm in drifting {
        let open = app.znga.threads.get(account).is_some_and(|t| {
            t.entries().iter().any(|e| {
                e.items.iter().any(|i| {
                    i.outcome.is_none()
                        && matches!(&i.card, Card::Drift { chat, .. } if *chat == rhythm.chat)
                })
            })
        });
        if open {
            continue;
        }
        unasked(
            app,
            account,
            Kind::Reminder,
            format!("{} has gone quiet.", rhythm.chat_title),
            vec![Item::new(Card::Drift {
                chat: rhythm.chat.clone(),
                chat_title: rhythm.chat_title.clone(),
                reason: rhythm.reason(),
            })],
        );
    }
}

/// An account's saved Znga chat arrived. With none saved yet, the earlier
/// Ask thread becomes its start (REQ-ZCHAT-002.A4).
pub fn loaded(
    app: &mut App,
    account: &str,
    saved: Vec<thread::Entry>,
    ask: &[super::provider::Turn],
) {
    let saved = if saved.is_empty() && !ask.is_empty() {
        Thread::from_ask(ask, crate::util::now())
    } else {
        saved
    };
    thread_of(app, account).restore(saved);
}

const DIGEST_CHECK: std::time::Duration = std::time::Duration::from_secs(30);
const PLAN_SOON: i64 = 2 * 3600;

/// Once a frame from [`hooks::tick`].
pub fn tick(app: &mut App) {
    if !app.znga.chat_mode {
        return;
    }
    let account = active(app);
    if app.znga.config.enabled {
        // Each account's Znga chat, loaded once.
        let wanted: Vec<(String, std::path::PathBuf)> = app
            .accounts
            .iter()
            .map(|a| (a.id.as_str().to_owned(), a.dirs.archive_db()))
            .filter(|(id, archive)| {
                !app.znga.restored.contains(id)
                    && !app.znga.loaded.contains(id)
                    && !app.znga.thread_requested.contains(id)
                    && archive.is_file()
            })
            .collect();
        for (id, archive) in wanted {
            app.znga.thread_requested.insert(id.clone());
            app.znga.store.send(super::store::Job::Load {
                account: id,
                archive,
            });
        }
    }
    if reading(app) {
        thread_of(app, &account).mark_read();
    }
    let config = app.znga.config.clone();
    if config.enabled {
        let quiet = quiet(&config);
        let now = crate::util::now();
        let reading_here = reading(app);
        let accounts: Vec<String> = app.znga.threads.keys().cloned().collect();
        for id in accounts {
            let here = reading_here && id == account;
            let posted =
                thread_of(app, &id).release_held(now, here, quiet, |k| kind_on(&config, k));
            if matches!(posted, Some(Posted::New(_))) && !here {
                notify(
                    app,
                    &id,
                    "Znga has messages that waited for quiet hours to end.",
                );
            }
        }
        for thread in app.znga.threads.values_mut() {
            thread.drop_held(|k| kind_on(&config, k));
        }
        digests(app);
        plan_reminders(app);
    }
    // Saved once the account's znga.db was read, so it is never
    // overwritten unread.
    let dirty: Vec<String> = app
        .znga
        .threads
        .iter_mut()
        .filter(|(id, _)| app.znga.restored.contains(*id))
        .filter_map(|(id, thread)| thread.take_dirty().then(|| id.clone()))
        .collect();
    for id in dirty {
        let Some(archive) = app
            .accounts
            .iter()
            .find(|a| a.id.as_str() == id)
            .map(|a| a.dirs.archive_db())
        else {
            continue;
        };
        let entries = app.znga.threads[&id].entries().to_vec();
        app.znga
            .store
            .send(super::store::Job::SaveThread { archive, entries });
    }
}

fn digests(app: &mut App) {
    let instant = std::time::Instant::now();
    if !app.znga.config.digests
        || app
            .znga
            .last_digest_check
            .is_some_and(|last| instant.duration_since(last) < DIGEST_CHECK)
    {
        return;
    }
    app.znga.last_digest_check = Some(instant);
    let local = local_now();
    let today = day_of(&local);
    let config = &app.znga.config;
    let last = |slot: &str| config.digest_last.get(slot).copied().unwrap_or(i64::MIN);
    let mut slots = Vec::new();
    if let Some(at) = thread::parse_hhmm(&config.digest_morning) {
        slots.push(("morning", at, last("morning")));
    }
    if let Some(at) = thread::parse_hhmm(&config.digest_evening) {
        slots.push(("evening", at, last("evening")));
    }
    let Some((slot, ran)) = thread::digest_due(today, minute_of(&local), &slots) else {
        return;
    };
    for done in ran {
        app.znga.config.digest_last.insert(done.to_owned(), today);
    }
    app.znga.mark_dirty();
    let now = crate::util::now();
    let config = app.znga.config.clone();
    let items: Vec<_> = app
        .znga
        .activity
        .recent(300)
        .into_iter()
        .filter(|seen| now - seen.at <= 12 * 3600 && config.persona(&seen.account).briefings)
        .collect();
    let account = active(app);
    if items.is_empty() {
        let name = if slot == "morning" {
            "Morning"
        } else {
            "Evening"
        };
        unasked(
            app,
            &account,
            Kind::Digest,
            format!("{name} digest: nothing new in your chats."),
            Vec::new(),
        );
        return;
    }
    let prompt = prompt::briefing(&hooks::your_name(app, &account), &items);
    if let Err(error) = app.znga.submit(prompt, Purpose::Digest { slot }) {
        log::debug!(target: "zapfast::znga", "digest skipped: {}", error.len());
    }
}

fn plan_reminders(app: &mut App) {
    if !app.znga.config.reminders {
        return;
    }
    let now = crate::util::now();
    let soon: Vec<super::FoundEvent> = app
        .znga
        .events
        .iter()
        .filter(|e| e.event.start > now && e.event.start - now <= PLAN_SOON)
        .filter(|e| {
            !app.znga
                .plans_reminded
                .contains(&(e.event.title.clone(), e.event.start))
        })
        .cloned()
        .collect();
    let account = active(app);
    for found in soon {
        app.znga
            .plans_reminded
            .insert((found.event.title.clone(), found.event.start));
        let at = jiff::Timestamp::from_second(found.event.start)
            .map(|t| {
                t.to_zoned(jiff::tz::TimeZone::system())
                    .strftime("%H:%M")
                    .to_string()
            })
            .unwrap_or_default();
        unasked(
            app,
            &account,
            Kind::Reminder,
            format!(
                "Coming up at {at}: {} (from {}).",
                found.event.title, found.chat_title
            ),
            vec![plan_card(&found.chat_title, &found.event)],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::model::Chat;
    use crate::paths::AppDirs;
    use crate::settings::Settings;

    fn chat(id: &str, name: &str) -> Chat {
        Chat::new(id.to_owned(), name.to_owned())
    }

    /// A headless app in chat mode with Ada, two Sams and a locked chat;
    /// every model answer is `reply`.
    fn app(reply: &'static str) -> (tempfile::TempDir, App) {
        let root = tempfile::tempdir().unwrap();
        let (mut app, _events) = App::headless(AppDirs::under(root.path()), Settings::default());
        app.znga.chat_mode = true;
        app.znga.config.enabled = true;
        app.znga.runner.fake = Some(Box::new(move |_| Ok(reply.to_owned())));
        let mut locked = chat("locked@s.whatsapp.net", "Secret Sam");
        locked.locked = true;
        app.account_mut().chats = vec![
            chat("ada@s.whatsapp.net", "Ada Lovelace"),
            chat("sam1@s.whatsapp.net", "Sam Lee"),
            chat("sam2@s.whatsapp.net", "Sam Patel"),
            chat("kim@s.whatsapp.net", "Kim"),
            locked,
        ];
        app.znga
            .config
            .excluded_chats
            .insert(chat_key(app.account().id.as_str(), "kim@s.whatsapp.net"));
        (root, app)
    }

    fn sends(app: &App) -> usize {
        let is_send = |a: &Action| matches!(a, Action::SendText { .. });
        app.actions.iter().filter(|a| is_send(a)).count()
            + app
                .deferred_account_actions
                .iter()
                .filter(|(_, a)| is_send(a))
                .count()
    }

    fn texts(app: &App) -> Vec<String> {
        app.znga.threads[app.account().id.as_str()]
            .entries()
            .iter()
            .map(|e| e.text.clone())
            .collect()
    }

    fn last_card(app: &App) -> (u64, Card) {
        let entry = app.znga.threads[app.account().id.as_str()]
            .entries()
            .iter()
            .rev()
            .find(|e| !e.items.is_empty())
            .unwrap();
        (entry.id, entry.items[0].card.clone())
    }

    #[test]
    fn tell_makes_a_draft_and_sends_only_on_the_click() {
        let (_root, mut app) = app("I'll call you at 5!");
        send_message(&mut app, "Tell Ada I'll call at 5");
        hooks::tick(&mut app);
        let (entry, card) = last_card(&app);
        assert!(matches!(&card, Card::Draft { chat, text, .. }
            if chat == "ada@s.whatsapp.net" && text == "I'll call you at 5!"));
        assert_eq!(sends(&app), 0);
        card_action(&mut app, entry, 0, "send");
        assert_eq!(sends(&app), 1);
        card_action(&mut app, entry, 0, "send");
        assert_eq!(sends(&app), 1, "a second Send does nothing");
        let thread = &app.znga.threads[app.account().id.as_str()];
        let item = &thread
            .entries()
            .iter()
            .find(|e| e.id == entry)
            .unwrap()
            .items[0];
        assert_eq!(item.outcome.as_deref(), Some("Sent"));
    }

    #[test]
    fn an_ambiguous_name_is_asked_about() {
        let (_root, mut app) = app("Running late, sorry!");
        send_message(&mut app, "Tell Sam I'm late");
        hooks::tick(&mut app);
        let last = texts(&app).last().cloned().unwrap();
        assert!(last.contains("Sam Lee") && last.contains("Sam Patel"));
        assert!(!last.contains("Secret Sam"));
        assert!(app.znga.drafts.is_empty());
        send_message(&mut app, "2");
        hooks::tick(&mut app);
        let (_, card) = last_card(&app);
        assert!(matches!(card, Card::Draft { chat, .. } if chat == "sam2@s.whatsapp.net"));
        assert_eq!(sends(&app), 0);
    }

    #[test]
    fn locked_and_excluded_chats_are_refused_and_logged() {
        let (_root, mut app) = app("hello");
        send_message(&mut app, "Tell Secret Sam hello");
        send_message(&mut app, "summarise Kim");
        hooks::tick(&mut app);
        let refusals = texts(&app)
            .iter()
            .filter(|t| t.contains("locked or excluded"))
            .count();
        assert_eq!(refusals, 2);
        let refused = app
            .znga
            .audit
            .entries()
            .filter(|e| !e.allowed && e.action.contains("locked or excluded"))
            .count();
        assert_eq!(refused, 2);
        assert!(app.znga.drafts.is_empty());
        for entry in app.znga.audit.entries() {
            assert!(!entry.action.contains("hello") && !entry.reason.contains("hello"));
        }
    }

    #[test]
    fn a_hundred_instructions_send_nothing() {
        let (_root, mut app) = app("Sure thing");
        let instructions = [
            "Tell Ada I'll call at 5",
            "tell Sam Lee I'm on my way",
            "message Ada: send me the file",
            "remind me to pay Sam Patel on Friday",
            "summarise Ada Lovelace",
            "done",
            "send this to everyone",
            "Do I owe anyone a reply?",
            "Tell Sam I'm late",
            "1",
        ];
        for n in 0..100 {
            send_message(&mut app, instructions[n % instructions.len()]);
            hooks::tick(&mut app);
        }
        assert_eq!(sends(&app), 0);
        assert!(!app.znga.drafts.is_empty());
    }

    #[test]
    fn a_reminder_becomes_an_open_promise() {
        let (_root, mut app) = app("ok");
        send_message(&mut app, "remind me to pay Sam Patel on Friday");
        let promise = app.znga.promises.all().last().unwrap().clone();
        assert_eq!(promise.chat, "sam2@s.whatsapp.net");
        assert!(promise.due.is_some());
        assert!(texts(&app).last().unwrap().starts_with("I'll remind you"));
    }

    #[test]
    fn questions_are_answered_in_the_chat_and_errors_say_what_to_do() {
        let (_root, mut app) = app("{\"search\": [\"reply\"]}");
        send_message(&mut app, "Do I owe anyone a reply?");
        // The archive search runs on the store thread.
        for _ in 0..300 {
            hooks::tick(&mut app);
            if texts(&app).len() >= 2 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let written = texts(&app);
        assert_eq!(written[0], "Do I owe anyone a reply?");
        assert!(written.len() >= 2);
        let (_other, mut failing) = super::tests::app("x");
        failing.znga.runner.fake = Some(Box::new(|_| {
            Err("Could not connect to 127.0.0.1:11434".into())
        }));
        send_message(&mut failing, "Tell Ada hi");
        hooks::tick(&mut failing);
        assert!(texts(&failing).last().unwrap().contains("Ollama"));
    }

    #[test]
    fn nothing_is_posted_with_znga_off() {
        let (_root, mut app) = app("x");
        app.znga.config.enabled = false;
        send_message(&mut app, "Tell Ada hi");
        let alert = super::super::proactive::Alert {
            account: active(&app),
            chat: "ada@s.whatsapp.net".into(),
            chat_title: "Ada Lovelace".into(),
            sender_name: "Ada".into(),
            text: "call me".into(),
            reason: "a request".into(),
            at: 0,
        };
        needs_you(&mut app, &alert);
        assert!(app.znga.threads.values().all(|t| t.entries().is_empty()));
    }

    #[test]
    fn needs_you_from_a_hidden_chat_is_never_posted() {
        let (_root, mut app) = app("x");
        let mut alert = super::super::proactive::Alert {
            account: active(&app),
            chat: "kim@s.whatsapp.net".into(),
            chat_title: "Kim".into(),
            sender_name: "Kim".into(),
            text: "call me".into(),
            reason: "a request".into(),
            at: 0,
        };
        needs_you(&mut app, &alert);
        alert.chat = "locked@s.whatsapp.net".into();
        needs_you(&mut app, &alert);
        assert!(app.znga.threads.values().all(|t| t.entries().is_empty()));
        alert.chat = "ada@s.whatsapp.net".into();
        needs_you(&mut app, &alert);
        assert_eq!(texts(&app).len(), 1);
        app.znga.config.alerts = false;
        alert.chat = "sam1@s.whatsapp.net".into();
        needs_you(&mut app, &alert);
        assert_eq!(texts(&app).len(), 1, "switched off at once");
        assert!(!texts(&app)[0].contains("Sam"));
    }

    #[test]
    fn suggest_reply_goes_to_the_chat_and_the_composer() {
        let (_root, mut app) = app("See you then!");
        app.open_chat = Some("ada@s.whatsapp.net".into());
        tool(&mut app, "ada@s.whatsapp.net", "suggest", "", "");
        hooks::tick(&mut app);
        let texts = texts(&app);
        assert_eq!(texts[0], "Suggest a reply to Ada Lovelace");
        let (_, card) = last_card(&app);
        assert!(matches!(card, Card::Draft { text, .. } if text == "See you then!"));
        let offer = app.znga.offer.clone().unwrap();
        assert_eq!(offer.chat, "ada@s.whatsapp.net");
        assert_eq!(offer.text, "See you then!");
        assert!(!offer.rewrite);
        assert_eq!(sends(&app), 0);
    }

    #[test]
    fn rewrites_are_offered_and_need_text() {
        let (_root, mut app) = app("Hi there, friend!");
        app.open_chat = Some("ada@s.whatsapp.net".into());
        tool(&mut app, "ada@s.whatsapp.net", "friendlier", "", "");
        assert!(app.znga.threads.values().all(|t| t.entries().is_empty()));
        tool(&mut app, "ada@s.whatsapp.net", "friendlier", "hi", "");
        hooks::tick(&mut app);
        let offer = app.znga.offer.clone().unwrap();
        assert!(offer.rewrite);
        assert_eq!(offer.text, "Hi there, friend!");
        assert!(app.znga.drafts.is_empty());
        assert!(!tools_for(&app, "kim@s.whatsapp.net"));
        assert!(!tools_for(&app, "locked@s.whatsapp.net"));
        assert_eq!(sends(&app), 0);
    }
}
