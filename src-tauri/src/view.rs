//! What the web interface shows, built from the core's state after each
//! step (REQ-WEBUI-002). The page never holds the state itself.

use serde::Serialize;
use zapfast::app::App;
use zapfast::backend::LinkStatus;
use zapfast::model::{Chat, ChatFilter, Content, Delivery, Message};

/// The most messages of the open chat sent at once.
const WINDOW: usize = 300;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub link: Link,
    pub me: Option<String>,
    pub filter: &'static str,
    pub search: String,
    pub chats: Vec<ChatRow>,
    pub open: Option<Open>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Link {
    Starting,
    Unlinked {
        qr: Option<String>,
        code: Option<String>,
    },
    Connecting,
    Connected,
    Disconnected {
        reason: String,
    },
    LoggedOut,
    Failed {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRow {
    pub id: String,
    pub title: String,
    pub avatar: Option<String>,
    pub group: bool,
    pub time: i64,
    pub unread: u32,
    pub marked_unread: bool,
    pub muted: bool,
    pub pinned: bool,
    pub favorite: bool,
    /// Where the phone puts this favourite; lower first.
    pub favorite_position: u32,
    pub last: Option<Last>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Last {
    pub from_me: bool,
    pub sender: Option<String>,
    pub text: String,
    pub status: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Open {
    pub id: String,
    pub title: String,
    pub avatar: Option<String>,
    pub group: bool,
    pub complete: bool,
    pub messages: Vec<Msg>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Msg {
    pub id: String,
    pub from_me: bool,
    pub sender: Option<String>,
    pub sender_id: String,
    pub time: i64,
    pub kind: String,
    pub text: String,
    pub status: &'static str,
    pub edited: bool,
    pub forwarded: bool,
    pub quoted: Option<Quote>,
    pub reactions: Vec<(String, u32)>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub author: String,
    pub text: String,
}

fn status(delivery: Delivery) -> &'static str {
    match delivery {
        Delivery::None => "none",
        Delivery::Pending => "pending",
        Delivery::Sent => "sent",
        Delivery::Delivered => "delivered",
        Delivery::Read | Delivery::Played => "read",
        Delivery::Failed => "failed",
    }
}

fn filter(filter: ChatFilter) -> &'static str {
    match filter {
        ChatFilter::All => "all",
        ChatFilter::Unread => "unread",
        ChatFilter::Private => "private",
        ChatFilter::Favorites => "favorites",
        ChatFilter::Groups => "groups",
        ChatFilter::Channels => "channels",
    }
}

pub fn parse_filter(name: &str) -> Option<ChatFilter> {
    Some(match name {
        "all" => ChatFilter::All,
        "unread" => ChatFilter::Unread,
        "private" => ChatFilter::Private,
        "favorites" => ChatFilter::Favorites,
        "groups" => ChatFilter::Groups,
        "channels" => ChatFilter::Channels,
        _ => return None,
    })
}

/// A file the page may load, as a `zapmedia:` address.
fn media(path: Option<std::path::PathBuf>) -> Option<String> {
    let path = path?;
    let text = path.to_str()?;
    Some(format!("http://zapmedia.localhost/{}", encode(text)))
}

fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn row(app: &mut App, chat: &Chat, now: i64) -> ChatRow {
    ChatRow {
        id: chat.id.clone(),
        title: app.chat_title(chat),
        avatar: media(app.avatar(&chat.id)),
        group: chat.is_group(),
        time: chat.last_activity,
        unread: chat.unread,
        marked_unread: chat.marked_unread,
        muted: chat.muted(now),
        pinned: chat.pinned,
        favorite: chat.favorite,
        favorite_position: chat.favorite_position,
        last: chat.last.as_ref().map(|last| Last {
            from_me: last.from_me,
            sender: (chat.is_group() && !last.from_me)
                .then(|| last.sender_name.clone().unwrap_or_default())
                .filter(|name| !name.is_empty()),
            text: last.summary.clone(),
            status: status(last.status),
        }),
    }
}

fn message(app: &App, group: bool, message: &Message) -> Msg {
    let kind = serde_json::to_value(&message.content)
        .ok()
        .and_then(|value| value.get("kind")?.as_str().map(str::to_owned))
        .unwrap_or_else(|| "text".into());
    let text = match &message.content {
        Content::Text { text, .. } => text.clone(),
        other => other.summary_in(app.locale),
    };
    let mut reactions: Vec<(String, u32)> = Vec::new();
    for reaction in &message.reactions {
        match reactions
            .iter_mut()
            .find(|(emoji, _)| *emoji == reaction.emoji)
        {
            Some((_, count)) => *count += 1,
            None => reactions.push((reaction.emoji.clone(), 1)),
        }
    }
    Msg {
        id: message.id.clone(),
        from_me: message.from_me,
        sender: (group && !message.from_me).then(|| {
            message
                .sender_name
                .clone()
                .unwrap_or_else(|| message.sender.clone())
        }),
        sender_id: message.sender.clone(),
        time: message.timestamp,
        kind,
        text,
        status: status(message.status),
        edited: message.edited,
        forwarded: message.forwarded,
        quoted: message.quoted.as_ref().map(|quoted| Quote {
            author: quoted
                .sender_name
                .clone()
                .unwrap_or_else(|| quoted.sender.clone()),
            text: quoted.summary.clone(),
        }),
        reactions,
    }
}

pub fn build(app: &mut App) -> State {
    let now = zapfast::util::now();
    let link = match &app.link {
        LinkStatus::Starting => Link::Starting,
        LinkStatus::Unlinked { qr, pair_code, .. } => Link::Unlinked {
            qr: qr.clone(),
            code: pair_code.clone(),
        },
        LinkStatus::Connecting => Link::Connecting,
        LinkStatus::Connected => Link::Connected,
        LinkStatus::Disconnected { reason } => Link::Disconnected {
            reason: reason.clone(),
        },
        LinkStatus::LoggedOut => Link::LoggedOut,
        LinkStatus::Failed(reason) => Link::Failed {
            reason: reason.clone(),
        },
    };
    let visible: Vec<Chat> = app.visible_chats().into_iter().cloned().collect();
    let chats = visible.iter().map(|chat| row(app, chat, now)).collect();
    let open = app.open_chat.clone().and_then(|id| {
        let chat = app.chat(&id)?.clone();
        let group = chat.is_group();
        let title = app.chat_title(&chat);
        let avatar = media(app.avatar(&id));
        let conversation = app.conversations.get(&id);
        let (messages, complete) = conversation
            .map(|conversation| {
                let start = conversation.messages.len().saturating_sub(WINDOW);
                (
                    conversation.messages[start..]
                        .iter()
                        .map(|m| message(app, group, m))
                        .collect(),
                    conversation.complete && start == 0,
                )
            })
            .unwrap_or_default();
        Some(Open {
            id,
            title,
            avatar,
            group,
            complete,
            messages,
        })
    });
    State {
        link,
        me: app.me.clone(),
        filter: filter(app.chat_filter),
        search: app.search.clone(),
        chats,
        open,
    }
}
