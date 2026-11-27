//! Profiles of contacts, groups and group members (specs/znga-profiles.md).
//!
//! A profile is what Znga, as the person's digital twin, knows about one
//! chat or one person: who they are to the person, how the person talks
//! with them, what a group is for and how it feels. Every value says where
//! it came from (learned from a chat, or confirmed by the person), and
//! confirmed values are never overwritten by learning (REQ-ZPROF-001).
//! Values that touch health, religion, politics, sexuality or money are
//! marked sensitive and stay out of every prompt until the person confirms
//! them (REQ-ZPROF-003).
//!
//! Everything here is pure: the app bridge is in [`super::hooks`] and
//! [`super::chat`], the file in [`super::store`]. Profile contents never
//! reach the log.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

/// How far back the one-time history read goes, per chat (REQ-ZPROF-002).
pub const HISTORY_DAYS: i64 = 90;
pub const HISTORY_MESSAGES: usize = 500;
/// Model calls for profiles per local day, backfill and live passes
/// together.
pub const DAILY_CALLS: u32 = 200;
/// A group's theme, mood and summary are refreshed after this many new
/// messages or this many days (REQ-ZPROF-004).
pub const REFRESH_MESSAGES: u32 = 50;
pub const REFRESH_DAYS: i64 = 7;
/// A contact's profile is relearned after this many new messages.
pub const CONTACT_MESSAGES: u32 = 10;
/// The window for a group's active members.
pub const ACTIVE_DAYS: i64 = 30;
/// The most values kept under one list field.
const PER_FIELD: usize = 12;

/// The relationships a contact can have to the person.
pub const RELATIONSHIPS: [&str; 7] = [
    "family",
    "partner",
    "friend",
    "colleague",
    "client",
    "service",
    "other",
];

/// One field a profile can hold.
#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub key: &'static str,
    pub label: &'static str,
    /// Holds one value (a new one replaces it) rather than a list.
    pub single: bool,
    pub contact: bool,
    pub group: bool,
    /// What the model is told the field means.
    hint: &'static str,
}

const fn field(
    key: &'static str,
    label: &'static str,
    single: bool,
    contact: bool,
    group: bool,
    hint: &'static str,
) -> Field {
    Field {
        key,
        label,
        single,
        contact,
        group,
        hint,
    }
}

pub const FIELDS: &[Field] = &[
    field(
        "relationship",
        "Who they are to you",
        true,
        true,
        false,
        "one of family, partner, friend, colleague, client, service, other; add the detail in brackets when clear, like \"family (sister)\"",
    ),
    field(
        "theme",
        "What it's about",
        true,
        false,
        true,
        "what the group is for, in one line",
    ),
    field(
        "role",
        "Your role",
        true,
        false,
        true,
        "the person's role in the group (organiser, member, admin, quiet reader…)",
    ),
    field(
        "language",
        "Language",
        true,
        true,
        true,
        "the language(s) used",
    ),
    field(
        "formality",
        "Formality",
        true,
        true,
        false,
        "casual, warm, polite or formal",
    ),
    field(
        "length",
        "Message length",
        true,
        true,
        false,
        "how long the person's messages to them usually are",
    ),
    field(
        "emoji",
        "Emoji",
        true,
        true,
        false,
        "how much emoji the person uses with them",
    ),
    field(
        "norms",
        "Norms",
        true,
        false,
        true,
        "tone and unwritten rules of the group",
    ),
    field("replies", "Reply time", true, true, false, ""),
    field("busy", "When it's busy", true, false, true, ""),
    field(
        "mood",
        "Mood",
        true,
        false,
        true,
        "the group's overall mood: positive, neutral, negative or mixed",
    ),
    field(
        "summary",
        "Recently",
        true,
        false,
        true,
        "what happened recently, two or three sentences",
    ),
    field("topics", "Topics", false, true, true, "recurring topics"),
    field(
        "likes",
        "Likes",
        false,
        true,
        false,
        "things they like or prefer",
    ),
    field(
        "dislikes",
        "Dislikes",
        false,
        true,
        false,
        "things they dislike or avoid",
    ),
    field(
        "dates",
        "Important dates",
        false,
        true,
        false,
        "birthdays, anniversaries, deadlines, with the date",
    ),
    field(
        "people",
        "People they mention",
        false,
        true,
        false,
        "people in their life, with who they are",
    ),
    field(
        "insights",
        "Open questions and plans",
        false,
        false,
        true,
        "open questions, decisions, plans and anything waiting for the person",
    ),
    field(
        "notes",
        "Notes",
        false,
        true,
        true,
        "anything else lasting and useful",
    ),
];

pub fn field_of(key: &str) -> Option<&'static Field> {
    FIELDS.iter().find(|f| f.key == key)
}

/// Fields computed on this computer, never asked of the model.
fn local(key: &str) -> bool {
    matches!(key, "replies" | "busy")
}

/// One value in a profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Value {
    pub text: String,
    /// Written or approved by the person: never overwritten by learning.
    pub confirmed: bool,
    /// Health, religion, politics, sexuality or money: held back until
    /// confirmed.
    pub sensitive: bool,
    /// When it was last seen or set (Unix seconds).
    pub at: i64,
    /// The chat it was learned from; empty when the person wrote it.
    #[serde(default)]
    pub from: String,
}

impl Value {
    /// Whether drafts, answers and questions may use it (REQ-ZPROF-003).
    pub fn usable(&self) -> bool {
        self.confirmed || !self.sensitive
    }
}

/// A group member's message count in the active window (REQ-ZPROF-004).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Active {
    pub id: String,
    pub name: String,
    pub count: u32,
    /// Not in the group's participant list any more.
    pub former: bool,
}

/// A member's mood in a group, with the one before it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mood {
    pub now: String,
    pub before: String,
}

/// What Znga knows about one contact, group or group member.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    /// Filled in on load; `znga.db` is per account already.
    #[serde(skip)]
    pub account: String,
    /// The chat id; for a group member, their WhatsApp id, which is also
    /// the id of a direct chat with them (REQ-ZPROF-001.A3).
    pub id: String,
    pub title: String,
    pub group: bool,
    pub fields: BTreeMap<String, Vec<Value>>,
    /// Deleted values ("key\u{1f}normal text") and skipped questions
    /// ("ask\u{1f}key"): never learned or asked again.
    pub forgotten: BTreeSet<String>,
    /// The mood before the current one, for the trend.
    pub mood_before: String,
    /// Groups: each member's mood, by member id.
    pub moods: BTreeMap<String, Mood>,
    /// Groups: members ranked by messages in the last 30 days.
    pub active: Vec<Active>,
    /// Last time the model updated this profile.
    pub learned_at: i64,
    /// Messages seen since then.
    pub pending: u32,
    /// The one-time history read ran for this chat.
    pub backfilled: bool,
}

fn normal(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tomb(key: &str, text: &str) -> String {
    format!("{key}\u{1f}{}", normal(text))
}

/// Word beginnings that mark a sensitive trait; two-word ones are matched
/// as a phrase.
const SENSITIVE_WORDS: &[&str] = &[
    "health",
    "illness",
    "sick",
    "cancer",
    "diabet",
    "pregnan",
    "therap",
    "depress",
    "anxiety",
    "diagnos",
    "hospital",
    "surgery",
    "medication",
    "disease",
    "disorder",
    "mental",
    "religio",
    "church",
    "mosque",
    "temple",
    "pray",
    "faith",
    "christian",
    "muslim",
    "islam",
    "hindu",
    "jewish",
    "buddhis",
    "atheis",
    "politic",
    "vote",
    "voting",
    "election",
    "liberal",
    "conservative",
    "gay",
    "lesbian",
    "bisexual",
    "transgender",
    "queer",
    "sexual",
    "debt",
    "loan",
    "bankrupt",
    "salary",
    "unemployed",
    "party member",
    "money trouble",
    "rent arrears",
    "physiotherap",
];

/// Whether a value touches a sensitive trait, by its words. The model also
/// marks them; either is enough.
pub fn looks_sensitive(text: &str) -> bool {
    let words = normal(text);
    let padded = format!(" {words} ");
    SENSITIVE_WORDS.iter().any(|stem| {
        if stem.contains(' ') {
            padded.contains(&format!(" {stem}"))
        } else {
            words.split_whitespace().any(|word| word.starts_with(stem))
        }
    })
}

fn mood_of(text: &str) -> Option<&'static str> {
    let lower = text.trim().to_lowercase();
    ["positive", "neutral", "negative", "mixed"]
        .into_iter()
        .find(|mood| lower.starts_with(mood))
}

impl Profile {
    pub fn values(&self, key: &str) -> &[Value] {
        self.fields.get(key).map(Vec::as_slice).unwrap_or_default()
    }

    pub fn first(&self, key: &str) -> Option<&Value> {
        self.values(key).first()
    }

    fn is_forgotten(&self, key: &str, text: &str) -> bool {
        self.forgotten.contains(&tomb(key, text))
    }

    /// A learned value: dropped when forgotten or when the field holds a
    /// confirmed value it would replace; otherwise added or refreshed.
    fn learn(&mut self, key: &str, text: &str, sensitive: bool, from: &str, at: i64) -> bool {
        let Some(def) = field_of(key) else {
            return false;
        };
        let text = text.trim().trim_end_matches('.').trim();
        if text.is_empty() || text.chars().count() > 300 || self.is_forgotten(key, text) {
            return false;
        }
        let value = Value {
            text: text.to_owned(),
            confirmed: false,
            sensitive: sensitive || looks_sensitive(text),
            at,
            from: from.to_owned(),
        };
        let values = self.fields.entry(key.to_owned()).or_default();
        if let Some(same) = values.iter_mut().find(|v| normal(&v.text) == normal(text)) {
            same.at = at;
            return false;
        }
        if def.single {
            if values.iter().any(|v| v.confirmed) {
                return false;
            }
            if key == "mood"
                && let Some(old) = values.first()
            {
                self.mood_before = old.text.clone();
            }
            *values = vec![value];
            return true;
        }
        values.push(value);
        while values.len() > PER_FIELD {
            match values.iter().position(|v| !v.confirmed) {
                Some(oldest) => {
                    values.remove(oldest);
                }
                None => break,
            }
        }
        true
    }

    /// The person wrote `text` under `key`: confirmed (REQ-ZPROF-003).
    fn set(&mut self, key: &str, text: &str, at: i64) -> bool {
        let Some(def) = field_of(key) else {
            return false;
        };
        let text = text.trim();
        if text.is_empty() {
            return false;
        }
        self.forgotten.remove(&tomb(key, text));
        let value = Value {
            text: text.to_owned(),
            confirmed: true,
            sensitive: looks_sensitive(text),
            at,
            from: String::new(),
        };
        let values = self.fields.entry(key.to_owned()).or_default();
        if def.single {
            *values = vec![value];
        } else if let Some(same) = values.iter_mut().find(|v| normal(&v.text) == normal(text)) {
            *same = value;
        } else {
            values.push(value);
        }
        true
    }

    fn confirm(&mut self, key: &str, text: &str) -> bool {
        let mut changed = false;
        for value in self.fields.get_mut(key).into_iter().flatten() {
            if value.text == text && !value.confirmed {
                value.confirmed = true;
                changed = true;
            }
        }
        changed
    }

    /// Deletes a value; learning never brings it back (REQ-ZPROF-003.A3).
    fn delete(&mut self, key: &str, text: &str) -> bool {
        let Some(values) = self.fields.get_mut(key) else {
            return false;
        };
        let before = values.len();
        values.retain(|v| v.text != text);
        let changed = values.len() != before;
        if changed {
            self.forgotten.insert(tomb(key, text));
        }
        if values.is_empty() {
            self.fields.remove(key);
        }
        changed
    }

    /// Lines for a prompt: usable values only, labelled.
    pub fn prompt_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for def in FIELDS {
            if !(if self.group { def.group } else { def.contact }) {
                continue;
            }
            let values: Vec<&str> = self
                .values(def.key)
                .iter()
                .filter(|v| v.usable())
                .map(|v| v.text.as_str())
                .collect();
            if !values.is_empty() {
                lines.push(format!("{}: {}", def.label, values.join("; ")));
            }
        }
        if self.group {
            let moods: Vec<String> = self
                .moods
                .iter()
                .filter_map(|(id, mood)| {
                    let name = self.active.iter().find(|a| a.id == *id)?.name.clone();
                    Some(format!("{name} {}", mood.now))
                })
                .collect();
            if !moods.is_empty() {
                lines.push(format!("Members' moods: {}", moods.join(", ")));
            }
            let active: Vec<String> = self
                .active
                .iter()
                .filter(|a| !a.former)
                .take(5)
                .map(|a| format!("{} ({})", a.name, a.count))
                .collect();
            if !active.is_empty() {
                lines.push(format!("Most active (30 days): {}", active.join(", ")));
            }
        }
        lines
    }

    /// The relationship in a few words ("your sister"), when known and
    /// usable.
    pub fn relation(&self) -> Option<String> {
        let value = self.first("relationship").filter(|v| v.usable())?;
        let text = value.text.trim().to_lowercase();
        let (base, detail) = match text.split_once('(') {
            Some((base, rest)) => (
                base.trim().to_owned(),
                rest.trim_end_matches(')').trim().to_owned(),
            ),
            None => (text.clone(), String::new()),
        };
        // "family (sister)" is "your sister"; "friend (since university)"
        // is "your friend".
        let role = !detail.is_empty()
            && detail.split_whitespace().count() <= 2
            && !detail
                .split_whitespace()
                .any(|w| matches!(w, "since" | "from" | "at" | "in" | "of" | "for" | "and"));
        match (base.as_str(), role) {
            ("family" | "colleague" | "client" | "partner" | "friend", true) => {
                Some(format!("your {detail}"))
            }
            ("friend" | "colleague" | "client" | "partner", false) => Some(format!("your {base}")),
            _ => None,
        }
    }

    /// Whether anything is known.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.active.is_empty() && self.moods.is_empty()
    }

    /// The trend of a mood: "up", "down", "same" or "" with nothing before.
    pub fn trend(now: &str, before: &str) -> &'static str {
        let rank = |m: &str| match mood_of(m) {
            Some("positive") => Some(2),
            Some("neutral") | Some("mixed") => Some(1),
            Some("negative") => Some(0),
            _ => None,
        };
        match (rank(now), rank(before)) {
            (Some(a), Some(b)) if a > b => "up",
            (Some(a), Some(b)) if a < b => "down",
            (Some(_), Some(_)) => "same",
            _ => "",
        }
    }
}

/// One learned item in a model answer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Item {
    pub key: String,
    pub value: String,
    pub sensitive: bool,
}

/// What the model says about one group member.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct MemberAnswer {
    pub tag: String,
    pub mood: String,
    pub items: Vec<Item>,
}

/// A profile answer: `{"profile": [...], "members": [...]}`.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Answer {
    pub profile: Vec<Item>,
    pub members: Vec<MemberAnswer>,
}

/// The JSON object in a model answer, or `None`.
pub fn parse(answer: &str) -> Option<Answer> {
    let start = answer.find('{')?;
    let end = answer.rfind('}')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&answer[start..=end]).ok()
}

/// A group member as the prompt names them: a short tag, so their number
/// never goes to the engine.
#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub tag: String,
    pub id: String,
    pub name: String,
}

/// A member id as the same person in every chat: the user part and the
/// server, without a device suffix.
pub fn member_key(sender: &str) -> String {
    let (user, server) = sender.split_once('@').unwrap_or((sender, "s.whatsapp.net"));
    let user = user.split(':').next().unwrap_or(user);
    format!("{user}@{server}")
}

/// Every profile Znga keeps, across accounts.
#[derive(Default)]
pub struct Profiles {
    list: Vec<Profile>,
    dirty: bool,
}

impl Profiles {
    /// Merged with anything learned before the file was read.
    pub fn restore(&mut self, loaded: Vec<Profile>) {
        for profile in loaded {
            if !self
                .list
                .iter()
                .any(|p| p.account == profile.account && p.id == profile.id)
            {
                self.list.push(profile);
            }
        }
    }

    pub fn get(&self, account: &str, id: &str) -> Option<&Profile> {
        self.list
            .iter()
            .find(|p| p.account == account && p.id == id)
    }

    fn entry(&mut self, account: &str, id: &str, title: &str, group: bool) -> &mut Profile {
        let index = match self
            .list
            .iter()
            .position(|p| p.account == account && p.id == id)
        {
            Some(index) => index,
            None => {
                self.list.push(Profile {
                    account: account.to_owned(),
                    id: id.to_owned(),
                    title: title.to_owned(),
                    group,
                    ..Profile::default()
                });
                self.list.len() - 1
            }
        };
        let profile = &mut self.list[index];
        if !title.is_empty() {
            profile.title = title.to_owned();
        }
        profile
    }

    /// Applies a model answer about `chat` (REQ-ZPROF-002). `tags` names
    /// the group members the prompt used. Returns how many values changed.
    #[allow(clippy::too_many_arguments)]
    pub fn apply(
        &mut self,
        account: &str,
        chat: &str,
        title: &str,
        group: bool,
        tags: &[Tag],
        answer: &Answer,
        now: i64,
    ) -> usize {
        let mut changed = 0;
        {
            let profile = self.entry(account, chat, title, group);
            for item in &answer.profile {
                let Some(def) = field_of(item.key.trim()) else {
                    continue;
                };
                if local(def.key) || !(if group { def.group } else { def.contact }) {
                    continue;
                }
                let text = match def.key {
                    "mood" => match mood_of(&item.value) {
                        Some(mood) => mood.to_owned(),
                        None => continue,
                    },
                    _ => item.value.clone(),
                };
                changed += usize::from(profile.learn(def.key, &text, item.sensitive, chat, now));
            }
            profile.learned_at = now;
            profile.pending = 0;
        }
        if !group {
            self.dirty = true;
            return changed;
        }
        for member in &answer.members {
            let Some(tag) = tags.iter().find(|t| t.tag == member.tag.trim()) else {
                continue;
            };
            if let Some(mood) = mood_of(&member.mood) {
                let profile = self.entry(account, chat, title, true);
                let entry = profile.moods.entry(tag.id.clone()).or_default();
                if entry.now != mood {
                    entry.before = std::mem::take(&mut entry.now);
                    entry.now = mood.to_owned();
                    changed += 1;
                }
            }
            let person = self.entry(account, &tag.id, "", false);
            if person.title.is_empty() {
                person.title = tag.name.clone();
            }
            for item in &member.items {
                let Some(def) = field_of(item.key.trim()) else {
                    continue;
                };
                // What a group shows about a member: never their
                // relationship to the person or how the person writes to
                // them, which only a direct chat shows.
                if local(def.key)
                    || !def.contact
                    || matches!(def.key, "relationship" | "formality" | "length" | "emoji")
                {
                    continue;
                }
                changed +=
                    usize::from(person.learn(def.key, &item.value, item.sensitive, chat, now));
            }
        }
        self.dirty = true;
        changed
    }

    /// A value computed on this computer (reply time, busy hours).
    #[allow(clippy::too_many_arguments)]
    pub fn set_local(
        &mut self,
        account: &str,
        id: &str,
        title: &str,
        group: bool,
        key: &str,
        text: &str,
        now: i64,
    ) {
        let profile = self.entry(account, id, title, group);
        let current = profile.first(key).map(|v| v.text.clone());
        if current.as_deref() != Some(text) && profile.learn(key, text, false, id, now) {
            self.dirty = true;
        }
    }

    /// A group's active members, counted on this computer
    /// (REQ-ZPROF-004.A1).
    pub fn set_active(&mut self, account: &str, id: &str, title: &str, active: Vec<Active>) {
        let profile = self.entry(account, id, title, true);
        if profile.active != active {
            profile.active = active;
            self.dirty = true;
        }
    }

    /// The person wrote a value (REQ-ZPROF-007).
    #[allow(clippy::too_many_arguments)]
    pub fn set(
        &mut self,
        account: &str,
        id: &str,
        title: &str,
        group: bool,
        key: &str,
        text: &str,
        now: i64,
    ) -> bool {
        let changed = self.entry(account, id, title, group).set(key, text, now);
        self.dirty |= changed;
        changed
    }

    pub fn confirm(&mut self, account: &str, id: &str, key: &str, text: &str) -> bool {
        let changed = self
            .list
            .iter_mut()
            .find(|p| p.account == account && p.id == id)
            .is_some_and(|p| p.confirm(key, text));
        self.dirty |= changed;
        changed
    }

    pub fn delete(&mut self, account: &str, id: &str, key: &str, text: &str) -> bool {
        let changed = self
            .list
            .iter_mut()
            .find(|p| p.account == account && p.id == id)
            .is_some_and(|p| p.delete(key, text));
        self.dirty |= changed;
        changed
    }

    /// Deletes the values of `id` that `phrase` names ("likes jazz"):
    /// every value sharing all of its words, or containing it. Returns the
    /// deleted texts.
    pub fn forget_matching(&mut self, account: &str, id: &str, phrase: &str) -> Vec<String> {
        let Some(profile) = self
            .list
            .iter_mut()
            .find(|p| p.account == account && p.id == id)
        else {
            return Vec::new();
        };
        let name = normal(&profile.title);
        let wanted = normal(phrase);
        let words: Vec<&str> = wanted
            .split_whitespace()
            .filter(|w| {
                w.len() > 2
                    && !name.split_whitespace().any(|n| n == *w)
                    && !matches!(
                        *w,
                        "that"
                            | "the"
                            | "and"
                            | "she"
                            | "her"
                            | "his"
                            | "him"
                            | "they"
                            | "their"
                            | "likes"
                            | "like"
                            | "loves"
                            | "love"
                            | "has"
                            | "have"
                            | "was"
                            | "are"
                    )
            })
            .collect();
        let mut found: Vec<(String, String)> = Vec::new();
        for (key, values) in &profile.fields {
            for value in values {
                let text = normal(&value.text);
                let matched = (!wanted.is_empty() && text.contains(&wanted))
                    || (!words.is_empty() && words.iter().all(|w| text.contains(w)));
                if matched {
                    found.push((key.clone(), value.text.clone()));
                }
            }
        }
        for (key, text) in &found {
            profile.delete(key, text);
        }
        if !found.is_empty() {
            self.dirty = true;
        }
        found.into_iter().map(|(_, text)| text).collect()
    }

    /// Forget all for one account (REQ-ZPROF-007.A2).
    pub fn forget_all(&mut self, account: &str) {
        let before = self.list.len();
        self.list.retain(|p| p.account != account);
        if self.list.len() != before {
            self.dirty = true;
        }
    }

    /// Drops profiles of chats Znga may no longer see and the values
    /// learned only from them, keeping values learned elsewhere
    /// (REQ-ZPROF-007.A3).
    pub fn retain_visible(&mut self, keep: impl Fn(&str, &str) -> bool) {
        let before = self.list.len();
        self.list.retain(|p| keep(&p.account, &p.id));
        let mut changed = self.list.len() != before;
        for profile in &mut self.list {
            let account = profile.account.clone();
            for values in profile.fields.values_mut() {
                let count = values.len();
                values.retain(|v| v.from.is_empty() || keep(&account, &v.from));
                changed |= values.len() != count;
            }
            profile.fields.retain(|_, values| !values.is_empty());
        }
        self.list
            .retain(|p| p.group || !p.fields.is_empty() || p.backfilled);
        if changed {
            self.dirty = true;
        }
    }

    /// A message arrived in `chat`.
    pub fn seen(&mut self, account: &str, chat: &str, title: &str, group: bool) {
        self.entry(account, chat, title, group).pending += 1;
    }

    /// Whether `chat` should be sent to the engine again
    /// (REQ-ZPROF-004.A3).
    pub fn due(&self, account: &str, chat: &str, group: bool, now: i64) -> bool {
        let Some(profile) = self.get(account, chat) else {
            return false;
        };
        if group {
            profile.pending >= REFRESH_MESSAGES
                || (profile.pending > 0
                    && (profile.learned_at == 0
                        || now - profile.learned_at >= REFRESH_DAYS * 86_400))
        } else {
            profile.pending >= CONTACT_MESSAGES || (profile.learned_at == 0 && profile.pending > 0)
        }
    }

    /// A failed update: due again in the next pass (REQ-ZPROF-002.A5).
    pub fn retry(&mut self, account: &str, chat: &str, group: bool) {
        if let Some(profile) = self
            .list
            .iter_mut()
            .find(|p| p.account == account && p.id == chat)
        {
            let floor = if group {
                REFRESH_MESSAGES
            } else {
                CONTACT_MESSAGES
            };
            profile.pending = profile.pending.max(floor);
        }
    }

    /// The update was sent: new messages count from here.
    pub fn sent(&mut self, account: &str, chat: &str) {
        if let Some(profile) = self
            .list
            .iter_mut()
            .find(|p| p.account == account && p.id == chat)
        {
            profile.pending = 0;
        }
    }

    pub fn mark_backfilled(&mut self, account: &str, chat: &str, title: &str, group: bool) {
        let profile = self.entry(account, chat, title, group);
        if !profile.backfilled {
            profile.backfilled = true;
            self.dirty = true;
        }
    }

    pub fn backfilled(&self, account: &str, chat: &str) -> bool {
        self.get(account, chat).is_some_and(|p| p.backfilled)
    }

    /// "Learn from history" on one profile runs the read again.
    pub fn reset_backfill(&mut self, account: &str, chat: &str) {
        if let Some(profile) = self
            .list
            .iter_mut()
            .find(|p| p.account == account && p.id == chat)
        {
            profile.backfilled = false;
            self.dirty = true;
        }
    }

    /// Records a skipped question so it is never asked again.
    pub fn skip_question(&mut self, account: &str, id: &str, key: &str) {
        let profile = self.entry(account, id, "", false);
        profile.forgotten.insert(format!("ask\u{1f}{key}"));
        self.dirty = true;
    }

    /// The next gap worth asking about (REQ-ZPROF-006): a contact the
    /// person talks with whose relationship is unknown. `chats` are the
    /// visible direct chats as (id, title), busiest first.
    pub fn next_question(
        &self,
        account: &str,
        chats: &[(String, String)],
    ) -> Option<(String, String, &'static str)> {
        chats.iter().find_map(|(id, title)| {
            let profile = self.get(account, id)?;
            if profile.group || profile.learned_at == 0 {
                return None;
            }
            let asked = profile.forgotten.contains("ask\u{1f}relationship");
            let known = profile.first("relationship").is_some_and(|v| v.confirmed);
            (!asked && !known).then(|| (id.clone(), title.clone(), "relationship"))
        })
    }

    /// One account's profiles, by title.
    pub fn for_account(&self, account: &str) -> Vec<&Profile> {
        let mut list: Vec<&Profile> = self
            .list
            .iter()
            .filter(|p| p.account == account && !p.is_empty())
            .collect();
        list.sort_by_key(|p| p.title.to_lowercase());
        list
    }

    pub fn all(&self) -> &[Profile] {
        &self.list
    }

    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

/// A group's members ranked by messages in the last 30 days. `messages`
/// are (member id, name, time); `participants` the current member ids
/// (empty when unknown).
pub fn activity(
    messages: &[(String, String, i64)],
    participants: &[String],
    now: i64,
) -> Vec<Active> {
    let since = now - ACTIVE_DAYS * 86_400;
    let current: BTreeSet<String> = participants.iter().map(|p| member_key(p)).collect();
    let mut counts: BTreeMap<String, (String, u32)> = BTreeMap::new();
    for (id, name, at) in messages {
        if *at < since {
            continue;
        }
        let entry = counts
            .entry(member_key(id))
            .or_insert_with(|| (name.clone(), 0));
        entry.1 += 1;
        if entry.0.is_empty() {
            entry.0 = name.clone();
        }
    }
    let mut active: Vec<Active> = counts
        .into_iter()
        .map(|(id, (name, count))| Active {
            former: !current.is_empty() && !current.contains(&id),
            id,
            name,
            count,
        })
        .collect();
    active.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));
    active
}

/// How quickly the person usually answers, from (time, from me), oldest
/// first: the median gap from their message to the person's reply.
pub fn reply_time(times: &[(i64, bool)]) -> Option<String> {
    let mut gaps: Vec<i64> = times
        .windows(2)
        .filter(|pair| !pair[0].1 && pair[1].1)
        .map(|pair| pair[1].0 - pair[0].0)
        .filter(|gap| *gap >= 0)
        .collect();
    if gaps.len() < 3 {
        return None;
    }
    gaps.sort_unstable();
    let median = gaps[gaps.len() / 2];
    Some(
        match median {
            0..600 => "usually within minutes",
            600..3600 => "usually within the hour",
            3600..21_600 => "usually within a few hours",
            21_600..86_400 => "usually the same day",
            _ => "often after a day or more",
        }
        .to_owned(),
    )
}

/// When a group is busiest: the three-hour local window with most
/// messages, from their times and the local offset in seconds.
pub fn busy_hours(times: &[i64], offset: i64) -> Option<String> {
    if times.len() < 20 {
        return None;
    }
    let mut hours = [0u32; 24];
    for at in times {
        hours[((at + offset).rem_euclid(86_400) / 3600) as usize] += 1;
    }
    let start = (0..24)
        .max_by_key(|h| {
            (
                hours[*h] + hours[(h + 1) % 24] + hours[(h + 2) % 24],
                hours[*h],
            )
        })
        .unwrap_or(0);
    Some(format!(
        "busiest {:02}:00–{:02}:00",
        start,
        (start + 3) % 24
    ))
}

/// The one-time history read (REQ-ZPROF-002).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BackfillState {
    #[default]
    Idle,
    Running,
    Paused,
    /// Waiting for something outside Znga; the reason says what.
    Waiting(String),
    Done,
}

#[derive(Default)]
pub struct Backfill {
    pub state: BackfillState,
    /// (account, chat, title, group), newest chats first.
    pub queue: VecDeque<(String, String, String, bool)>,
    pub total: usize,
    pub done: usize,
    /// The chat being read now.
    pub current: Option<(String, String)>,
}

impl Backfill {
    pub fn start(&mut self, queue: Vec<(String, String, String, bool)>) {
        self.total = queue.len();
        self.done = 0;
        self.queue = queue.into();
        self.current = None;
        self.state = if self.total == 0 {
            BackfillState::Done
        } else {
            BackfillState::Running
        };
    }

    /// Cancel keeps what was learned (REQ-ZPROF-002.A3).
    pub fn cancel(&mut self) {
        self.queue.clear();
        self.current = None;
        self.state = BackfillState::Idle;
    }

    pub fn finished(&mut self, account: &str, chat: &str) {
        if self
            .current
            .as_ref()
            .is_some_and(|(a, c)| a == account && c == chat)
        {
            self.current = None;
            self.done += 1;
            if self.queue.is_empty() && self.state == BackfillState::Running {
                self.state = BackfillState::Done;
            }
        }
    }

    pub fn active(&self) -> bool {
        matches!(
            self.state,
            BackfillState::Running | BackfillState::Waiting(_)
        )
    }
}

/// How the profile prompt asks for fields, for a contact or a group.
pub fn schema(group: bool) -> String {
    FIELDS
        .iter()
        .filter(|f| !local(f.key) && if group { f.group } else { f.contact })
        .map(|f| {
            format!(
                "- \"{}\"{}: {}",
                f.key,
                if f.single { "" } else { " (several allowed)" },
                f.hint
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The member fields a group prompt may return.
pub fn member_schema() -> String {
    FIELDS
        .iter()
        .filter(|f| {
            f.contact
                && !local(f.key)
                && !matches!(f.key, "relationship" | "formality" | "length" | "emoji")
        })
        .map(|f| format!("\"{}\"", f.key))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(key: &str, value: &str) -> Item {
        Item {
            key: key.into(),
            value: value.into(),
            sensitive: false,
        }
    }

    fn contact(items: Vec<Item>) -> Answer {
        Answer {
            profile: items,
            members: Vec::new(),
        }
    }

    #[test]
    fn answers_are_parsed_from_the_model_text() {
        let text = "Sure:\n{\"profile\": [{\"key\": \"relationship\", \"value\": \"friend\"}], \"members\": []}";
        let answer = parse(text).unwrap();
        assert_eq!(answer.profile, vec![item("relationship", "friend")]);
        assert!(parse("nothing here").is_none());
    }

    #[test]
    fn a_confirmed_value_is_never_overwritten_by_learning() {
        let mut profiles = Profiles::default();
        profiles.set(
            "1",
            "ada",
            "Ada",
            false,
            "relationship",
            "family (sister)",
            1,
        );
        profiles.apply(
            "1",
            "ada",
            "Ada",
            false,
            &[],
            &contact(vec![item("relationship", "colleague")]),
            2,
        );
        let value = profiles
            .get("1", "ada")
            .unwrap()
            .first("relationship")
            .unwrap()
            .clone();
        assert_eq!(value.text, "family (sister)");
        assert!(value.confirmed);
    }

    #[test]
    fn the_same_id_on_two_accounts_is_two_profiles() {
        let mut profiles = Profiles::default();
        profiles.apply(
            "1",
            "ada",
            "Ada",
            false,
            &[],
            &contact(vec![item("likes", "jazz")]),
            1,
        );
        assert!(profiles.get("2", "ada").is_none());
        profiles.apply(
            "2",
            "ada",
            "Ada",
            false,
            &[],
            &contact(vec![item("likes", "tea")]),
            1,
        );
        assert_eq!(
            profiles.get("1", "ada").unwrap().values("likes")[0].text,
            "jazz"
        );
        assert_eq!(
            profiles.get("2", "ada").unwrap().values("likes")[0].text,
            "tea"
        );
    }

    #[test]
    fn sensitive_values_stay_out_of_prompts_until_confirmed() {
        let mut profiles = Profiles::default();
        let answer = contact(vec![
            Item {
                key: "notes".into(),
                value: "Goes to church on Sundays".into(),
                sensitive: true,
            },
            item("likes", "hiking"),
        ]);
        profiles.apply("1", "ada", "Ada", false, &[], &answer, 1);
        let lines = profiles.get("1", "ada").unwrap().prompt_lines().join("\n");
        assert!(lines.contains("hiking"));
        assert!(!lines.contains("church"));
        profiles.confirm("1", "ada", "notes", "Goes to church on Sundays");
        let lines = profiles.get("1", "ada").unwrap().prompt_lines().join("\n");
        assert!(lines.contains("church"));
        // "still" is not "ill".
        profiles.apply(
            "1",
            "bo",
            "Bo",
            false,
            &[],
            &contact(vec![item("notes", "Lunch is still open")]),
            1,
        );
        assert!(!profiles.get("1", "bo").unwrap().values("notes")[0].sensitive);
        // Marked by its words even when the model did not mark it.
        profiles.apply(
            "1",
            "sam",
            "Sam",
            false,
            &[],
            &contact(vec![item("notes", "Has diabetes")]),
            1,
        );
        assert!(profiles.get("1", "sam").unwrap().values("notes")[0].sensitive);
    }

    #[test]
    fn a_deleted_value_is_not_learned_again() {
        let mut profiles = Profiles::default();
        let answer = contact(vec![item("likes", "jazz")]);
        profiles.apply("1", "ada", "Ada", false, &[], &answer, 1);
        assert_eq!(
            profiles.forget_matching("1", "ada", "Ada likes jazz"),
            vec!["jazz"]
        );
        profiles.apply("1", "ada", "Ada", false, &[], &answer, 2);
        assert!(profiles.get("1", "ada").unwrap().values("likes").is_empty());
    }

    #[test]
    fn a_member_seen_in_two_groups_is_one_profile_shared_with_their_chat() {
        let mut profiles = Profiles::default();
        let tags = vec![Tag {
            tag: "m1".into(),
            id: "44700:3@s.whatsapp.net".into(),
            name: "Ada".into(),
        }];
        let tags: Vec<Tag> = tags
            .into_iter()
            .map(|t| Tag {
                id: member_key(&t.id),
                ..t
            })
            .collect();
        let group = |value: &str| Answer {
            profile: vec![item("theme", "school")],
            members: vec![MemberAnswer {
                tag: "m1".into(),
                mood: "positive".into(),
                items: vec![item("likes", value), item("relationship", "friend")],
            }],
        };
        profiles.apply("1", "g1", "School", true, &tags, &group("chess"), 1);
        profiles.apply("1", "g2", "Book club", true, &tags, &group("novels"), 1);
        let member = profiles.get("1", "44700@s.whatsapp.net").unwrap();
        let likes: Vec<&str> = member
            .values("likes")
            .iter()
            .map(|v| v.text.as_str())
            .collect();
        assert_eq!(likes, vec!["chess", "novels"]);
        // A group never says who they are to the person.
        assert!(member.first("relationship").is_none());
        assert_eq!(
            profiles.get("1", "g1").unwrap().moods["44700@s.whatsapp.net"].now,
            "positive"
        );
        // Excluding one group keeps what the other taught.
        profiles.retain_visible(|_, chat| chat != "g1");
        assert!(profiles.get("1", "g1").is_none());
        let member = profiles.get("1", "44700@s.whatsapp.net").unwrap();
        let likes: Vec<&str> = member
            .values("likes")
            .iter()
            .map(|v| v.text.as_str())
            .collect();
        assert_eq!(likes, vec!["novels"]);
    }

    #[test]
    fn active_members_are_counted_over_thirty_days_with_former_members_marked() {
        let now = 100 * 86_400;
        let messages = vec![
            ("a@s.whatsapp.net".to_owned(), "Ada".to_owned(), now - 10),
            ("a@s.whatsapp.net".to_owned(), "Ada".to_owned(), now - 20),
            ("b@s.whatsapp.net".to_owned(), "Ben".to_owned(), now - 5),
            (
                "b@s.whatsapp.net".to_owned(),
                "Ben".to_owned(),
                now - 40 * 86_400,
            ),
        ];
        let active = activity(&messages, &["a@s.whatsapp.net".into()], now);
        assert_eq!(active[0].name, "Ada");
        assert_eq!(active[0].count, 2);
        assert!(!active[0].former);
        assert_eq!(active[1].count, 1);
        assert!(active[1].former);
    }

    #[test]
    fn groups_refresh_only_after_enough_has_happened() {
        let mut profiles = Profiles::default();
        let answer = Answer {
            profile: vec![item("theme", "school")],
            members: vec![],
        };
        profiles.apply("1", "g", "School", true, &[], &answer, 1_000);
        for _ in 0..10 {
            profiles.seen("1", "g", "School", true);
        }
        assert!(!profiles.due("1", "g", true, 2_000));
        for _ in 0..40 {
            profiles.seen("1", "g", "School", true);
        }
        assert!(profiles.due("1", "g", true, 2_000));
    }

    #[test]
    fn a_skipped_question_is_not_asked_again() {
        let mut profiles = Profiles::default();
        profiles.apply(
            "1",
            "ada",
            "Ada",
            false,
            &[],
            &contact(vec![item("likes", "tea")]),
            1,
        );
        let chats = vec![("ada".to_owned(), "Ada".to_owned())];
        assert!(profiles.next_question("1", &chats).is_some());
        profiles.skip_question("1", "ada", "relationship");
        assert!(profiles.next_question("1", &chats).is_none());
    }

    #[test]
    fn reply_time_and_busy_hours_are_read_locally() {
        let times = vec![
            (0, false),
            (60, true),
            (100, false),
            (200, true),
            (300, false),
            (400, true),
        ];
        assert_eq!(
            reply_time(&times).as_deref(),
            Some("usually within minutes")
        );
        let busy: Vec<i64> = (0..30).map(|n| 19 * 3600 + n * 60).collect();
        assert_eq!(busy_hours(&busy, 0).as_deref(), Some("busiest 19:00–22:00"));
    }

    #[test]
    fn relation_reads_naturally() {
        let mut profiles = Profiles::default();
        profiles.set(
            "1",
            "ada",
            "Ada",
            false,
            "relationship",
            "family (sister)",
            1,
        );
        assert_eq!(
            profiles.get("1", "ada").unwrap().relation().as_deref(),
            Some("your sister")
        );
        profiles.set("1", "bo", "Bo", false, "relationship", "colleague", 1);
        assert_eq!(
            profiles.get("1", "bo").unwrap().relation().as_deref(),
            Some("your colleague")
        );
    }
}
