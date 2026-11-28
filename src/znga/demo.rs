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
    if part == "znga-translate" {
        translate(app);
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
    // An answer that cites messages and is laid out (specs/znga-answers.md).
    let message_of = |chat: &str, fallback: &str| {
        app.conversations
            .get(chat)
            .and_then(|c| c.messages.iter().rev().find(|m| !m.from_me))
            .map(|m| m.id.clone())
            .unwrap_or_else(|| fallback.to_owned())
    };
    let refs = vec![
        super::answer::Ref::new(
            1,
            &a,
            &a_title,
            &message_of(&a, "demo-a"),
            &a_title,
            now - 86_400 * 3,
            "my email is ada.l@example.com, send the invite there",
        ),
        super::answer::Ref::new(
            2,
            &b,
            &b_title,
            &message_of(&b, "demo-b"),
            &b_title,
            now - 86_400 * 9,
            "bookings go to hello@harbourhouse.example",
        ),
    ];
    thread.post_person("Show my messages that have an email address", now - 40);
    thread.post_cited(
        format!(
            "## Emails people gave you\n\n- **{a_title}** asked you to send the invite to \
             `ada.l@example.com` [1]\n- **{b_title}** shared the venue's booking address, \
             `hello@harbourhouse.example` [2]\n\nWant me to draft the invite to {a_title}?"
        ),
        Vec::new(),
        refs,
        now - 30,
        true,
    );
    thread.unread = 0;
    app.znga.threads.insert(account.clone(), thread);
    app.znga.restored.insert(account);
}

/// A chat with messages in Spanish, one translated in the popup
/// (`--demo-page znga-translate`, specs/znga-answers.md REQ-ZANS-004).
fn translate(app: &mut App) {
    let Some(chat) = app
        .account()
        .chats
        .iter()
        .find(|chat| !chat.locked && !chat.is_group())
        .map(|chat| chat.id.clone())
    else {
        return;
    };
    app.znga.config.enabled = true;
    app.znga.chat_mode = true;
    app.znga.chat_open = false;
    let now = crate::util::now();
    // Shaped like a message the demo already has in that chat.
    let Some(template) = app
        .conversations
        .get(&chat)
        .and_then(|c| c.messages.iter().rev().find(|m| !m.from_me).cloned())
    else {
        return;
    };
    let incoming = |id: &str, text: &str, at: i64| {
        let mut message = template.clone();
        message.content = crate::model::Content::text(text);
        message.reactions = Default::default();
        message.id = id.to_owned();
        message.chat = chat.clone();
        message.sender = chat.clone();
        message.from_me = false;
        message.timestamp = at;
        message
    };
    let first = incoming(
        "demo-es-1",
        "¡Hola! ¿Nos vemos mañana en la casa de mi madre a las ocho?",
        now - 120,
    );
    let second = incoming(
        "demo-es-2",
        "Y no te olvides de traer el pastel, por favor 🎂",
        now - 60,
    );
    let conversation = app.conversations.entry(chat.clone()).or_default();
    conversation.messages.push(first.clone());
    conversation.messages.push(second);
    app.open_chat = Some(chat.clone());
    app.znga.translations.done.insert(
        (chat.clone(), first.id.clone()),
        "Hi! Shall we meet tomorrow at my mother's house at eight?".into(),
    );
    app.znga.translations.popup = Some(super::translate::Popup {
        account: app.account().id.as_str().to_owned(),
        chat,
        messages: vec![super::translate::Shown {
            message: first.id.clone(),
            sender: "Ada Lovelace".into(),
            at: first.timestamp,
            original: "¡Hola! ¿Nos vemos mañana en la casa de mi madre a las ocho?".into(),
        }],
    });
}
