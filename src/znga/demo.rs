//! Synthetic Znga states for demo screenshots (`--demo-page znga`,
//! `znga-drafts`, `znga-ask`, `znga-setup`). Sample content only.

use crate::app::App;

use super::drafts::{Draft, Kind};
use super::proactive::Alert;
use super::provider::Turn;
use super::{Briefing, Tab};

/// Applies a Znga demo flag; `false` when `part` is not one.
pub fn apply(app: &mut App, part: &str) -> bool {
    if part == "znga-chat" {
        chat(app);
        return true;
    }
    let tab = match part {
        "znga" => Tab::Today,
        "znga-drafts" => Tab::Drafts,
        "znga-ask" => Tab::Ask,
        "znga-setup" => Tab::Setup,
        _ => return false,
    };
    let account = app.account().id.as_str().to_owned();
    let chats: Vec<(String, String)> = app
        .account()
        .chats
        .iter()
        .filter(|chat| !chat.locked)
        .take(4)
        .map(|chat| (chat.id.clone(), app.chat_title(chat)))
        .collect();
    let pick = |i: usize| chats.get(i).or(chats.first()).cloned().unwrap_or_default();
    let now = crate::util::now();
    let znga = &mut app.znga;
    znga.config.enabled = true;
    znga.open = true;
    znga.tab = tab;
    let (chat, title) = pick(0);
    znga.alerts.push(Alert {
        account: account.clone(),
        chat: chat.clone(),
        chat_title: title.clone(),
        sender_name: title.clone(),
        text: "Can you call me before 6? It's about Saturday".into(),
        reason: "asks you something, time-sensitive, writing repeatedly".into(),
        at: now - 240,
    });
    znga.briefings.push(Briefing {
        text: format!(
            "WHAT HAPPENED\n• {title} wants to talk about Saturday's plans before 6pm.\n• {} \
             shared the photos from the trip and asked which to print.\n\nNEEDS YOU\n1. Call \
             {title} before 6pm.\n2. Pick the photos for {}.",
            pick(1).1,
            pick(1).1
        ),
        at: now - 600,
        proactive: true,
    });
    znga.drafts.add(Draft {
        id: 0,
        kind: Kind::AutoReply,
        account: account.clone(),
        chat: chat.clone(),
        chat_title: title.clone(),
        text: "yes! give me 10 min, calling you right after this meeting".into(),
        created: now - 200,
    });
    for i in [1, 2] {
        let (chat, title) = pick(i);
        znga.drafts.add(Draft {
            id: 0,
            kind: Kind::Update,
            account: account.clone(),
            chat,
            chat_title: title,
            text: "running about 20 min late, sorry!! save me a seat 🙏".into(),
            created: now - 60,
        });
    }
    znga.ask.push(Turn::user("Do I owe anyone a reply?"));
    znga.ask.push(Turn::assistant(format!(
        "Two people are waiting on you:\n1. {title} asked you to call before 6pm.\n2. {} \
         asked which photos to print.\nWant me to draft replies?",
        pick(1).1
    )));
    znga.unseen = 0;
    true
}

/// The Znga chat with a sample of each kind of message
/// (`--demo-page znga-chat`, specs/znga-chat.md).
fn chat(app: &mut App) {
    use super::thread::{Card, Item, Kind, Thread};
    let account = app.account().id.as_str().to_owned();
    let chats: Vec<(String, String)> = app
        .account()
        .chats
        .iter()
        .filter(|chat| !chat.locked)
        .take(4)
        .map(|chat| (chat.id.clone(), app.chat_title(chat)))
        .collect();
    let pick = |i: usize| chats.get(i).or(chats.first()).cloned().unwrap_or_default();
    let now = crate::util::now();
    app.znga.config.enabled = true;
    app.znga.chat_mode = true;
    app.znga.chat_open = true;
    app.open_chat = None;
    let mut thread = Thread::default();
    let (a, a_title) = pick(0);
    let (b, b_title) = pick(1);
    thread.post_person("Do I owe anyone a reply?", now - 3600);
    thread.post(
        format!("Yes: {a_title} asked about Saturday yesterday and you have not answered yet."),
        Vec::new(),
        now - 3590,
        true,
    );
    thread.post_person(format!("Tell {a_title} I'll call at 5"), now - 3500);
    thread.post(
        format!("A draft for {a_title}:"),
        vec![Item::new(Card::Draft {
            draft: 0,
            chat: a.clone(),
            chat_title: a_title.clone(),
            text: "Hey! I'll call you at 5, promise 😊".into(),
            label: "Update".into(),
        })],
        now - 3490,
        true,
    );
    thread.unasked(
        Kind::NeedsYou,
        format!("{a_title} needs you: asks you something, time-sensitive"),
        vec![Item::new(Card::NeedsYou {
            chat: a.clone(),
            chat_title: a_title.clone(),
            sender: a_title.clone(),
            text: "Can you call me before 6? It's about Saturday".into(),
            reason: "asks you something, time-sensitive".into(),
        })],
        now - 900,
        false,
        false,
    );
    thread.unasked(
        Kind::Reminder,
        format!("You promised {b_title}: send the trip photos"),
        vec![Item::new(Card::Promise {
            chat: b.clone(),
            chat_title: b_title.clone(),
            text: "send the trip photos".into(),
            owes_me: true,
        })],
        now - 600,
        false,
        false,
    );
    thread.unasked(
        Kind::WelcomeBack,
        "Welcome back. Here is what happened while you were away.",
        vec![Item::new(Card::Briefing {
            text: format!(
                "WHAT HAPPENED\n• {a_title} wants to talk about Saturday before 6pm.\n• \
                 {b_title} shared the trip photos.\n\nNEEDS YOU\n1. Call {a_title} before 6pm."
            ),
        })],
        now - 60,
        false,
        false,
    );
    thread.unread = 0;
    app.znga.threads.insert(account.clone(), thread);
    app.znga.restored.insert(account);
}
