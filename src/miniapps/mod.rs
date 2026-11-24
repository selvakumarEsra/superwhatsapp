//! Mini-apps (specs/miniapps.md): small apps that live in one chat.
//!
//! An app never touches the account. It is handed events addressed to it
//! ([`Event`]) and answers with actions ([`AppAction`]); the [`host`]
//! checks each action with the Znga Sentinel, applies its caps, and carries
//! it out. Apps are built in: [`registry`] lists them.

#[cfg(any(test, feature = "demo"))]
pub mod demo;
pub mod goal;
pub mod hooks;
pub mod host;
pub mod store;
pub mod ui;

use std::collections::BTreeMap;

/// What an app may do, declared in its manifest (REQ-MINIAPP-001).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Send messages to the chat it is installed in.
    Send,
    /// Act on commands and replies from other people in that chat.
    AnswerMembers,
    /// Ask the configured Znga engine for completions.
    UseZnga,
    /// Read Znga's recent activity, promises and found plans.
    ReadActivity,
}

impl Permission {
    /// The permission in plain words, for the install screen.
    pub fn describe(self) -> &'static str {
        match self {
            Permission::Send => "Send messages to this chat, starting with its name",
            Permission::AnswerMembers => "Act on commands and replies from other people here",
            Permission::UseZnga => "Ask Znga (your configured AI engine) for help",
            Permission::ReadActivity => {
                "Read Znga's recent activity, promises and plans from visible chats"
            }
        }
    }
}

/// The kinds of chat an app can be installed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChatKind {
    /// The person's own "Message yourself" chat.
    Own,
    Direct,
    Group,
}

impl ChatKind {
    pub fn label(self) -> &'static str {
        match self {
            ChatKind::Own => "your own chat",
            ChatKind::Direct => "one-to-one chats",
            ChatKind::Group => "groups",
        }
    }
}

/// How an app describes itself (REQ-MINIAPP-001).
#[derive(Debug, PartialEq)]
pub struct Manifest {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    /// The word after `/` it answers to in chats.
    pub command: &'static str,
    pub kinds: &'static [ChatKind],
    pub permissions: &'static [Permission],
}

impl Manifest {
    pub fn allows(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    /// What every message the app sends starts with (REQ-MINIAPP-004.A3).
    pub fn marker(&self) -> String {
        format!("{} {}", self.icon, self.name)
    }

    /// The text after `/command`, when `text` is a command for this app.
    pub fn command_in<'a>(&self, text: &'a str) -> Option<&'a str> {
        let rest = text.trim_start().strip_prefix('/')?;
        let word_end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
        rest[..word_end]
            .eq_ignore_ascii_case(self.command)
            .then(|| rest[word_end..].trim())
    }
}

/// Who caused an event.
#[derive(Clone, Debug, PartialEq)]
pub enum From {
    /// The person, in ZapFast or on their phone.
    Me,
    /// Another person in the chat (their id and name).
    Member { id: String, name: String },
}

/// One control on a card.
#[derive(Clone, Debug, PartialEq)]
pub enum Control {
    Button(String),
    /// A step was ticked (`true`) or unticked.
    Step(usize, bool),
}

/// Something addressed to an app (REQ-MINIAPP-003).
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Just installed.
    Installed,
    /// `/command text`.
    Command { text: String, from: From },
    /// A quote-reply to one of the app's messages.
    Reply { text: String, from: From },
    /// A click on a card; `fields` holds the card's text fields.
    Click {
        card: String,
        control: Control,
        fields: BTreeMap<String, String>,
    },
    /// A wake-up the app asked for. `late` when it passed while ZapFast
    /// was closed (REQ-MINIAPP-003.A5).
    Wake { at: i64, late: bool },
    /// Znga's answer to [`AppAction::AskZnga`] (REQ-MINIAPP-007.A2).
    Answer {
        tag: String,
        answer: Result<String, String>,
    },
    /// What [`AppAction::ReadActivity`] found.
    Activity { tag: String, activity: Activity },
    /// A store write was refused (REQ-MINIAPP-006.A4).
    StoreFull { key: String },
    /// A refused action, so the app can tell the person.
    Refused { action: String, reason: String },
}

/// Znga's view of what is going on, from visible chats only.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Activity {
    /// Open promises due today or tomorrow.
    pub promises: Vec<String>,
    /// Plans found for today.
    pub plans: Vec<String>,
    /// Recent messages, as "chat: sender: text".
    pub recent: Vec<String>,
}

/// A text field on a card; its value comes with every click.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub id: String,
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Button {
    pub id: String,
    pub label: String,
}

impl Button {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// What an app shows in the Mini-apps window (REQ-MINIAPP-005).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Card {
    pub id: String,
    pub title: String,
    pub lines: Vec<String>,
    /// 0.0 ..= 1.0.
    pub progress: Option<f32>,
    pub steps: Vec<(String, bool)>,
    pub fields: Vec<Field>,
    /// The first [`Card::MAX_BUTTONS`] are shown.
    pub buttons: Vec<Button>,
}

impl Card {
    pub const MAX_BUTTONS: usize = 4;

    pub fn shown_buttons(&self) -> &[Button] {
        &self.buttons[..self.buttons.len().min(Self::MAX_BUTTONS)]
    }
}

/// What an app asks the host to do (REQ-MINIAPP-004).
#[derive(Clone, Debug, PartialEq)]
pub enum AppAction {
    /// Send `text` to the chat the event came from; the host puts the
    /// app's marker first.
    Send { text: String },
    /// Replace the app's cards.
    Cards(Vec<Card>),
    /// Ask Znga; the answer comes back as [`Event::Answer`].
    AskZnga {
        tag: String,
        system: String,
        user: String,
    },
    /// Read Znga's activity; it comes back as [`Event::Activity`].
    ReadActivity { tag: String },
    /// Wake the app at this time (replaces any earlier wake-up).
    WakeAt(i64),
    /// Store a value, or remove it with `None`.
    Store { key: String, value: Option<String> },
    /// Something went wrong: the app is turned off in this install
    /// (REQ-MINIAPP-007.A1).
    Fail(String),
}

/// What an app is handed with every event.
pub struct Ctx<'a> {
    pub now: i64,
    pub zone: &'a jiff::tz::TimeZone,
    /// Whether Znga is on and can be asked.
    pub znga_on: bool,
    /// This install's stored values.
    pub data: &'a BTreeMap<String, String>,
}

/// A built-in app. Apps keep no state of their own: everything lasting
/// goes through [`AppAction::Store`].
pub trait MiniApp: Send {
    fn manifest(&self) -> &'static Manifest;
    fn handle(&self, event: &Event, ctx: &Ctx) -> Vec<AppAction>;
}

/// Every built-in app.
pub fn registry() -> Vec<Box<dyn MiniApp>> {
    vec![Box::new(goal::GoalApp)]
}

/// Whether a text in a chat belongs to a built-in app: one of its
/// messages or a command for it. Such texts are never questions for Znga
/// (REQ-MINIAPP-008.A2).
pub fn is_app_text(text: &str) -> bool {
    registry().iter().any(|app| {
        let manifest = app.manifest();
        text.trim_start().starts_with(&manifest.marker()) || manifest.command_in(text).is_some()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_built_in_app_has_a_unique_manifest() {
        let apps = registry();
        let mut ids = HashSet::new();
        let mut commands = HashSet::new();
        for app in &apps {
            let manifest = app.manifest();
            assert!(!manifest.name.is_empty() && !manifest.icon.is_empty());
            assert!(!manifest.description.is_empty() && !manifest.kinds.is_empty());
            assert!(!manifest.permissions.is_empty());
            assert!(ids.insert(manifest.id));
            assert!(commands.insert(manifest.command.to_lowercase()));
        }
    }

    #[test]
    fn commands_are_found_only_with_the_command_word() {
        let manifest = goal::GoalApp.manifest();
        assert_eq!(manifest.command_in("/goal done"), Some("done"));
        assert_eq!(manifest.command_in("  /Goal   ship it "), Some("ship it"));
        assert_eq!(manifest.command_in("/goal"), Some(""));
        assert_eq!(manifest.command_in("done"), None);
        assert_eq!(manifest.command_in("/goals done"), None);
        assert!(is_app_text("🎯 Goal of the day\nhi"));
        assert!(!is_app_text("✨ Znga\nhi"));
    }

    #[test]
    fn cards_show_at_most_four_buttons() {
        let card = Card {
            buttons: (0..6).map(|i| Button::new(&i.to_string(), "b")).collect(),
            ..Card::default()
        };
        assert_eq!(card.shown_buttons().len(), 4);
        assert_eq!(card.shown_buttons()[3].id, "3");
    }
}
