//! Asking the whole archive (REQ-ZNGA-020): Znga first says what to search
//! for, the archive is searched in visible chats only, and the answer is
//! written from those hits.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

/// Where a question came from, and so where its answer goes.
#[derive(Clone, Debug, PartialEq)]
pub enum AskOrigin {
    /// The Ask tab.
    Panel,
    /// The Znga chat of this account (REQ-ZCHAT-002).
    Thread { account: String },
    /// The person's own "Message yourself" chat (REQ-ZNGA-012).
    OwnChat { account: String, chat: String },
}

/// An ask in progress: plan → search → answer.
#[derive(Clone, Debug, PartialEq)]
pub struct Asking {
    pub origin: AskOrigin,
    pub question: String,
    /// The conversation with Znga before this question.
    pub history: Vec<super::provider::Turn>,
    pub hits: Vec<Hit>,
}

/// One archive message found for a question.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub chat: String,
    pub chat_title: String,
    pub sender: String,
    pub text: String,
    pub at: i64,
}

#[derive(Deserialize)]
struct Plan {
    #[serde(default)]
    search: Vec<String>,
}

/// The search terms in a planning answer: `{"search": ["…"]}`. At most
/// five, each at least three characters.
pub fn parse_plan(answer: &str) -> Vec<String> {
    let (Some(start), Some(end)) = (answer.find('{'), answer.rfind('}')) else {
        return Vec::new();
    };
    if end < start {
        return Vec::new();
    }
    let plan: Plan = serde_json::from_str(&answer[start..=end]).unwrap_or(Plan { search: vec![] });
    let mut seen = HashSet::new();
    plan.search
        .into_iter()
        .map(|term| term.trim().to_owned())
        .filter(|term| term.chars().count() >= 3 && seen.insert(term.to_lowercase()))
        .take(5)
        .collect()
}

/// The chats the search may read: titles of visible chats by id. Anything
/// else is dropped (REQ-ZNGA-020.A2).
pub type Visible = HashMap<String, (String, bool)>;

/// Searches `archive` for each term and keeps hits in visible chats,
/// newest first, without repeats. Runs on the store thread.
pub fn search(
    archive: &crate::archive::Archive,
    terms: &[String],
    visible: &Visible,
    names: &HashMap<String, String>,
    limit: usize,
) -> Vec<Hit> {
    let mut hits: Vec<Hit> = Vec::new();
    let mut ids = HashSet::new();
    for term in terms {
        let Ok(found) = archive.search_messages(term, 200) else {
            continue;
        };
        for message in found {
            let Some((title, _group)) = visible.get(&message.chat) else {
                continue;
            };
            if !ids.insert((message.chat.clone(), message.id.clone())) {
                continue;
            }
            let sender = if message.from_me {
                "Me".to_owned()
            } else {
                message
                    .sender_name
                    .clone()
                    .or_else(|| names.get(&message.sender).cloned())
                    .unwrap_or_else(|| title.clone())
            };
            hits.push(Hit {
                chat: message.chat.clone(),
                chat_title: title.clone(),
                sender,
                text: message.content.full_summary(),
                at: message.timestamp,
            });
        }
    }
    hits.sort_by_key(|hit| std::cmp::Reverse(hit.at));
    hits.truncate(limit);
    hits
}

/// A plain incoming text message, for tests.
#[cfg(test)]
pub fn tests_message(chat: &str, text: &str, at: i64) -> crate::model::Message {
    use crate::model::{Content, Delivery, Message};
    Message {
        id: format!("{chat}-{at}"),
        chat: chat.into(),
        sender: chat.into(),
        sender_name: None,
        from_me: false,
        timestamp: at,
        content: Content::Text {
            text: text.into(),
            preview: None,
        },
        status: Delivery::Delivered,
        delivered_at: None,
        read_at: None,
        quoted: None,
        reactions: Default::default(),
        history_order: None,
        edited: false,
        mentions: Default::default(),
        forwarded: false,
        thumbnail: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_are_parsed_and_tidied() {
        let terms = parse_plan(r#"Sure: {"search": ["flight", "Flight", "ok", "landing time"]}"#);
        assert_eq!(terms, vec!["flight", "landing time"]);
        assert!(parse_plan("no json").is_empty());
    }

    #[test]
    fn hits_come_only_from_visible_chats() {
        let archive = crate::archive::Archive::in_memory().unwrap();
        for (chat, text) in [
            ("ravi@s.whatsapp.net", "my flight lands at 9"),
            ("locked@s.whatsapp.net", "flight secret"),
        ] {
            archive.ensure_chat(chat, "x").unwrap();
            let message = tests_message(chat, text, 100);
            archive.insert_message(&message, None).unwrap();
        }
        let mut visible = Visible::new();
        visible.insert("ravi@s.whatsapp.net".into(), ("Ravi".into(), false));
        let hits = search(&archive, &["flight".into()], &visible, &HashMap::new(), 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chat_title, "Ravi");
        assert_eq!(hits[0].sender, "Ravi");
        assert!(hits[0].text.contains("lands at 9"));
    }
}
