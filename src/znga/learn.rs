//! The app side of Znga's profiles (specs/znga-profiles.md): counting new
//! messages, the live learning pass, the one-time history read, group
//! activity, and the profile lines drafts and answers use. The profiles
//! themselves are in [`super::profiles`].
//!
//! Only the Tauri interface turns profiles on ([`super::Znga::chat_mode`]),
//! since only it can show, edit and forget them (REQ-ZPROF-007).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::app::App;
use crate::model::{Chat, ChatKind, Content, Message};

use super::Purpose;
use super::hooks::{archive_of, title_of, your_name};
use super::privacy::{ChatFacts, visible};
use super::profiles::{self, BackfillState, Tag, member_key};
use super::prompt::{self, Line};
use super::sentinel::{Entry as AuditEntry, Origin};
use super::store::Job;

/// How often chats with enough new messages are relearned.
const LIVE_SCAN: Duration = Duration::from_secs(5 * 60);
/// At most this many live updates per scan.
const LIVE_PER_SCAN: usize = 3;
/// Messages read for a live update.
const LIVE_MESSAGES: usize = 120;
/// How often groups' active members are recounted.
const ACTIVITY_SCAN: Duration = Duration::from_secs(30 * 60);

/// Profiles are on: Znga on, the switch on, in the Tauri interface.
pub fn on(app: &App) -> bool {
    app.znga.chat_mode && app.znga.config.enabled && app.znga.config.profiles
}

/// Profiles on, and `account`'s `znga.db` read: an account whose file
/// could not be read keeps and learns nothing (REQ-ZPROF-001.A5).
fn on_for(app: &App, account: &str) -> bool {
    on(app) && app.znga.restored.contains(account)
}

fn local_day() -> i64 {
    let zoned = jiff::Timestamp::now().to_zoned(jiff::tz::TimeZone::system());
    (zoned.timestamp().as_second() + i64::from(zoned.offset().seconds())).div_euclid(86_400)
}

fn local_offset() -> i64 {
    let zoned = jiff::Timestamp::now().to_zoned(jiff::tz::TimeZone::system());
    i64::from(zoned.offset().seconds())
}

fn chat_of<'a>(app: &'a App, account: &str, chat: &str) -> Option<&'a Chat> {
    app.accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .and_then(|known| known.chats.iter().find(|c| c.id == chat))
}

/// Whether Znga may learn from or use `id` on `account`: a visible chat,
/// or a group member whose direct chat (if any) is not locked or excluded.
pub fn may_see(app: &App, account: &str, id: &str) -> bool {
    match chat_of(app, account, id) {
        Some(chat) => visible(&app.znga.config, ChatFacts::of(account, chat)),
        None => !app.znga.config.is_excluded(account, id),
    }
}

/// A message arrived (`observe_incoming`): it counts toward the chat's
/// next update.
pub fn observe(app: &mut App, account: &str, chat: &Chat) {
    if !on_for(app, account) || !visible(&app.znga.config, ChatFacts::of(account, chat)) {
        return;
    }
    let title = app.chat_title(chat);
    app.znga
        .profiles
        .seen(account, &chat.id, &title, chat.is_group());
}

/// The profile lines a draft to `chat` uses (REQ-ZPROF-005.A1).
pub fn lines_for(app: &App, account: &str, chat: &str) -> Vec<String> {
    if !on(app) || chat.is_empty() {
        return Vec::new();
    }
    app.znga
        .profiles
        .get(account, chat)
        .map(|p| p.prompt_lines())
        .unwrap_or_default()
}

/// Profiles a question names ("what is the school group about?"), for the
/// Znga chat's answer (REQ-ZPROF-005.A2): visible ones only.
pub fn named_in(app: &App, account: &str, question: &str) -> Vec<(String, Vec<String>)> {
    if !on(app) {
        return Vec::new();
    }
    let asked = format!(" {} ", super::thread::fold(question));
    app.znga
        .profiles
        .for_account(account)
        .into_iter()
        .filter(|p| {
            let title = super::thread::fold(&p.title);
            let first = title
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_owned();
            title.chars().count() >= 3
                && (asked.contains(&format!(" {title} "))
                    || (!p.group
                        && first.chars().count() >= 3
                        && asked.contains(&format!(" {first} "))))
        })
        .filter(|p| may_see(app, account, &p.id))
        .take(4)
        .map(|p| (p.title.clone(), p.prompt_lines()))
        .filter(|(_, lines)| !lines.is_empty())
        .collect()
}

/// A group's lines with each member tagged, so their numbers never reach
/// the engine; a contact's as they are.
fn lines_and_tags(app: &App, group: bool, messages: &[Message]) -> (Vec<Line>, Vec<Tag>) {
    let mut tags: Vec<Tag> = Vec::new();
    let mut by_id: HashMap<String, usize> = HashMap::new();
    let mut lines = Vec::new();
    for message in messages {
        if matches!(message.content, Content::Call { .. }) {
            continue;
        }
        let text = app.message_text(message);
        if text.trim().is_empty() {
            continue;
        }
        let name = app.display_name_or(&message.sender, message.sender_name.as_deref());
        let speaker = if group && !message.from_me {
            let id = member_key(&message.sender);
            let index = *by_id.entry(id.clone()).or_insert_with(|| {
                tags.push(Tag {
                    tag: format!("m{}", tags.len() + 1),
                    id,
                    name: name.clone(),
                });
                tags.len() - 1
            });
            format!("{} [{}]", tags[index].name, tags[index].tag)
        } else {
            name
        };
        lines.push(Line {
            speaker,
            text,
            from_me: message.from_me,
        });
    }
    (lines, tags)
}

/// The store sent a chat's messages: update its profile from them.
pub fn history(app: &mut App, account: &str, chat: &str, backfill: bool, messages: Vec<Message>) {
    let Some(found) = chat_of(app, account, chat) else {
        if backfill {
            app.znga.backfill.finished(account, chat);
        }
        return;
    };
    let group = found.is_group();
    let title = app.chat_title(found);
    if !on_for(app, account) || !may_see(app, account, chat) {
        if backfill {
            app.znga.backfill.finished(account, chat);
        }
        return;
    }
    let now = crate::util::now();
    if !group {
        let times: Vec<(i64, bool)> = messages.iter().map(|m| (m.timestamp, m.from_me)).collect();
        if let Some(replies) = profiles::reply_time(&times) {
            app.znga
                .profiles
                .set_local(account, chat, &title, false, "replies", &replies, now);
        }
    }
    let (lines, tags) = lines_and_tags(app, group, &messages);
    if lines.is_empty() {
        if backfill {
            app.znga
                .profiles
                .mark_backfilled(account, chat, &title, group);
            app.znga.backfill.finished(account, chat);
        }
        return;
    }
    if !app.znga.config.spend_profile_call(local_day()) {
        app.znga.mark_dirty();
        if backfill {
            // Today's calls are spent: this chat is first tomorrow
            // (REQ-ZPROF-002.A2).
            app.znga.backfill.current = None;
            app.znga
                .backfill
                .queue
                .push_front((account.to_owned(), chat.to_owned(), title, group));
            app.znga.backfill.state = BackfillState::Waiting(
                "Today's profile updates are used up; it continues tomorrow.".into(),
            );
        } else {
            app.znga.profiles.retry(account, chat, group);
        }
        return;
    }
    app.znga.mark_dirty();
    let mut known = app
        .znga
        .profiles
        .get(account, chat)
        .map(|p| p.prompt_lines())
        .unwrap_or_default();
    known.extend(app.znga.people.about(account, chat));
    let prompt = prompt::profile(&your_name(app, account), &title, group, &lines, &known);
    app.znga.profiles.sent(account, chat);
    let purpose = Purpose::Profile {
        account: account.to_owned(),
        chat: chat.to_owned(),
        chat_title: title,
        group,
        tags,
        backfill,
    };
    if app.znga.submit(prompt, purpose).is_err() {
        app.znga.profiles.retry(account, chat, group);
        if backfill {
            app.znga.backfill.finished(account, chat);
        }
    }
}

/// Who wrote in each group in the last 30 days arrived from the store.
pub fn activity(app: &mut App, account: &str, groups: Vec<(String, super::store::Senders)>) {
    if !on_for(app, account) {
        return;
    }
    let now = crate::util::now();
    let offset = local_offset();
    for (group, senders) in groups {
        let Some(chat) = chat_of(app, account, &group) else {
            continue;
        };
        if !visible(&app.znga.config, ChatFacts::of(account, chat)) {
            continue;
        }
        let title = app.chat_title(chat);
        let participants = chat.participants.clone();
        let named: Vec<(String, String, i64)> = senders
            .iter()
            .map(|(sender, name, at)| {
                (
                    sender.clone(),
                    app.display_name_or(sender, name.as_deref()),
                    *at,
                )
            })
            .collect();
        let active = profiles::activity(&named, &participants, now);
        app.znga
            .profiles
            .set_active(account, &group, &title, active);
        let times: Vec<i64> = senders.iter().map(|(_, _, at)| *at).collect();
        if let Some(busy) = profiles::busy_hours(&times, offset) {
            app.znga
                .profiles
                .set_local(account, &group, &title, true, "busy", &busy, now);
        }
    }
}

/// Asks the store for one chat's recent messages.
fn request(app: &mut App, account: &str, chat: &str, backfill: bool) -> bool {
    let Some(archive) = archive_of(app, account).filter(|a| a.is_file()) else {
        return false;
    };
    let now = crate::util::now();
    let (since, limit) = if backfill {
        (
            now - profiles::HISTORY_DAYS * 86_400,
            profiles::HISTORY_MESSAGES,
        )
    } else {
        (0, LIVE_MESSAGES)
    };
    app.znga.store.send(Job::ProfileHistory {
        account: account.to_owned(),
        archive,
        chat: chat.to_owned(),
        since,
        limit,
        backfill,
    });
    true
}

/// Recounts one group's active members now (the info panel opened it).
pub fn count_group(app: &mut App, account: &str, group: &str) {
    if !on(app) {
        return;
    }
    if let Some(archive) = archive_of(app, account).filter(|a| a.is_file()) {
        app.znga.store.send(Job::GroupActivity {
            account: account.to_owned(),
            archive,
            groups: vec![group.to_owned()],
            since: crate::util::now() - profiles::ACTIVE_DAYS * 86_400,
        });
    }
}

/// Refresh, or "Learn from history", on one profile (REQ-ZPROF-004,
/// REQ-ZPROF-007).
pub fn refresh(app: &mut App, account: &str, chat: &str, history: bool) {
    if !on(app) || !may_see(app, account, chat) || chat_of(app, account, chat).is_none() {
        return;
    }
    app.znga.audit.log(AuditEntry {
        at: crate::util::now(),
        action: if history {
            "learn a profile from history".into()
        } else {
            "refresh a profile".into()
        },
        origin: Origin::Person,
        allowed: true,
        reason: "you asked".into(),
    });
    if history {
        app.znga.profiles.reset_backfill(account, chat);
    }
    request(app, account, chat, history);
    if chat_of(app, account, chat).is_some_and(|c| c.is_group()) {
        count_group(app, account, chat);
    }
}

/// Every visible chat of the restored accounts not read yet, newest first.
fn backfill_queue(app: &App) -> Vec<(String, String, String, bool)> {
    let mut chats: Vec<(i64, (String, String, String, bool))> = Vec::new();
    for known in &app.accounts {
        let account = known.id.as_str();
        if !app.znga.restored.contains(account) {
            continue;
        }
        for chat in &known.chats {
            if chat.kind == ChatKind::Broadcast
                || !visible(&app.znga.config, ChatFacts::of(account, chat))
                || app.znga.profiles.backfilled(account, &chat.id)
                || super::proactive::own_chat(known.me.as_deref().unwrap_or_default())
                    .is_some_and(|own| own == chat.id)
            {
                continue;
            }
            chats.push((
                chat.last_activity,
                (
                    account.to_owned(),
                    chat.id.clone(),
                    app.chat_title(chat),
                    chat.is_group(),
                ),
            ));
        }
    }
    chats.sort_by_key(|(at, _)| std::cmp::Reverse(*at));
    chats.into_iter().map(|(_, chat)| chat).collect()
}

/// Starts the one-time history read (turning profiles on, or Start in
/// Settings › Znga › People).
pub fn start_backfill(app: &mut App) {
    if !on(app) {
        return;
    }
    let queue = backfill_queue(app);
    app.znga.backfill.start(queue);
    app.znga.config.backfill_running = true;
    app.znga.mark_dirty();
    app.znga.audit.log(AuditEntry {
        at: crate::util::now(),
        action: "learn profiles from history".into(),
        origin: Origin::Person,
        allowed: true,
        reason: "profiles were turned on".into(),
    });
}

pub fn pause_backfill(app: &mut App, paused: bool) {
    let backfill = &mut app.znga.backfill;
    match (paused, &backfill.state) {
        (true, BackfillState::Running | BackfillState::Waiting(_)) => {
            backfill.state = BackfillState::Paused;
        }
        (false, BackfillState::Paused) => backfill.state = BackfillState::Running,
        _ => {}
    }
}

/// Cancel keeps what was learned (REQ-ZPROF-002.A3).
pub fn cancel_backfill(app: &mut App) {
    app.znga.backfill.cancel();
    app.znga.config.backfill_running = false;
    app.znga.mark_dirty();
}

/// Forget all (REQ-ZPROF-007.A2): the backfill does not restart until the
/// person runs it again.
pub fn forget_all(app: &mut App, account: &str) {
    app.znga.profiles.forget_all(account);
    app.znga.backfill.queue.retain(|(a, _, _, _)| a != account);
    if app.znga.backfill.queue.is_empty() && app.znga.backfill.current.is_none() {
        cancel_backfill(app);
    }
}

/// Why the backfill cannot run now, if it cannot (REQ-ZPROF-002.A1).
fn waiting_reason(app: &App) -> Option<String> {
    let config = &app.znga.config;
    if config.is_cloud() && !config.cloud_consent {
        return Some(
            "Waiting for \u{201c}Let this engine read my messages\u{201d} in Settings \u{203a} Znga."
                .into(),
        );
    }
    if let Some(reason) = app.znga.blocked() {
        return Some(reason.to_owned());
    }
    if config.profile_calls_day == local_day() && config.profile_calls >= profiles::DAILY_CALLS {
        return Some("Today's profile updates are used up; it continues tomorrow.".into());
    }
    None
}

fn step_backfill(app: &mut App) {
    // Resumes after a restart once every account's profiles are read.
    if app.znga.config.backfill_running && app.znga.backfill.state == BackfillState::Idle {
        let all_read = app.accounts.iter().all(|known| {
            app.znga.restored.contains(known.id.as_str()) || !known.dirs.archive_db().is_file()
        });
        if all_read {
            let queue = backfill_queue(app);
            app.znga.backfill.start(queue);
        }
        return;
    }
    if app.znga.backfill.state == BackfillState::Done {
        if app.znga.config.backfill_running {
            app.znga.config.backfill_running = false;
            app.znga.mark_dirty();
        }
        return;
    }
    if !app.znga.backfill.active() || app.znga.backfill.current.is_some() {
        return;
    }
    if let Some(reason) = waiting_reason(app) {
        app.znga.backfill.state = BackfillState::Waiting(reason);
        return;
    }
    app.znga.backfill.state = BackfillState::Running;
    while let Some((account, chat, title, group)) = app.znga.backfill.queue.pop_front() {
        if !may_see(app, &account, &chat) || app.znga.profiles.backfilled(&account, &chat) {
            app.znga.backfill.done += 1;
            continue;
        }
        app.znga.backfill.current = Some((account.clone(), chat.clone()));
        if !request(app, &account, &chat, true) {
            app.znga
                .profiles
                .mark_backfilled(&account, &chat, &title, group);
            app.znga.backfill.finished(&account, &chat);
            continue;
        }
        return;
    }
    if app.znga.backfill.current.is_none() {
        app.znga.backfill.state = BackfillState::Done;
    }
}

/// Live updates: chats with enough new messages (REQ-ZPROF-002), groups
/// after 50 messages or 7 days (REQ-ZPROF-004.A3).
fn scan_live(app: &mut App) {
    let now = crate::util::now();
    let mut due: Vec<(i64, String, String)> = Vec::new();
    for known in &app.accounts {
        let account = known.id.as_str();
        if !app.znga.restored.contains(account) {
            continue;
        }
        for chat in &known.chats {
            if !visible(&app.znga.config, ChatFacts::of(account, chat)) {
                continue;
            }
            if app
                .znga
                .profiles
                .due(account, &chat.id, chat.is_group(), now)
            {
                due.push((chat.last_activity, account.to_owned(), chat.id.clone()));
            }
        }
    }
    due.sort_by_key(|(at, _, _)| std::cmp::Reverse(*at));
    for (_, account, chat) in due.into_iter().take(LIVE_PER_SCAN) {
        let busy = app.znga.waiting_for(|p| {
            matches!(p, Purpose::Profile { account: a, chat: c, .. } if *a == account && *c == chat)
        });
        if !busy {
            app.znga.profiles.sent(&account, &chat);
            request(app, &account, &chat, false);
        }
    }
}

fn scan_activity(app: &mut App) {
    let since = crate::util::now() - profiles::ACTIVE_DAYS * 86_400;
    let accounts: Vec<String> = app
        .accounts
        .iter()
        .map(|a| a.id.as_str().to_owned())
        .collect();
    for account in accounts {
        let groups: Vec<String> = chat_groups(app, &account);
        if groups.is_empty() {
            continue;
        }
        if let Some(archive) = archive_of(app, &account).filter(|a| a.is_file()) {
            app.znga.store.send(Job::GroupActivity {
                account,
                archive,
                groups,
                since,
            });
        }
    }
}

fn chat_groups(app: &App, account: &str) -> Vec<String> {
    app.accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .map(|known| {
            known
                .chats
                .iter()
                .filter(|c| c.is_group() && visible(&app.znga.config, ChatFacts::of(account, c)))
                .map(|c| c.id.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// Hands changed profiles to the store, per account, once each account's
/// `znga.db` was read (REQ-ZPROF-001.A5).
pub fn save(app: &mut App) {
    if !app.znga.profiles.take_dirty() {
        return;
    }
    let accounts: Vec<String> = app
        .accounts
        .iter()
        .map(|a| a.id.as_str().to_owned())
        .collect();
    for account in accounts {
        if !app.znga.restored.contains(&account) {
            continue;
        }
        let Some(archive) = archive_of(app, &account) else {
            continue;
        };
        let list = app
            .znga
            .profiles
            .all()
            .iter()
            .filter(|p| p.account == account)
            .cloned()
            .collect();
        app.znga.store.send(Job::SaveProfiles {
            archive,
            profiles: list,
        });
    }
}

/// Once a frame from [`super::hooks::tick`], with Znga on.
pub fn tick(app: &mut App) {
    if !on(app) {
        return;
    }
    step_backfill(app);
    let instant = Instant::now();
    if app.znga.blocked().is_none()
        && app
            .znga
            .last_profile_scan
            .is_none_or(|last| instant.duration_since(last) >= LIVE_SCAN)
    {
        app.znga.last_profile_scan = Some(instant);
        scan_live(app);
    }
    if app
        .znga
        .last_activity_scan
        .is_none_or(|last| instant.duration_since(last) >= ACTIVITY_SCAN)
        && !app.znga.restored.is_empty()
    {
        app.znga.last_activity_scan = Some(instant);
        scan_activity(app);
    }
}

/// The relationship to name in a Needs you line ("your manager").
pub fn relation(app: &App, account: &str, chat: &str) -> Option<String> {
    if !on(app) {
        return None;
    }
    app.znga.profiles.get(account, chat)?.relation()
}

/// The visible direct chats of `account` as (id, title), busiest first,
/// for questions.
pub fn direct_chats(app: &App, account: &str) -> Vec<(String, String)> {
    let mut chats: Vec<(i64, String, String)> = app
        .accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .map(|known| {
            known
                .chats
                .iter()
                .filter(|c| {
                    c.kind == ChatKind::Direct
                        && visible(&app.znga.config, ChatFacts::of(account, c))
                })
                .map(|c| (c.last_activity, c.id.clone(), app.chat_title(c)))
                .collect()
        })
        .unwrap_or_default();
    chats.sort_by_key(|(at, _, _)| std::cmp::Reverse(*at));
    chats
        .into_iter()
        .map(|(_, id, title)| (id, title))
        .collect()
}

/// The title Znga uses for a profile id: the chat's, else the stored one.
pub fn title(app: &App, account: &str, id: &str) -> String {
    match chat_of(app, account, id) {
        Some(_) => title_of(app, account, id),
        None => app
            .znga
            .profiles
            .get(account, id)
            .map(|p| p.title.clone())
            .unwrap_or_else(|| id.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::model::Chat;
    use crate::paths::AppDirs;
    use crate::settings::Settings;
    use crate::znga::archive_ask::tests_message;
    use crate::znga::{chat as zchat, hooks};

    const ADA: &str = "ada@s.whatsapp.net";
    const GROUP: &str = "1203@g.us";

    /// A headless app in chat mode with profiles on; every model answer is
    /// `reply`, and the last system prompt is kept in the returned cell.
    fn app(reply: &'static str) -> (tempfile::TempDir, App, Arc<Mutex<String>>) {
        let root = tempfile::tempdir().unwrap();
        let (mut app, _events) = App::headless(AppDirs::under(root.path()), Settings::default());
        app.znga.chat_mode = true;
        app.znga.config.enabled = true;
        app.znga.config.profiles = true;
        let me = app.account().id.as_str().to_owned();
        app.znga.restored.insert(me);
        let seen = Arc::new(Mutex::new(String::new()));
        let keep = seen.clone();
        app.znga.runner.fake = Some(Box::new(move |prompt| {
            let turns: Vec<String> = prompt.turns.iter().map(|t| t.text.clone()).collect();
            *keep.lock().unwrap() = format!("{}\n{}", prompt.system, turns.join("\n"));
            Ok(reply.to_owned())
        }));
        let mut locked = Chat::new("locked@s.whatsapp.net".into(), "Secret Sam".into());
        locked.locked = true;
        let mut group = Chat::new(GROUP.into(), "School parents".into());
        group.participants = vec!["ben@s.whatsapp.net".into()];
        app.account_mut().chats = vec![Chat::new(ADA.into(), "Ada Lovelace".into()), group, locked];
        (root, app, seen)
    }

    fn account(app: &App) -> String {
        app.account().id.as_str().to_owned()
    }

    fn texts(app: &App) -> Vec<String> {
        app.znga.threads[app.account().id.as_str()]
            .entries()
            .iter()
            .map(|e| e.text.clone())
            .collect()
    }

    const CONTACT_ANSWER: &str = r#"{"profile": [
        {"key": "relationship", "value": "family (sister)"},
        {"key": "formality", "value": "casual"},
        {"key": "likes", "value": "jazz"},
        {"key": "notes", "value": "Seeing a therapist", "sensitive": true}
    ]}"#;

    #[test]
    fn a_contact_is_learned_from_its_messages_and_used_in_drafts() {
        let (_root, mut app, seen) = app(CONTACT_ANSWER);
        let me = account(&app);
        let messages = vec![
            tests_message(ADA, "Are we still on for the concert?", 1),
            tests_message(ADA, "I'll bring snacks", 2),
        ];
        history(&mut app, &me, ADA, false, messages);
        hooks::tick(&mut app);
        let profile = app.znga.profiles.get(&me, ADA).expect("a profile");
        assert_eq!(
            profile.first("relationship").unwrap().text,
            "family (sister)"
        );
        // The prompt carried the messages, and never a phone number.
        assert!(seen.lock().unwrap().contains("bring snacks"));
        // A draft to Ada follows her profile; the sensitive value waits
        // for the person (REQ-ZPROF-003.A1, REQ-ZPROF-005.A1).
        let twin = hooks::twin_for(&app, &me, ADA);
        let system = prompt::twin_reply(&twin, "Ada", &[]).system;
        assert!(system.contains("family (sister)") && system.contains("casual"));
        assert!(!system.contains("therapist"));
        app.znga
            .profiles
            .confirm(&me, ADA, "notes", "Seeing a therapist");
        let twin = hooks::twin_for(&app, &me, ADA);
        assert!(
            prompt::twin_reply(&twin, "Ada", &[])
                .system
                .contains("therapist")
        );
    }

    #[test]
    fn a_group_is_learned_with_tagged_members_and_no_numbers() {
        let reply = r#"{"profile": [{"key": "theme", "value": "school pick-ups"},
            {"key": "mood", "value": "positive"}],
            "members": [{"tag": "m1", "mood": "neutral", "items": [{"key": "likes", "value": "cycling"}]}]}"#;
        let (_root, mut app, seen) = app(reply);
        let me = account(&app);
        let mut message = tests_message(GROUP, "Who can drive on Friday?", 1);
        message.sender = "ben:2@s.whatsapp.net".into();
        message.sender_name = Some("Ben".into());
        history(&mut app, &me, GROUP, false, vec![message]);
        hooks::tick(&mut app);
        let sent = seen.lock().unwrap().clone();
        assert!(sent.contains("Ben [m1]"));
        assert!(!sent.contains("ben:2") && !sent.contains("ben@"));
        let group = app.znga.profiles.get(&me, GROUP).unwrap();
        assert_eq!(group.first("theme").unwrap().text, "school pick-ups");
        assert_eq!(group.moods["ben@s.whatsapp.net"].now, "neutral");
        let ben = app.znga.profiles.get(&me, "ben@s.whatsapp.net").unwrap();
        assert_eq!(ben.values("likes")[0].text, "cycling");
    }

    #[test]
    fn active_members_are_counted_without_the_engine() {
        let (_root, mut app, _) = app("not called");
        app.znga.runner.fake = Some(Box::new(|_| panic!("no engine call")));
        let me = account(&app);
        let now = crate::util::now();
        let senders = vec![
            (
                "ben@s.whatsapp.net".to_owned(),
                Some("Ben".to_owned()),
                now - 60,
            ),
            (
                "ben@s.whatsapp.net".to_owned(),
                Some("Ben".to_owned()),
                now - 120,
            ),
            (
                "cy@s.whatsapp.net".to_owned(),
                Some("Cy".to_owned()),
                now - 60,
            ),
        ];
        activity(&mut app, &me, vec![(GROUP.to_owned(), senders)]);
        let group = app.znga.profiles.get(&me, GROUP).unwrap();
        // Named as the app names unsaved senders.
        assert!(group.active[0].name.ends_with("Ben"));
        assert_eq!(group.active[0].count, 2);
        assert!(group.active[1].former, "Cy is not in the group any more");
    }

    #[test]
    fn the_history_read_waits_for_message_reading_with_a_cloud_engine() {
        let (_root, mut app, _) = app("{}");
        let me = account(&app);
        app.znga.config.engine = crate::znga::config::Engine::Claude;
        app.znga.config.cloud_consent = false;
        start_backfill(&mut app);
        tick(&mut app);
        assert!(
            matches!(&app.znga.backfill.state, BackfillState::Waiting(why) if why.contains("read my messages"))
        );
        assert!(app.znga.backfill.current.is_none());
        // Locked chats are never queued (REQ-ZPROF-002.A4).
        assert!(
            app.znga
                .backfill
                .queue
                .iter()
                .all(|(_, chat, _, _)| chat != "locked@s.whatsapp.net")
        );
        assert!(!app.znga.profiles.backfilled(&me, ADA));
        app.znga.config.cloud_consent = true;
        tick(&mut app);
        // Reading on starts it; with no archive here each chat finishes
        // at once.
        assert!(!matches!(
            app.znga.backfill.state,
            BackfillState::Waiting(_)
        ));
        assert!(
            app.znga.profiles.backfilled(&me, ADA),
            "reading on starts it"
        );
        cancel_backfill(&mut app);
        assert_eq!(app.znga.backfill.state, BackfillState::Idle);
        assert!(!app.znga.config.backfill_running);
    }

    #[test]
    fn the_daily_limit_holds_the_rest_for_tomorrow() {
        let (_root, mut app, _) = app(CONTACT_ANSWER);
        let me = account(&app);
        app.znga.config.profile_calls_day = local_day();
        app.znga.config.profile_calls = profiles::DAILY_CALLS;
        app.znga.backfill.state = BackfillState::Running;
        app.znga.backfill.current = Some((me.clone(), ADA.to_owned()));
        history(&mut app, &me, ADA, true, vec![tests_message(ADA, "hi", 1)]);
        assert!(matches!(app.znga.backfill.state, BackfillState::Waiting(_)));
        assert_eq!(
            app.znga.backfill.queue.front().map(|q| q.1.as_str()),
            Some(ADA)
        );
        assert!(!app.znga.profiles.backfilled(&me, ADA));
    }

    #[test]
    fn the_chat_shows_and_forgets_what_znga_knows() {
        let (_root, mut app, _) = app("x");
        let me = account(&app);
        let now = crate::util::now();
        app.znga
            .profiles
            .set(&me, ADA, "Ada Lovelace", false, "likes", "jazz", now);
        zchat::send_message(&mut app, "What do you know about Ada?");
        assert!(texts(&app).last().unwrap().contains("jazz"));
        zchat::send_message(&mut app, "forget that Ada likes jazz");
        assert!(texts(&app).last().unwrap().contains("Forgotten"));
        assert!(
            app.znga
                .profiles
                .get(&me, ADA)
                .unwrap()
                .values("likes")
                .is_empty()
        );
        zchat::send_message(&mut app, "what do you know about Secret Sam");
        assert!(texts(&app).last().unwrap().contains("locked or excluded"));
    }

    #[test]
    fn one_question_a_day_and_the_answer_is_confirmed() {
        let (_root, mut app, _) = app("x");
        let me = account(&app);
        app.znga.profiles.set_local(
            &me,
            ADA,
            "Ada Lovelace",
            false,
            "replies",
            "usually within minutes",
            1,
        );
        // Learned once, so it is worth asking about.
        let answer = profiles::parse(r#"{"profile": [{"key": "likes", "value": "tea"}]}"#).unwrap();
        app.znga
            .profiles
            .apply(&me, ADA, "Ada Lovelace", false, &[], &answer, 1);
        hooks::tick(&mut app);
        let asked = texts(&app)
            .iter()
            .filter(|t| t.contains("who is Ada Lovelace to you"))
            .count();
        assert_eq!(asked, 1);
        hooks::tick(&mut app);
        let asked = texts(&app)
            .iter()
            .filter(|t| t.contains("who is Ada Lovelace to you"))
            .count();
        assert_eq!(asked, 1, "at most one a day");
        zchat::send_message(&mut app, "1");
        let value = app
            .znga
            .profiles
            .get(&me, ADA)
            .unwrap()
            .first("relationship")
            .cloned()
            .unwrap();
        assert_eq!(value.text, "family");
        assert!(value.confirmed);
    }

    #[test]
    fn needs_you_names_why_the_sender_matters() {
        let (_root, mut app, _) = app("x");
        let me = account(&app);
        app.znga.profiles.set(
            &me,
            ADA,
            "Ada Lovelace",
            false,
            "relationship",
            "colleague (manager)",
            1,
        );
        let alert = crate::znga::proactive::Alert {
            account: me.clone(),
            chat: ADA.into(),
            chat_title: "Ada Lovelace".into(),
            sender_name: "Ada".into(),
            text: "call me".into(),
            reason: "a request".into(),
            at: 0,
        };
        zchat::needs_you(&mut app, &alert);
        assert!(texts(&app)[0].starts_with("Ada Lovelace (your manager) needs you"));
    }

    #[test]
    fn an_account_whose_memory_was_not_read_learns_nothing() {
        let (_root, mut app, _) = app(CONTACT_ANSWER);
        let me = account(&app);
        app.znga.restored.clear();
        history(&mut app, &me, ADA, false, vec![tests_message(ADA, "hi", 1)]);
        hooks::tick(&mut app);
        assert!(app.znga.profiles.get(&me, ADA).is_none());
    }

    #[test]
    fn excluding_a_chat_deletes_its_profile() {
        let (_root, mut app, _) = app("x");
        let me = account(&app);
        app.znga
            .profiles
            .set(&me, ADA, "Ada Lovelace", false, "likes", "jazz", 1);
        app.znga
            .config
            .excluded_chats
            .insert(crate::znga::config::chat_key(&me, ADA));
        app.znga.last_scan = None;
        hooks::tick(&mut app);
        assert!(app.znga.profiles.get(&me, ADA).is_none());
    }
}
