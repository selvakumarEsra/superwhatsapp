//! What the web interface shows, built from the core's state after each
//! step (REQ-WEBUI-002). The page never holds the state itself.

use serde::Serialize;
use zapfast::app::App;
use zapfast::backend::LinkStatus;
use zapfast::model::{CallStatus, Chat, ChatFilter, Content, Delivery, Message, Page};

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
    /// What the sidebar shows as chosen: chats, favourites, archived,
    /// starred or calls.
    pub section: &'static str,
    /// The starred messages, newest first, while Starred is shown.
    pub starred: Vec<StarredRow>,
    /// The call log, newest first, while Calls is shown.
    pub calls: Vec<CallRow>,
    /// The contact or group info panel, when open.
    pub info: Option<Info>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StarredRow {
    pub chat: String,
    pub chat_title: String,
    pub msg: Msg,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRow {
    pub id: String,
    pub chat: String,
    pub title: String,
    pub avatar: Option<String>,
    pub group: bool,
    pub time: i64,
    pub outgoing: bool,
    pub missed: bool,
    pub video: bool,
    /// Seconds the call was up; zero when it never was.
    pub duration: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub admin: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub id: String,
    pub title: String,
    pub avatar: Option<String>,
    pub group: bool,
    pub channel: bool,
    pub phone: Option<String>,
    pub description: Option<String>,
    pub member_count: usize,
    pub members: Vec<Member>,
    /// Groups this contact shares with us: (id, title).
    pub common_groups: Vec<(String, String)>,
    pub muted: bool,
    pub pinned: bool,
    pub archived: bool,
    pub favorite: bool,
    pub can_leave: bool,
    pub left: bool,
    /// The disappearing-messages timer in seconds; zero when off.
    pub timer: u32,
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
    let section = if app.page == Page::Calls {
        "calls"
    } else if app.show_starred {
        "starred"
    } else if app.show_archived {
        "archived"
    } else if app.chat_filter == ChatFilter::Favorites && app.label_filter.is_none() {
        "favourites"
    } else {
        "chats"
    };
    let starred = if app.show_starred {
        let rows: Vec<zapfast::archive::Starred> = app.starred.clone();
        rows.iter()
            .map(|row| {
                let chat = app.chat(&row.message.chat).cloned();
                let group = chat.as_ref().is_some_and(Chat::is_group);
                StarredRow {
                    chat: row.message.chat.clone(),
                    chat_title: chat
                        .as_ref()
                        .map(|chat| app.chat_title(chat))
                        .unwrap_or_else(|| app.chat_title_of(&row.message.chat)),
                    msg: message(app, group, &row.message),
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    let calls = if app.page == Page::Calls {
        let log = app.call_log.clone();
        log.iter().map(|entry| call_row(app, entry)).collect()
    } else {
        Vec::new()
    };
    let info = app
        .info
        .as_ref()
        .map(|panel| panel.chat.clone())
        .map(|id| info(app, &id, now));
    State {
        link,
        me: app.me.clone(),
        filter: filter(app.chat_filter),
        search: app.search.clone(),
        chats,
        open,
        section,
        starred,
        calls,
        info,
    }
}

fn call_row(app: &mut App, entry: &Message) -> CallRow {
    let (video, missed, duration, group_call) = match &entry.content {
        Content::Call {
            video,
            status,
            duration,
            group,
            ..
        } => (*video, *status == CallStatus::Missed, *duration, *group),
        _ => (false, false, 0, false),
    };
    let chat = app.chat(&entry.chat).cloned();
    CallRow {
        id: entry.id.clone(),
        chat: entry.chat.clone(),
        title: chat
            .as_ref()
            .map(|chat| app.chat_title(chat))
            .unwrap_or_else(|| app.chat_title_of(&entry.chat)),
        avatar: media(app.avatar(&entry.chat)),
        group: group_call || chat.as_ref().is_some_and(Chat::is_group),
        time: entry.timestamp,
        outgoing: entry.from_me,
        missed,
        video,
        duration,
    }
}

fn info(app: &mut App, id: &str, now: i64) -> Info {
    // Group members may have no chat of their own.
    let chat = app
        .chat(id)
        .cloned()
        .unwrap_or_else(|| Chat::new(id.to_owned(), app.display_name(id)));
    let ours: Vec<String> = app.our_ids().into_iter().map(str::to_owned).collect();
    let ours_ref: Vec<&str> = ours.iter().map(String::as_str).collect();
    let roles = zapfast::group_admin::Roles::of(&chat);
    let members = if chat.is_group() {
        app.participant_list(&chat)
            .into_iter()
            .map(|(member, name)| Member {
                avatar: media(app.avatar(&member)),
                admin: roles.is_admin(&member),
                id: member,
                name,
            })
            .collect()
    } else {
        Vec::new()
    };
    let common_groups = if chat.is_group() || chat.is_channel() || ours.iter().any(|o| o == id) {
        Vec::new()
    } else {
        let groups: Vec<Chat> = app
            .chats
            .iter()
            .filter(|group| group.is_group() && group.participants.iter().any(|p| p == id))
            .cloned()
            .collect();
        groups
            .iter()
            .map(|group| (group.id.clone(), app.chat_title(group)))
            .collect()
    };
    Info {
        id: chat.id.clone(),
        title: app.chat_title(&chat),
        avatar: media(app.avatar(id)),
        group: chat.is_group(),
        channel: chat.is_channel(),
        phone: chat.phone().map(str::to_owned),
        description: chat
            .group_description
            .clone()
            .filter(|text| !text.trim().is_empty()),
        member_count: chat.participants.len(),
        members,
        common_groups,
        muted: chat.muted(now),
        pinned: chat.pinned,
        archived: chat.archived,
        favorite: chat.favorite,
        can_leave: chat.can_leave(&ours_ref),
        left: chat.left,
        timer: chat.ephemeral_expiration.unwrap_or(0),
    }
}
