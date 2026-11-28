//! The Znga chat (specs/znga-chat.md): a local chat at the top of the chat
//! list where everything between the person and Znga happens as messages.
//! One [`Thread`] per linked account, saved in that account's encrypted
//! `znga.db` and never handed to WhatsApp (REQ-ZCHAT-001).
//!
//! Pure state with time passed in, so it is unit-tested: pacing of unasked
//! messages ([`Thread::unasked`]), quiet hours, digest times and reading
//! the person's plain-word replies ([`intent`]).

use serde::{Deserialize, Serialize};

/// The Znga chat's id in the interface. It never names a WhatsApp chat.
pub const ZNGA_CHAT: &str = "znga@superwhatsapp.local";
/// The most messages kept per account.
pub const KEEP: usize = 500;
/// At most one unasked message per this many seconds; what arrives
/// meanwhile is folded into it (REQ-ZCHAT-005.A2).
pub const FOLD_WINDOW: i64 = 10 * 60;

/// Something Znga found, with the buttons that act on it (REQ-ZCHAT-003).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Card {
    #[serde(rename_all = "camelCase")]
    NeedsYou {
        chat: String,
        chat_title: String,
        sender: String,
        text: String,
        reason: String,
    },
    #[serde(rename_all = "camelCase")]
    Briefing { text: String },
    /// A draft waiting for Send. `draft` is its id in Znga's draft list
    /// while the app runs; the text is kept here so it survives a restart.
    #[serde(rename_all = "camelCase")]
    Draft {
        draft: u64,
        chat: String,
        chat_title: String,
        text: String,
        label: String,
    },
    #[serde(rename_all = "camelCase")]
    CatchUp { chat: String, chat_title: String },
    #[serde(rename_all = "camelCase")]
    Plan {
        chat_title: String,
        title: String,
        start: i64,
        end: i64,
        place: String,
        notes: String,
    },
    #[serde(rename_all = "camelCase")]
    Voice {
        note: u64,
        chat: String,
        chat_title: String,
        sender: String,
        seconds: u32,
    },
    #[serde(rename_all = "camelCase")]
    Promise {
        chat: String,
        chat_title: String,
        text: String,
        owes_me: bool,
    },
    #[serde(rename_all = "camelCase")]
    Drift {
        chat: String,
        chat_title: String,
        reason: String,
    },
}

impl Card {
    /// How the card reads: (kind, title, body, meta). Shared by both
    /// interfaces so they say the same thing.
    pub fn view(&self) -> (&'static str, String, String, String) {
        match self {
            Card::NeedsYou {
                chat_title,
                sender,
                text,
                reason,
                ..
            } => (
                "needsYou",
                format!("{chat_title} needs you"),
                format!("{sender}: {text}"),
                reason.clone(),
            ),
            Card::Briefing { text } => ("briefing", String::new(), text.clone(), String::new()),
            Card::Draft {
                chat_title,
                text,
                label,
                ..
            } => (
                "draft",
                format!("{label} to {chat_title}"),
                text.clone(),
                String::new(),
            ),
            Card::CatchUp { chat_title, .. } => {
                ("catchUp", chat_title.clone(), String::new(), String::new())
            }
            Card::Plan {
                chat_title,
                title,
                start,
                place,
                ..
            } => (
                "plan",
                title.clone(),
                if place.is_empty() {
                    card_time(*start)
                } else {
                    format!("{}, {place}", card_time(*start))
                },
                format!("From {chat_title}"),
            ),
            Card::Voice {
                chat_title,
                sender,
                seconds,
                ..
            } => (
                "voice",
                format!("Voice note from {sender}"),
                format!("{seconds} s"),
                chat_title.clone(),
            ),
            Card::Promise {
                chat_title,
                text,
                owes_me,
                ..
            } => (
                "promise",
                if chat_title.is_empty() {
                    "Reminder".into()
                } else if *owes_me {
                    format!("You promised {chat_title}")
                } else {
                    format!("{chat_title} is waiting on you")
                },
                text.clone(),
                String::new(),
            ),
            Card::Drift {
                chat_title, reason, ..
            } => ("drift", chat_title.clone(), reason.clone(), String::new()),
        }
    }

    /// The buttons this card offers, as (action, label), in order
    /// (REQ-ZCHAT-003.A1).
    pub fn actions(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Card::NeedsYou { .. } => &[
                ("open", "Open chat"),
                ("draft", "Draft reply"),
                ("done", "Done"),
            ],
            Card::Briefing { .. } => &[("copy", "Copy")],
            Card::Draft { .. } => &[
                ("send", "Send"),
                ("edit", "Edit in chat"),
                ("discard", "Discard"),
            ],
            Card::CatchUp { .. } => &[("open", "Open chat")],
            Card::Plan { .. } => &[("calendar", "Add to calendar"), ("dismiss", "Dismiss")],
            Card::Voice { .. } => &[("summarise", "Summarise")],
            Card::Promise { .. } => &[("done", "Done"), ("notPromise", "Not a promise")],
            Card::Drift { .. } => &[("opener", "Draft an opener")],
        }
    }

    /// The chat this card is about, when it is about one.
    pub fn chat(&self) -> Option<&str> {
        match self {
            Card::NeedsYou { chat, .. }
            | Card::Draft { chat, .. }
            | Card::CatchUp { chat, .. }
            | Card::Voice { chat, .. }
            | Card::Promise { chat, .. }
            | Card::Drift { chat, .. } => Some(chat).filter(|c| !c.is_empty()).map(|c| c.as_str()),
            Card::Briefing { .. } | Card::Plan { .. } => None,
        }
    }

    /// What [`Card::actions`]'s `action` reads as once taken.
    pub fn outcome(action: &str) -> &'static str {
        match action {
            "send" => "Sent",
            "edit" => "Opened in the chat",
            "discard" => "Discarded",
            "done" => "Done",
            "calendar" => "Added to calendar",
            "dismiss" => "Dismissed",
            "notPromise" => "Not a promise",
            "summarise" => "Summarising…",
            "opener" => "Drafting an opener…",
            "draft" => "Drafting a reply…",
            "copy" => "Copied",
            _ => "Opened",
        }
    }

    /// Whether taking `action` closes the card. Opening a chat or copying
    /// can be done again.
    pub fn closes(action: &str) -> bool {
        !matches!(action, "open" | "copy")
    }
}

/// A card and, once acted on, what happened.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub card: Card,
    #[serde(default)]
    pub outcome: Option<String>,
}

impl Item {
    pub fn new(card: Card) -> Self {
        Self {
            card,
            outcome: None,
        }
    }
}

/// One message in the Znga chat.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: u64,
    /// Unix seconds.
    pub at: i64,
    /// Written by the person (a question, an instruction, or a line saying
    /// what they asked for elsewhere).
    pub from_person: bool,
    pub text: String,
    pub items: Vec<Item>,
    /// Znga wrote it without being asked (REQ-ZCHAT-005).
    pub unasked: bool,
}

/// What an unasked message is, for its switch and quiet hours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    NeedsYou,
    WelcomeBack,
    Digest,
    Reminder,
}

/// Where an unasked message went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Posted {
    /// A new message, with its id.
    New(u64),
    /// Folded into the recent unasked message with this id.
    Folded(u64),
    /// Held until quiet hours end.
    Held,
}

/// A question Znga asked back, waiting for the person's answer
/// (REQ-ZCHAT-006.A2).
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    pub intent: Intent,
    /// (chat id, title) the person picks from.
    pub choices: Vec<(String, String)>,
}

/// One account's Znga chat.
#[derive(Default)]
pub struct Thread {
    entries: Vec<Entry>,
    next: u64,
    /// Znga messages since the person last read the chat.
    pub unread: u32,
    dirty: bool,
    /// The latest unasked message: (id, when).
    last_unasked: Option<(u64, i64)>,
    /// Unasked messages waiting for quiet hours to end.
    held: Vec<(Kind, String, Vec<Item>)>,
    pub question: Option<Question>,
}

impl Thread {
    /// Starts from saved messages, keeping anything posted before they
    /// loaded after them.
    pub fn restore(&mut self, saved: Vec<Entry>) {
        let newer = std::mem::take(&mut self.entries);
        self.entries = saved;
        self.next = self.entries.iter().map(|e| e.id).max().unwrap_or(0);
        for mut entry in newer {
            self.next += 1;
            if self.last_unasked.is_some_and(|(id, _)| id == entry.id) {
                self.last_unasked = Some((self.next, entry.at));
            }
            entry.id = self.next;
            self.entries.push(entry);
        }
        self.trim();
        self.dirty = true;
    }

    /// Messages from the earlier Ask tab, as the start of the chat
    /// (REQ-ZCHAT-002.A4).
    pub fn from_ask(turns: &[super::provider::Turn], at: i64) -> Vec<Entry> {
        turns
            .iter()
            .enumerate()
            .map(|(index, turn)| Entry {
                id: index as u64 + 1,
                at,
                from_person: turn.from_user,
                text: turn.text.clone(),
                items: Vec::new(),
                unasked: false,
            })
            .collect()
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Entry> {
        self.dirty = true;
        self.entries.iter_mut().find(|e| e.id == id)
    }

    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    pub fn mark_read(&mut self) {
        self.unread = 0;
    }

    fn push(&mut self, entry: Entry) -> u64 {
        self.next += 1;
        let id = self.next;
        self.entries.push(Entry { id, ..entry });
        self.trim();
        self.dirty = true;
        id
    }

    fn trim(&mut self) {
        let extra = self.entries.len().saturating_sub(KEEP);
        self.entries.drain(..extra);
    }

    /// What the person wrote.
    pub fn post_person(&mut self, text: impl Into<String>, now: i64) -> u64 {
        self.push(Entry {
            id: 0,
            at: now,
            from_person: true,
            text: text.into(),
            items: Vec::new(),
            unasked: false,
        })
    }

    /// Znga's answer to something the person asked (REQ-ZCHAT-002).
    pub fn post(
        &mut self,
        text: impl Into<String>,
        items: Vec<Item>,
        now: i64,
        reading: bool,
    ) -> u64 {
        if !reading {
            self.unread += 1;
        }
        self.push(Entry {
            id: 0,
            at: now,
            from_person: false,
            text: text.into(),
            items,
            unasked: false,
        })
    }

    /// A message Znga posts on its own (REQ-ZCHAT-005): held in quiet
    /// hours unless it is Needs you, and folded into the last unasked
    /// message when that is under [`FOLD_WINDOW`] old.
    pub fn unasked(
        &mut self,
        kind: Kind,
        line: impl Into<String>,
        items: Vec<Item>,
        now: i64,
        reading: bool,
        quiet: bool,
    ) -> Posted {
        let line = line.into();
        if quiet && kind != Kind::NeedsYou {
            self.held.push((kind, line, items));
            return Posted::Held;
        }
        if let Some((id, at)) = self.last_unasked
            && now - at < FOLD_WINDOW
            && let Some(entry) = self.entries.iter_mut().find(|e| e.id == id)
        {
            if !entry.text.starts_with("• ") {
                entry.text = format!("• {}", entry.text);
            }
            entry.text.push_str(&format!("\n• {line}"));
            entry.items.extend(items);
            if !reading && self.unread == 0 {
                self.unread = 1;
            }
            self.dirty = true;
            return Posted::Folded(id);
        }
        if !reading {
            self.unread += 1;
        }
        let id = self.push(Entry {
            id: 0,
            at: now,
            from_person: false,
            text: line,
            items,
            unasked: true,
        });
        self.last_unasked = Some((id, now));
        Posted::New(id)
    }

    /// Posts what waited for quiet hours, as one message, dropping kinds
    /// switched off meanwhile (REQ-ZCHAT-005.A4, A5).
    pub fn release_held(
        &mut self,
        now: i64,
        reading: bool,
        quiet: bool,
        enabled: impl Fn(Kind) -> bool,
    ) -> Option<Posted> {
        if quiet || self.held.is_empty() {
            return None;
        }
        let held: Vec<_> = std::mem::take(&mut self.held)
            .into_iter()
            .filter(|(kind, ..)| enabled(*kind))
            .collect();
        let mut posted = None;
        for (kind, line, items) in held {
            let result = self.unasked(kind, line, items, now, reading, false);
            posted.get_or_insert(result);
        }
        posted
    }

    /// Forgets held messages of a kind switched off (REQ-ZCHAT-005.A5).
    pub fn drop_held(&mut self, enabled: impl Fn(Kind) -> bool) {
        self.held.retain(|(kind, ..)| enabled(*kind));
    }

    pub fn held(&self) -> usize {
        self.held.len()
    }

    /// The card at (`entry`, `index`), when it is still open.
    pub fn open_card(&self, entry: u64, index: usize) -> Option<&Card> {
        let item = self
            .entries
            .iter()
            .find(|e| e.id == entry)?
            .items
            .get(index)?;
        item.outcome.is_none().then_some(&item.card)
    }

    /// Records what happened to a card; its buttons are then disabled
    /// (REQ-ZCHAT-003.A2).
    pub fn settle(&mut self, entry: u64, index: usize, outcome: impl Into<String>) {
        if let Some(item) = self
            .entries
            .iter_mut()
            .find(|e| e.id == entry)
            .and_then(|e| e.items.get_mut(index))
        {
            item.outcome = Some(outcome.into());
            self.dirty = true;
        }
    }

    /// The newest open Needs you card, for a plain "done".
    pub fn latest_needs_you(&self) -> Option<(u64, usize)> {
        self.entries.iter().rev().find_map(|entry| {
            entry
                .items
                .iter()
                .enumerate()
                .rev()
                .find(|(_, item)| {
                    item.outcome.is_none() && matches!(item.card, Card::NeedsYou { .. })
                })
                .map(|(index, _)| (entry.id, index))
        })
    }

    /// Settles every open card for `chat` of the given shape, e.g. a
    /// draft sent from somewhere else.
    pub fn settle_drafts(&mut self, draft: u64, outcome: &str) {
        for entry in &mut self.entries {
            for item in &mut entry.items {
                if item.outcome.is_none()
                    && matches!(&item.card, Card::Draft { draft: d, .. } if *d == draft)
                {
                    item.outcome = Some(outcome.to_owned());
                    self.dirty = true;
                }
            }
        }
    }
}

/// Minutes since local midnight → whether it is inside quiet hours from
/// `from` to `to` (minutes since midnight; `from > to` wraps midnight).
pub fn in_quiet(minute: u32, from: u32, to: u32) -> bool {
    match from.cmp(&to) {
        std::cmp::Ordering::Equal => false,
        std::cmp::Ordering::Less => (from..to).contains(&minute),
        std::cmp::Ordering::Greater => minute >= from || minute < to,
    }
}

/// `"08:00"` → minutes since midnight.
pub fn parse_hhmm(text: &str) -> Option<u32> {
    let (h, m) = text.trim().split_once(':')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60).then_some(h * 60 + m)
}

/// Which digest is due now, if any, given today's local date (as a day
/// number), the minute of the day, the digest times and the day each slot
/// last ran. Several due at once post one digest named after the latest
/// (REQ-ZCHAT-005.A3). Returns (slot, every slot to mark as run today).
pub fn digest_due(
    today: i64,
    minute: u32,
    slots: &[(&'static str, u32, i64)],
) -> Option<(&'static str, Vec<&'static str>)> {
    let due: Vec<&(&str, u32, i64)> = slots
        .iter()
        .filter(|(_, at, last)| minute >= *at && *last < today)
        .collect();
    let latest = due.iter().max_by_key(|(_, at, _)| *at)?;
    Some((latest.0, due.iter().map(|(slot, ..)| *slot).collect()))
}

/// What the person's message to Znga asks for (REQ-ZCHAT-006).
#[derive(Clone, Debug, PartialEq)]
pub enum Intent {
    /// "tell Ada I'll call at 5": `rest` starts with the person's name.
    Tell { rest: String },
    /// "remind me to pay Sam on Friday".
    Remind { what: String },
    /// "summarise the school group".
    CatchUp { name: String },
    /// "done", about the latest Needs you.
    Done,
    /// Anything else: a question for the archive (REQ-ZNGA-020).
    Ask(String),
}

fn strip_prefix_ci<'a>(text: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    let lower = text.to_lowercase();
    prefixes.iter().find_map(|prefix| {
        (lower.starts_with(prefix) && text.is_char_boundary(prefix.len()))
            .then(|| text[prefix.len()..].trim())
    })
}

/// Reads a plain-word message. Nothing here sends: every intent becomes a
/// draft, a promise, a catch-up or an answer.
pub fn intent(text: &str) -> Intent {
    let text = text.trim();
    let bare = text.trim_end_matches(['.', '!']).trim().to_lowercase();
    if matches!(
        bare.as_str(),
        "done" | "mark done" | "mark it done" | "it's done" | "handled"
    ) {
        return Intent::Done;
    }
    if let Some(rest) = strip_prefix_ci(
        text,
        &["tell ", "message ", "text ", "write to ", "reply to "],
    ) && !rest.is_empty()
    {
        return Intent::Tell {
            rest: rest.to_owned(),
        };
    }
    if let Some(what) = strip_prefix_ci(
        text,
        &["remind me to ", "remind me ", "reminder: ", "reminder to "],
    ) && !what.is_empty()
    {
        return Intent::Remind {
            what: what.to_owned(),
        };
    }
    if let Some(name) = strip_prefix_ci(
        text,
        &[
            "summarise ",
            "summarize ",
            "catch me up on ",
            "catch me up with ",
            "recap ",
            "what's new in ",
            "what is new in ",
        ],
    ) && !name.is_empty()
    {
        let name = name.trim_end_matches(['.', '?', '!']).trim();
        let name = strip_prefix_ci(name, &["the "]).unwrap_or(name);
        return Intent::CatchUp {
            name: name.to_owned(),
        };
    }
    Intent::Ask(text.to_owned())
}

/// Case-, accent- and spacing-insensitive form of a name.
pub fn fold(name: &str) -> String {
    crate::util::search_key(name)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Every chat `name` could mean: exact title matches if any, else titles
/// starting with it, else titles containing it as whole words.
pub fn candidates<'a>(name: &str, chats: &'a [(String, String)]) -> Vec<&'a (String, String)> {
    let wanted = fold(name);
    if wanted.is_empty() {
        return Vec::new();
    }
    let folded: Vec<(&(String, String), String)> =
        chats.iter().map(|chat| (chat, fold(&chat.1))).collect();
    let exact: Vec<_> = folded
        .iter()
        .filter(|(_, t)| *t == wanted)
        .map(|(c, _)| *c)
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    let prefix: Vec<_> = folded
        .iter()
        .filter(|(_, t)| t.starts_with(&format!("{wanted} ")))
        .map(|(c, _)| *c)
        .collect();
    if !prefix.is_empty() {
        return prefix;
    }
    let words: Vec<&str> = wanted.split(' ').collect();
    folded
        .iter()
        .filter(|(_, t)| {
            let title: Vec<&str> = t.split(' ').collect();
            title.windows(words.len()).any(|w| w == words.as_slice())
        })
        .map(|(c, _)| *c)
        .collect()
}

/// Who a "tell …" is for and what to say: the longest leading run of
/// words (up to four) that names at least one chat. `None` when no
/// leading words name a chat.
pub fn addressee<'a>(
    rest: &str,
    chats: &'a [(String, String)],
) -> Option<(Vec<&'a (String, String)>, String)> {
    let words: Vec<&str> = rest.split_whitespace().collect();
    for take in (1..=words.len().min(4)).rev() {
        let name = words[..take].join(" ");
        let name = name.trim_end_matches([',', ':']);
        let found = candidates(name, chats);
        if !found.is_empty() {
            let message = words[take..].join(" ");
            let message = strip_prefix_ci(&message, &["that ", ": "]).unwrap_or(&message);
            return Some((found, message.trim_start_matches(':').trim().to_owned()));
        }
    }
    None
}

/// The person's pick among a question's choices: a number, or a name that
/// matches exactly one choice.
pub fn pick<'a>(answer: &str, choices: &'a [(String, String)]) -> Option<&'a (String, String)> {
    let answer = answer.trim().trim_end_matches('.');
    if let Ok(n) = answer.parse::<usize>() {
        return choices.get(n.checked_sub(1)?);
    }
    let found = candidates(answer, choices);
    (found.len() == 1).then(|| found[0])
}

/// A chat named in a reminder ("pay Sam on Friday"), when exactly one
/// chat title appears in it as whole words.
pub fn mentioned<'a>(text: &str, chats: &'a [(String, String)]) -> Option<&'a (String, String)> {
    let words: Vec<String> = fold(text).split(' ').map(str::to_owned).collect();
    let mut hits: Vec<&(String, String)> = Vec::new();
    for chat in chats {
        let title: Vec<String> = fold(&chat.1).split(' ').map(str::to_owned).collect();
        let first: Vec<String> = title.iter().take(1).cloned().collect();
        let matches = |needle: &[String]| {
            !needle.is_empty() && words.windows(needle.len()).any(|w| w == needle)
        };
        if matches(&title) || matches(&first) {
            hits.push(chat);
        }
    }
    // A full-title hit beats first-name hits.
    let full: Vec<_> = hits
        .iter()
        .copied()
        .filter(|chat| {
            let title: Vec<String> = fold(&chat.1).split(' ').map(str::to_owned).collect();
            words.windows(title.len()).any(|w| w == title.as_slice())
        })
        .collect();
    match (full.len(), hits.len()) {
        (1, _) => Some(full[0]),
        (0, 1) => Some(hits[0]),
        _ => None,
    }
}

/// When a reminder is due: "today", "tonight", "tomorrow", a weekday, and
/// "at 5", "at 5pm", "at 17:30". `now` is local; returns local civil time.
pub fn due_in(text: &str, now: jiff::civil::DateTime) -> Option<jiff::civil::DateTime> {
    use jiff::civil::Weekday;
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != ':')
        .filter(|w| !w.is_empty())
        .collect();
    let mut date = None;
    for word in &words {
        let day = match *word {
            "today" | "tonight" => Some(now.date()),
            "tomorrow" => now.date().tomorrow().ok(),
            _ => None,
        };
        if day.is_some() {
            date = day;
            break;
        }
        let weekday = match *word {
            "monday" => Some(Weekday::Monday),
            "tuesday" => Some(Weekday::Tuesday),
            "wednesday" => Some(Weekday::Wednesday),
            "thursday" => Some(Weekday::Thursday),
            "friday" => Some(Weekday::Friday),
            "saturday" => Some(Weekday::Saturday),
            "sunday" => Some(Weekday::Sunday),
            _ => None,
        };
        if let Some(weekday) = weekday {
            let mut day = now.date();
            for _ in 0..7 {
                day = day.tomorrow().ok()?;
                if day.weekday() == weekday {
                    break;
                }
            }
            date = Some(day);
            break;
        }
    }
    let mut time = None;
    for pair in words.windows(2) {
        if pair[0] != "at" {
            continue;
        }
        let token = pair[1];
        let (digits, pm, am) = if let Some(d) = token.strip_suffix("pm") {
            (d, true, false)
        } else if let Some(d) = token.strip_suffix("am") {
            (d, false, true)
        } else {
            (token, false, false)
        };
        let (h, m) = match digits.split_once(':') {
            Some((h, m)) => (h.parse::<i8>().ok(), m.parse::<i8>().ok()),
            None => (digits.parse::<i8>().ok(), Some(0)),
        };
        if let (Some(mut h), Some(m)) = (h, m) {
            // "at 5" with no am/pm means the afternoon for 1..=7.
            if pm && h < 12 || !am && !pm && (1..=7).contains(&h) {
                h += 12;
            }
            if am && h == 12 {
                h = 0;
            }
            time = jiff::civil::Time::new(h, m, 0, 0).ok();
            break;
        }
    }
    let lower_has = |w: &str| words.contains(&w);
    if time.is_none() && lower_has("tonight") {
        time = jiff::civil::Time::new(20, 0, 0, 0).ok();
    }
    match (date, time) {
        (None, None) => None,
        (Some(date), time) => {
            Some(date.to_datetime(time.unwrap_or(jiff::civil::Time::constant(9, 0, 0, 0))))
        }
        (None, Some(time)) => {
            let today = now.date().to_datetime(time);
            Some(if today > now {
                today
            } else {
                now.date().tomorrow().ok()?.to_datetime(time)
            })
        }
    }
}

/// A short, actionable line for an engine error (REQ-ZCHAT-002.A3).
pub fn explain_error(error: &str, engine: super::config::Engine) -> String {
    use super::config::Engine;
    let lower = error.to_lowercase();
    let unreachable = [
        "connect",
        "refused",
        "unreachable",
        "timed out",
        "dns",
        "resolve",
    ]
    .iter()
    .any(|word| lower.contains(word));
    let hint = if error == super::runner::STOPPED {
        ""
    } else if unreachable && engine == Engine::Local {
        " Is Ollama running? Start it (ollama serve) and try again."
    } else if unreachable {
        " Check your connection and the engine's address in Settings › Znga."
    } else if lower.contains("401") || lower.contains("403") || lower.contains("key") {
        " Check the engine's key in Settings › Znga."
    } else if lower.contains("setup") || lower.contains("turn znga on") {
        " Open Settings › Znga."
    } else {
        ""
    };
    format!("⚠ {error}{hint}")
}

/// A plan's start as "Fri 28 Nov, 18:00", in local time.
pub fn card_time(at: i64) -> String {
    jiff::Timestamp::from_second(at)
        .map(|t| {
            t.to_zoned(jiff::tz::TimeZone::system())
                .strftime("%a %-d %b, %H:%M")
                .to_string()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn needs(chat: &str) -> Vec<Item> {
        vec![Item::new(Card::NeedsYou {
            chat: chat.into(),
            chat_title: chat.into(),
            sender: "Ada".into(),
            text: "are you there?".into(),
            reason: "a question".into(),
        })]
    }

    #[test]
    fn unasked_messages_fold_within_ten_minutes() {
        let mut thread = Thread::default();
        let first = thread.unasked(
            Kind::NeedsYou,
            "Ada needs you",
            needs("a"),
            1000,
            false,
            false,
        );
        assert!(matches!(first, Posted::New(_)));
        assert!(matches!(
            thread.unasked(
                Kind::NeedsYou,
                "Ben needs you",
                needs("b"),
                1100,
                false,
                false
            ),
            Posted::Folded(_)
        ));
        assert!(matches!(
            thread.unasked(
                Kind::Reminder,
                "A promise is due",
                Vec::new(),
                1500,
                false,
                false
            ),
            Posted::Folded(_)
        ));
        assert_eq!(thread.entries().len(), 1);
        let entry = &thread.entries()[0];
        assert_eq!(entry.items.len(), 2);
        assert!(
            entry.text.contains("Ada")
                && entry.text.contains("Ben")
                && entry.text.contains("promise")
        );
        assert_eq!(thread.unread, 1);
        assert!(matches!(
            thread.unasked(
                Kind::NeedsYou,
                "Cy needs you",
                needs("c"),
                1000 + FOLD_WINDOW,
                false,
                false
            ),
            Posted::New(_)
        ));
        assert_eq!(thread.unread, 2);
    }

    #[test]
    fn quiet_hours_hold_everything_but_needs_you() {
        let mut thread = Thread::default();
        assert_eq!(
            thread.unasked(Kind::Digest, "Morning digest", Vec::new(), 0, false, true),
            Posted::Held
        );
        assert!(matches!(
            thread.unasked(Kind::NeedsYou, "Ada needs you", needs("a"), 0, false, true),
            Posted::New(_)
        ));
        assert_eq!(thread.entries().len(), 1);
        assert_eq!(thread.release_held(100, false, true, |_| true), None);
        assert!(thread.release_held(100, false, false, |_| true).is_some());
        assert!(thread.entries()[0].text.contains("Morning digest"));
    }

    #[test]
    fn a_kind_switched_off_while_held_is_dropped() {
        let mut thread = Thread::default();
        thread.unasked(Kind::Digest, "Digest", Vec::new(), 0, false, true);
        assert_eq!(
            thread.release_held(5, false, false, |kind| kind != Kind::Digest),
            None
        );
        assert!(thread.entries().is_empty());
    }

    #[test]
    fn reading_keeps_unread_at_zero_and_settled_cards_stay_settled() {
        let mut thread = Thread::default();
        let id = thread.post(
            "A draft",
            vec![Item::new(Card::Draft {
                draft: 1,
                chat: "ada".into(),
                chat_title: "Ada".into(),
                text: "On my way".into(),
                label: "Suggested reply".into(),
            })],
            0,
            true,
        );
        assert_eq!(thread.unread, 0);
        assert!(thread.open_card(id, 0).is_some());
        thread.settle(id, 0, "Sent");
        assert!(thread.open_card(id, 0).is_none());
    }

    #[test]
    fn quiet_hours_wrap_midnight() {
        assert!(in_quiet(23 * 60, 22 * 60, 7 * 60));
        assert!(in_quiet(60, 22 * 60, 7 * 60));
        assert!(!in_quiet(12 * 60, 22 * 60, 7 * 60));
        assert!(in_quiet(13 * 60, 12 * 60, 14 * 60));
        assert!(!in_quiet(13 * 60, 9 * 60, 9 * 60));
        assert_eq!(parse_hhmm("08:00"), Some(480));
        assert_eq!(parse_hhmm("25:00"), None);
    }

    #[test]
    fn a_digest_runs_once_a_day_at_the_first_chance() {
        let slots = [("morning", 480, 0), ("evening", 1200, 0)];
        assert_eq!(digest_due(10, 479, &slots), None);
        assert_eq!(
            digest_due(10, 481, &slots),
            Some(("morning", vec!["morning"]))
        );
        let ran = [("morning", 480, 10), ("evening", 1200, 0)];
        assert_eq!(digest_due(10, 600, &ran), None);
        // Opened late: one digest, both slots marked.
        assert_eq!(
            digest_due(11, 1300, &ran),
            Some(("evening", vec!["morning", "evening"]))
        );
    }

    #[test]
    fn plain_words_become_intents() {
        assert_eq!(
            intent("Tell Ada I'll call at 5"),
            Intent::Tell {
                rest: "Ada I'll call at 5".into()
            }
        );
        assert_eq!(
            intent("remind me to pay Sam on Friday"),
            Intent::Remind {
                what: "pay Sam on Friday".into()
            }
        );
        assert_eq!(
            intent("summarise the school group"),
            Intent::CatchUp {
                name: "school group".into()
            }
        );
        assert_eq!(intent("Done."), Intent::Done);
        assert_eq!(
            intent("Do I owe anyone a reply?"),
            Intent::Ask("Do I owe anyone a reply?".into())
        );
    }

    fn chats() -> Vec<(String, String)> {
        vec![
            ("ada".into(), "Ada Lovelace".into()),
            ("sam1".into(), "Sam Lee".into()),
            ("sam2".into(), "Sam Patel".into()),
            ("school".into(), "School group".into()),
        ]
    }

    #[test]
    fn the_addressee_is_found_or_ambiguous() {
        let chats = chats();
        let (found, message) = addressee("Ada I'll call at 5", &chats).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "ada");
        assert_eq!(message, "I'll call at 5");
        let (found, message) = addressee("Sam I'm late", &chats).unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(message, "I'm late");
        let (found, _) = addressee("Sam Patel that I'm late", &chats).unwrap();
        assert_eq!(found[0].0, "sam2");
        assert!(addressee("Zed hello", &chats).is_none());
        let choices = vec![chats[1].clone(), chats[2].clone()];
        assert_eq!(pick("2", &choices).unwrap().0, "sam2");
        assert_eq!(pick("Sam Lee", &choices).unwrap().0, "sam1");
        assert!(pick("Sam", &choices).is_none());
        assert_eq!(
            mentioned("pay Sam Lee on Friday", &chats).unwrap().0,
            "sam1"
        );
        assert!(mentioned("pay Sam on Friday", &chats).is_none());
        assert_eq!(candidates("school group", &chats)[0].0, "school");
    }

    #[test]
    fn reminder_times_are_read() {
        let now: jiff::civil::DateTime = "2026-11-26T10:00".parse().unwrap(); // a Thursday
        assert_eq!(
            due_in("pay Sam on Friday", now).unwrap().to_string(),
            "2026-11-27T09:00:00"
        );
        assert_eq!(
            due_in("call mum tomorrow at 6pm", now).unwrap().to_string(),
            "2026-11-27T18:00:00"
        );
        assert_eq!(
            due_in("ring at 5", now).unwrap().to_string(),
            "2026-11-26T17:00:00"
        );
        assert_eq!(due_in("buy milk", now), None);
    }

    #[test]
    fn engine_errors_say_what_to_do() {
        use super::super::config::Engine;
        assert!(
            explain_error("Could not connect to localhost:11434", Engine::Local).contains("Ollama")
        );
        assert!(explain_error("HTTP 401", Engine::OpenAi).contains("key"));
    }

    #[test]
    fn restored_messages_come_before_newer_ones() {
        let mut thread = Thread::default();
        thread.post("new", Vec::new(), 5, false);
        let saved = Thread::from_ask(
            &[
                super::super::provider::Turn::user("q"),
                super::super::provider::Turn::assistant("a"),
            ],
            1,
        );
        thread.restore(saved);
        let texts: Vec<&str> = thread.entries().iter().map(|e| e.text.as_str()).collect();
        assert_eq!(texts, ["q", "a", "new"]);
        let ids: Vec<u64> = thread.entries().iter().map(|e| e.id).collect();
        assert_eq!(ids, [1, 2, 3]);
    }
}
