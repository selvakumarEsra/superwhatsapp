//! Znga: SuperWhatsapp's on-device AI companion (fork-only, feature `znga`).
//!
//! Znga briefs the person on what happened, flags what needs them, and
//! drafts messages in their voice (the "digital twin"). It never sends to
//! anyone but the person themselves without a click: see `specs/znga.md`.
//!
//! Layout, so upstream merges stay easy: everything lives in this folder.
//! Upstream files only call [`hooks`] and [`ui`] from lines marked
//! `znga hook`.
//!
//! - [`config`]: `znga.json`, separate from SuperWhatsapp's settings
//! - [`keys`]: API keys from the environment or the OS keyring
//! - [`provider`] and [`runner`]: model calls on a worker thread
//! - [`triage`], [`proactive`]: alerts, briefings, notes to self, check-ins
//! - [`style`], [`drafts`], [`prompt`]: the twin
//! - [`privacy`]: which chats Znga may see at all
//! - [`sentinel`]: the one gate every Znga action passes, with its log
//! - [`store`]: `znga.db`, Znga's encrypted memory, and archive reads
//! - [`memory`], [`promises`], [`coach`], [`calendar`], [`voice`],
//!   [`archive_ask`], [`selfchat`]: the companion's advanced features
//!   (`specs/znga-advanced.md`)
//! - [`hooks`]: the bridge to [`crate::app::App`]
//! - [`ui`]: the Znga panel and its rail button

pub mod archive_ask;
pub mod calendar;
pub mod coach;
pub mod config;
#[cfg(any(test, feature = "demo"))]
pub mod demo;
pub mod drafts;
pub mod hooks;
pub mod keys;
pub mod memory;
pub mod privacy;
pub mod proactive;
pub mod promises;
pub mod prompt;
pub mod provider;
pub mod runner;
pub mod selfchat;
pub mod sentinel;
pub mod store;
pub mod style;
pub mod triage;
pub mod ui;
pub mod update;
pub mod voice;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use config::{Config, chat_key};
use drafts::{Draft, Drafts, Kind};
use proactive::{Activity, Alert, SelfNotes};
use provider::{Prompt, Target, Turn};
use style::{Examples, StyleProfile};

/// What a model answer was for.
#[derive(Clone, Debug, PartialEq)]
pub enum Purpose {
    Briefing {
        proactive: bool,
    },
    Draft {
        kind: Kind,
        account: String,
        chat: String,
        chat_title: String,
    },
    /// An "update my people" instruction; `chats` are the account's
    /// (chat id, title) pairs when it was asked.
    FanOut {
        account: String,
        chats: Vec<(String, String)>,
    },
    Ask,
    /// Step 1 of asking (REQ-ZNGA-020): what to search the archive for.
    Plan {
        ask: u64,
    },
    /// Step 2: the answer, from recent activity and archive hits.
    Answer {
        ask: u64,
    },
    Promises {
        account: String,
        chat: String,
        chat_title: String,
    },
    Facts {
        account: String,
        chat: String,
        chat_title: String,
    },
    CatchUp {
        account: String,
        chat: String,
        chat_title: String,
    },
    Events {
        account: String,
        chat: String,
        chat_title: String,
    },
    Voice {
        note: u64,
    },
    ExplainEdit {
        key: String,
    },
    Test,
}

impl Purpose {
    /// Requests that run on the background lane (REQ-ZNGA-015.A2).
    pub fn background(&self) -> bool {
        matches!(
            self,
            Purpose::Briefing { proactive: true }
                | Purpose::Draft {
                    kind: Kind::AutoReply | Kind::CheckIn,
                    ..
                }
                | Purpose::Promises { .. }
                | Purpose::Facts { .. }
        )
    }
}

/// Something [`hooks`] must do in the app after an answer.
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// A desktop notification titled for Znga. `message` deep-links to it.
    Notify {
        title: String,
        body: String,
        account: String,
        chat: String,
        message: Option<String>,
        only_when_away: bool,
    },
    /// Send `text` to our own chat (proactive briefings, when enabled).
    SelfNote(String),
    /// Answer in the person's own chat (REQ-ZNGA-012).
    SendOwn {
        account: String,
        chat: String,
        text: String,
    },
    /// Search the archive for an ask in progress (REQ-ZNGA-020).
    Search { ask: u64, terms: Vec<String> },
}

/// A catch-up on one chat (REQ-ZNGA-021).
#[derive(Clone, Debug, PartialEq)]
pub struct CatchUp {
    pub account: String,
    pub chat: String,
    pub chat_title: String,
    pub summary: String,
    pub at: i64,
}

/// An event found in a chat, waiting for "Add to calendar" (REQ-ZNGA-022).
#[derive(Clone, Debug, PartialEq)]
pub struct FoundEvent {
    pub chat_title: String,
    pub event: calendar::Event,
}

/// A transcription finished on its thread: (voice note id, transcript).
pub type Transcribed = (u64, Result<String, String>);

/// A voice note Znga may summarise (REQ-ZNGA-023).
#[derive(Clone, Debug, PartialEq)]
pub struct VoiceNote {
    pub id: u64,
    pub account: String,
    pub chat: String,
    pub chat_title: String,
    pub sender_name: String,
    pub message: String,
    pub seconds: Option<u32>,
    pub at: i64,
    pub state: VoiceState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VoiceState {
    New,
    Working,
    Done { transcript: String, summary: String },
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Briefing {
    pub text: String,
    pub at: i64,
    pub proactive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Today,
    Drafts,
    Ask,
    /// Facts, promises and the Sentinel's log.
    Memory,
    Setup,
}

/// A status line under the panel's tabs.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub text: String,
    pub error: bool,
}

const MAX_ALERTS: usize = 50;
const MAX_BRIEFINGS: usize = 20;

pub struct Znga {
    pub config: Config,
    config_path: PathBuf,
    style_path: PathBuf,
    /// Empty folder for the Claude Code engine to run in.
    workdir: PathBuf,
    dirty: bool,
    pub open: bool,
    pub tab: Tab,
    pub activity: Activity,
    /// Newest last.
    pub alerts: Vec<Alert>,
    /// Newest last.
    pub briefings: Vec<Briefing>,
    pub drafts: Drafts,
    pub examples: Examples,
    /// Per account id.
    pub styles: HashMap<String, StyleProfile>,
    /// The Ask thread.
    pub ask: Vec<Turn>,
    pub ask_input: String,
    pub update_input: String,
    pub translate_to: String,
    pub key_input: String,
    /// The Setup page's chat search.
    pub chat_filter: String,
    pub status: Option<Status>,
    /// Alerts, briefings and drafts that arrived while the panel was shut.
    pub unseen: u32,
    pub runner: runner::Runner<Purpose>,
    self_notes: SelfNotes,
    pub(crate) checkins_suggested: HashMap<String, i64>,
    pub(crate) last_scan: Option<Instant>,
    pub(crate) last_checkin_scan: Option<Instant>,
    /// (engine, key) read once per engine instead of on every request.
    key_cache: Option<(config::Engine, Option<String>)>,
    pub store: store::Store,
    pub audit: sentinel::Audit,
    pub people: memory::People,
    pub promises: promises::Promises,
    /// Asks in progress (plan → search → answer).
    pub asking: HashMap<u64, archive_ask::Asking>,
    next_ask: u64,
    pub selfchat: selfchat::SelfChat,
    /// Newest last.
    pub catch_ups: Vec<CatchUp>,
    pub events: Vec<FoundEvent>,
    /// Newest last.
    pub voice_notes: Vec<VoiceNote>,
    next_voice: u64,
    pub(crate) voice_results: (
        std::sync::mpsc::Sender<Transcribed>,
        std::sync::mpsc::Receiver<Transcribed>,
    ),
    pub rhythms: Vec<coach::Rhythm>,
    /// Edit explanations by kept-copy key; not saved (REQ-ZNGA-026.A3).
    pub explanations: HashMap<String, Result<String, String>>,
    /// The account whose Ask thread is in [`Self::ask`].
    pub ask_account: Option<String>,
    pub(crate) ask_dirty: bool,
    /// Accounts whose `znga.db` was asked for.
    pub(crate) loaded: HashSet<String>,
    /// Accounts whose `znga.db` came back; only these are ever saved, so
    /// memory found before loading never overwrites what was saved.
    pub(crate) restored: HashSet<String>,
    /// Newest message time each chat was mined for promises and facts.
    pub(crate) extracted: HashMap<String, i64>,
    pub(crate) last_extract: Option<Instant>,
    pub(crate) last_rhythms: Option<Instant>,
    pub(crate) last_promise_check: Option<Instant>,
}

impl Znga {
    pub fn load(config_dir: &Path, cache_dir: &Path, waker: crate::backend::Waker) -> Self {
        let config_path = Config::path(config_dir);
        let style_path = config_dir.join("znga-style.json");
        let styles = std::fs::read_to_string(&style_path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        let mut config = Config::load(&config_path);
        if !cfg!(test) && !config_path.exists() && provider::find_claude("").is_some() {
            // First run with Claude Code installed: offer it as the engine.
            // Znga stays off, and asks before sending anything to it.
            config.engine = config::Engine::ClaudeCode;
        }
        Self {
            config,
            config_path,
            style_path,
            workdir: cache_dir.join("znga-claude-code"),
            dirty: false,
            open: false,
            tab: Tab::default(),
            activity: Activity::default(),
            alerts: Vec::new(),
            briefings: Vec::new(),
            drafts: Drafts::default(),
            examples: Examples::default(),
            styles,
            ask: Vec::new(),
            ask_input: String::new(),
            update_input: String::new(),
            translate_to: "English".into(),
            key_input: String::new(),
            chat_filter: String::new(),
            status: None,
            unseen: 0,
            runner: runner::Runner::new(waker.clone()),
            self_notes: SelfNotes::default(),
            checkins_suggested: HashMap::new(),
            last_scan: None,
            last_checkin_scan: None,
            key_cache: None,
            store: store::Store::new(waker.clone()),
            audit: sentinel::Audit::default(),
            people: memory::People::default(),
            promises: promises::Promises::default(),
            asking: HashMap::new(),
            next_ask: 0,
            selfchat: selfchat::SelfChat::default(),
            catch_ups: Vec::new(),
            events: Vec::new(),
            voice_notes: Vec::new(),
            next_voice: 0,
            voice_results: std::sync::mpsc::channel(),
            rhythms: Vec::new(),
            explanations: HashMap::new(),
            ask_account: None,
            ask_dirty: false,
            loaded: HashSet::new(),
            restored: HashSet::new(),
            extracted: HashMap::new(),
            last_extract: None,
            last_rhythms: None,
            last_promise_check: None,
        }
    }

    /// Starts an ask; returns its id.
    pub fn begin_ask(&mut self, asking: archive_ask::Asking) -> u64 {
        self.next_ask += 1;
        self.asking.insert(self.next_ask, asking);
        self.next_ask
    }

    pub fn add_voice_note(&mut self, mut note: VoiceNote) {
        if self
            .voice_notes
            .iter()
            .any(|n| n.message == note.message && n.chat == note.chat)
        {
            return;
        }
        self.next_voice += 1;
        note.id = self.next_voice;
        self.voice_notes.push(note);
        if self.voice_notes.len() > 20 {
            self.voice_notes.remove(0);
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Hands `znga.json` to the app's file writer when something changed.
    pub fn save_if_dirty(&mut self, files: &fastframe_file::Writer) {
        if !std::mem::take(&mut self.dirty) {
            return;
        }
        if self.config.save_to(files, &self.config_path).is_err() {
            log::warn!("znga: could not save its settings");
        }
    }

    /// Style statistics only: numbers, never message text.
    pub fn save_styles(&self, files: &fastframe_file::Writer) {
        if let Ok(text) = serde_json::to_string(&self.styles) {
            files.write(&self.style_path, text, fastframe_file::Access::Default);
        }
    }

    /// Forgets the cached key, after a key is saved or the engine changes.
    pub fn refresh_key(&mut self) {
        self.key_cache = None;
    }

    pub fn target(&mut self) -> Target {
        let engine = self.config.engine;
        let key = match &self.key_cache {
            Some((cached, key)) if *cached == engine => key.clone(),
            _ => {
                let key = keys::resolve_key(engine);
                self.key_cache = Some((engine, key.clone()));
                key
            }
        };
        Target {
            engine,
            endpoint: self.config.endpoint(),
            model: self.config.model(),
            key,
            workdir: self.workdir.clone(),
        }
    }

    /// Why Znga cannot call a model now, if it cannot.
    pub fn blocked(&self) -> Option<&'static str> {
        if !self.config.enabled {
            Some("Turn Znga on in Setup first.")
        } else if self.config.is_cloud() && !self.config.cloud_consent {
            Some("Confirm in Setup that this engine may receive your messages.")
        } else {
            None
        }
    }

    /// Queues a model call, or says why it cannot run.
    pub fn submit(&mut self, prompt: Prompt, purpose: Purpose) -> Result<(), String> {
        if let Some(reason) = self.blocked() {
            return Err(reason.into());
        }
        let lane = runner::lane_for(purpose.background());
        if lane == runner::Lane::Background && self.runner.busy_on(lane) >= runner::BUSY {
            return Err("Znga is busy; skipped an automatic request.".into());
        }
        let target = self.target();
        self.runner.submit(target, prompt, purpose, lane);
        Ok(())
    }

    pub fn waiting_for(&self, matches: impl Fn(&Purpose) -> bool) -> bool {
        self.runner.is_waiting(matches)
    }

    pub fn push_alert(&mut self, alert: Alert) {
        self.alerts.push(alert);
        if self.alerts.len() > MAX_ALERTS {
            self.alerts.remove(0);
        }
        if !self.open {
            self.unseen += 1;
        }
    }

    pub fn style_for(&self, account: &str) -> StyleProfile {
        self.styles.get(account).cloned().unwrap_or_default()
    }

    pub fn set_status(&mut self, text: impl Into<String>, error: bool) {
        self.status = Some(Status {
            text: text.into(),
            error,
        });
    }

    /// Applies a model answer and says what the app must do about it.
    pub fn handle(
        &mut self,
        purpose: Purpose,
        answer: Result<String, String>,
        now: i64,
    ) -> Vec<Effect> {
        let mut effects = Vec::new();
        match (purpose, answer) {
            (Purpose::Briefing { proactive }, Ok(text)) => {
                self.briefings.push(Briefing {
                    text: text.clone(),
                    at: now,
                    proactive,
                });
                if self.briefings.len() > MAX_BRIEFINGS {
                    self.briefings.remove(0);
                }
                self.status = None;
                if proactive {
                    if !self.open {
                        self.unseen += 1;
                    }
                    effects.push(Effect::Notify {
                        title: "Znga · while you were away".into(),
                        body: notification_body(&text),
                        account: String::new(),
                        chat: String::new(),
                        message: None,
                        only_when_away: true,
                    });
                    if self.self_notes.take(&self.config, now) {
                        effects.push(Effect::SelfNote(format!("✨ Znga briefing\n\n{text}")));
                    }
                }
            }
            (
                Purpose::Draft {
                    kind,
                    account,
                    chat,
                    chat_title,
                },
                Ok(text),
            ) => {
                let text = clean_draft(&text);
                if text.is_empty() {
                    self.set_status("Znga had nothing to suggest.", false);
                    return effects;
                }
                self.drafts.add(Draft {
                    id: 0,
                    kind,
                    account: account.clone(),
                    chat: chat.clone(),
                    chat_title: chat_title.clone(),
                    text: text.clone(),
                    created: now,
                });
                if !self.open {
                    self.unseen += 1;
                }
                if matches!(kind, Kind::AutoReply | Kind::CheckIn) {
                    let title = match kind {
                        Kind::CheckIn => format!("Znga · check in with {chat_title}?"),
                        _ => format!("Znga drafted a reply to {chat_title}"),
                    };
                    effects.push(Effect::Notify {
                        title,
                        body: text,
                        account,
                        chat,
                        message: None,
                        only_when_away: true,
                    });
                } else {
                    self.tab = Tab::Drafts;
                    self.status = None;
                }
            }
            (Purpose::FanOut { account, chats }, Ok(text)) => match drafts::parse_fan_out(&text) {
                Ok(items) => {
                    let mut missing = Vec::new();
                    let mut made = 0;
                    for item in items {
                        match drafts::match_chat(&item.to, &chats) {
                            Some(chat) => {
                                let title = chats
                                    .iter()
                                    .find(|(id, _)| id == chat)
                                    .map(|(_, title)| title.clone())
                                    .unwrap_or_default();
                                self.drafts.add(Draft {
                                    id: 0,
                                    kind: Kind::Update,
                                    account: account.clone(),
                                    chat: chat.to_owned(),
                                    chat_title: title,
                                    text: clean_draft(&item.text),
                                    created: now,
                                });
                                made += 1;
                            }
                            None => missing.push(item.to),
                        }
                    }
                    self.tab = Tab::Drafts;
                    if missing.is_empty() {
                        self.set_status(format!("{made} drafts ready to review."), false);
                    } else {
                        self.set_status(
                            format!(
                                "{made} drafts ready. No unique chat found for: {}.",
                                missing.join(", ")
                            ),
                            true,
                        );
                    }
                }
                Err(error) => self.set_status(error, true),
            },
            (Purpose::Ask, Ok(text)) => {
                self.ask.push(Turn::assistant(text));
                self.ask_dirty = true;
                self.status = None;
            }
            (Purpose::Ask, Err(error)) => {
                self.ask.push(Turn::assistant(format!("⚠ {error}")));
                self.ask_dirty = true;
            }
            (Purpose::Plan { ask }, Err(error)) if error == runner::STOPPED => {
                return self.handle(Purpose::Answer { ask }, Err(error), now);
            }
            (Purpose::Plan { ask }, answer) => {
                let terms = answer
                    .map(|text| archive_ask::parse_plan(&text))
                    .unwrap_or_default();
                effects.push(Effect::Search { ask, terms });
            }
            (Purpose::Answer { ask }, answer) => {
                let Some(asking) = self.asking.remove(&ask) else {
                    return effects;
                };
                match asking.origin {
                    archive_ask::AskOrigin::Panel => {
                        let text = match answer {
                            Ok(text) => text,
                            Err(error) => format!("⚠ {error}"),
                        };
                        self.ask.push(Turn::assistant(text));
                        self.ask_dirty = true;
                        self.status = None;
                    }
                    archive_ask::AskOrigin::OwnChat { account, chat } => {
                        let text = match answer {
                            Ok(text) => text,
                            Err(error) => format!("⚠ {error}"),
                        };
                        effects.push(Effect::SendOwn {
                            account,
                            chat,
                            text: selfchat::marked(&text),
                        });
                    }
                }
            }
            (
                Purpose::Promises {
                    account,
                    chat,
                    chat_title,
                },
                Ok(text),
            ) => {
                let zone = jiff::tz::TimeZone::system();
                let found = promises::parse(&text, &zone);
                self.promises.add(&account, &chat, &chat_title, found, now);
            }
            (
                Purpose::Facts {
                    account,
                    chat,
                    chat_title,
                },
                Ok(text),
            ) => {
                let facts = memory::parse_facts(&text);
                self.people.add(&account, &chat, &chat_title, &facts, now);
            }
            (Purpose::Promises { .. } | Purpose::Facts { .. }, Err(_)) => {
                log::debug!("znga: an extraction failed");
            }
            (
                Purpose::CatchUp {
                    account,
                    chat,
                    chat_title,
                },
                Ok(text),
            ) => {
                let (summary, reply) = prompt::split_catch_up(&text);
                self.catch_ups
                    .retain(|c| !(c.account == account && c.chat == chat));
                self.catch_ups.push(CatchUp {
                    account: account.clone(),
                    chat: chat.clone(),
                    chat_title: chat_title.clone(),
                    summary,
                    at: now,
                });
                if self.catch_ups.len() > 10 {
                    self.catch_ups.remove(0);
                }
                if let Some(reply) = reply {
                    self.drafts.add(Draft {
                        id: 0,
                        kind: Kind::Suggestion,
                        account,
                        chat,
                        chat_title,
                        text: clean_draft(&reply),
                        created: now,
                    });
                }
                self.tab = Tab::Today;
                self.status = None;
            }
            (Purpose::Events { chat_title, .. }, Ok(text)) => {
                let zone = jiff::tz::TimeZone::system();
                let found = calendar::parse_events(&text, &zone);
                if found.is_empty() {
                    self.set_status(
                        format!("No plans with a date found in {chat_title}."),
                        false,
                    );
                } else {
                    self.set_status(
                        format!("{} plan(s) found in {chat_title}.", found.len()),
                        false,
                    );
                }
                self.events.retain(|e| e.chat_title != chat_title);
                self.events
                    .extend(found.into_iter().map(|event| FoundEvent {
                        chat_title: chat_title.clone(),
                        event,
                    }));
                self.tab = Tab::Today;
            }
            (Purpose::Voice { note }, answer) => {
                if let Some(entry) = self.voice_notes.iter_mut().find(|n| n.id == note) {
                    let transcript = match &entry.state {
                        VoiceState::Done { transcript, .. } => transcript.clone(),
                        _ => String::new(),
                    };
                    match answer {
                        Ok(text) => {
                            let (summary, reply) = prompt::split_catch_up(&text);
                            entry.state = VoiceState::Done {
                                transcript,
                                summary,
                            };
                            if let Some(reply) = reply {
                                let draft = Draft {
                                    id: 0,
                                    kind: Kind::Suggestion,
                                    account: entry.account.clone(),
                                    chat: entry.chat.clone(),
                                    chat_title: entry.chat_title.clone(),
                                    text: clean_draft(&reply),
                                    created: now,
                                };
                                self.drafts.add(draft);
                            }
                        }
                        Err(error) => entry.state = VoiceState::Failed(error),
                    }
                }
            }
            (Purpose::ExplainEdit { key }, answer) => {
                self.explanations.insert(key, answer);
            }
            (Purpose::Test, Ok(_)) => self.set_status("Connected. Znga is ready.", false),
            (Purpose::Briefing { proactive: true }, Err(error))
            | (
                Purpose::Draft {
                    kind: Kind::AutoReply | Kind::CheckIn,
                    ..
                },
                Err(error),
            ) => {
                // Automatic requests fail quietly: a status line, no popup.
                log::debug!("znga: an automatic request failed");
                self.set_status(error, true);
            }
            (_, Err(error)) => self.set_status(error, true),
        }
        effects
    }

    /// Records one of our own sent messages as a style example.
    pub fn learn(&mut self, account: &str, chat: &str, text: &str) {
        self.examples.add(&chat_key(account, chat), text);
    }

    /// Recomputes each account's style statistics from the examples.
    /// Returns whether any changed.
    pub fn remeasure(&mut self, accounts: &[String]) -> bool {
        let mut changed = false;
        for account in accounts {
            let own = self.examples.texts_for_account(account);
            if own.len() < 3 {
                continue;
            }
            let profile = StyleProfile::measure(own);
            if self.styles.get(account) != Some(&profile) {
                self.styles.insert(account.clone(), profile);
                changed = true;
            }
        }
        changed
    }
}

/// The first useful lines of a briefing, for a notification.
fn notification_body(text: &str) -> String {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let start = lines
        .iter()
        .position(|line| line.to_uppercase().starts_with("NEEDS YOU"))
        .map(|i| i + 1)
        .filter(|i| *i < lines.len())
        .unwrap_or(0);
    let body = lines[start..]
        .iter()
        .take(3)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    if body.chars().count() > 240 {
        body.chars().take(240).collect::<String>() + "…"
    } else {
        body
    }
}

/// A model's draft without wrapping quotes or a "Draft:" label.
pub fn clean_draft(text: &str) -> String {
    let mut text = text.trim();
    for label in ["Draft:", "Reply:", "Message:"] {
        if let Some(rest) = text.strip_prefix(label) {
            text = rest.trim();
        }
    }
    for (open, close) in [('"', '"'), ('“', '”'), ('\'', '\'')] {
        if text.len() > 1 && text.starts_with(open) && text.ends_with(close) {
            text = text[open.len_utf8()..text.len() - close.len_utf8()].trim();
        }
    }
    text.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn znga() -> (tempfile::TempDir, Znga) {
        let dir = tempfile::tempdir().unwrap();
        let mut znga = Znga::load(dir.path(), dir.path(), crate::backend::Waker::default());
        znga.config.enabled = true;
        (dir, znga)
    }

    fn draft_purpose(kind: Kind) -> Purpose {
        Purpose::Draft {
            kind,
            account: "1".into(),
            chat: "mom".into(),
            chat_title: "Mom".into(),
        }
    }

    #[test]
    fn nothing_runs_until_znga_is_on_and_cloud_is_consented() {
        let (_dir, mut znga) = znga();
        znga.config.enabled = false;
        assert!(
            znga.submit(
                prompt::rewrite("hi", prompt::Rewrite::Shorter),
                Purpose::Test
            )
            .is_err()
        );
        znga.config.enabled = true;
        znga.config.engine = config::Engine::ClaudeCode;
        assert!(
            znga.blocked().is_some(),
            "Claude Code sends text to Anthropic"
        );
        znga.config.cloud_consent = true;
        assert!(znga.blocked().is_none());
    }

    #[test]
    fn answers_become_drafts_and_never_send() {
        let (_dir, mut znga) = znga();
        znga.runner.fake = Some(Box::new(|_| Ok("\"sure, see you sunday\"".into())));
        znga.submit(
            prompt::rewrite("x", prompt::Rewrite::Shorter),
            draft_purpose(Kind::AutoReply),
        )
        .unwrap();
        let answers = znga.runner.poll();
        assert_eq!(answers.len(), 1);
        for (purpose, answer) in answers {
            let effects = znga.handle(purpose, answer, 100);
            assert!(effects.iter().all(|e| !matches!(e, Effect::SelfNote(_))));
            assert!(matches!(
                effects[0],
                Effect::Notify {
                    only_when_away: true,
                    ..
                }
            ));
        }
        assert_eq!(znga.drafts.len(), 1);
        assert_eq!(znga.drafts.items()[0].text, "sure, see you sunday");
        assert_eq!(znga.unseen, 1);
    }

    #[test]
    fn proactive_briefings_notify_and_copy_to_self_only_when_enabled() {
        let (_dir, mut znga) = znga();
        let text = "WHAT HAPPENED\nAda asked about Sunday\nNEEDS YOU\n1. Reply to Ada".to_owned();
        let effects = znga.handle(Purpose::Briefing { proactive: true }, Ok(text.clone()), 0);
        assert_eq!(effects.len(), 1);
        match &effects[0] {
            Effect::Notify { body, .. } => assert_eq!(body, "1. Reply to Ada"),
            other => panic!("{other:?}"),
        }
        znga.config.self_notes = true;
        let effects = znga.handle(Purpose::Briefing { proactive: true }, Ok(text.clone()), 0);
        assert!(matches!(&effects[1], Effect::SelfNote(note) if note.contains("Reply to Ada")));
        let manual = znga.handle(Purpose::Briefing { proactive: false }, Ok(text), 0);
        assert!(manual.is_empty());
        assert_eq!(znga.briefings.len(), 3);
    }

    #[test]
    fn fan_out_drafts_only_matched_people() {
        let (_dir, mut znga) = znga();
        let chats = vec![
            ("mom".to_owned(), "Mom".to_owned()),
            ("alex".to_owned(), "Alex".to_owned()),
        ];
        let answer = r#"[{"to":"Mom","text":"running late, 20 min"},{"to":"Zed","text":"hi"}]"#;
        znga.handle(
            Purpose::FanOut {
                account: "1".into(),
                chats,
            },
            Ok(answer.into()),
            0,
        );
        assert_eq!(znga.drafts.len(), 1);
        assert_eq!(znga.drafts.items()[0].chat, "mom");
        let status = znga.status.clone().unwrap();
        assert!(status.error && status.text.contains("Zed"));
    }

    #[test]
    fn automatic_failures_are_quiet_and_ask_failures_show_in_the_thread() {
        let (_dir, mut znga) = znga();
        let effects = znga.handle(draft_purpose(Kind::AutoReply), Err("offline".into()), 0);
        assert!(effects.is_empty());
        assert!(znga.status.as_ref().unwrap().error);
        znga.handle(Purpose::Ask, Err("offline".into()), 0);
        assert!(znga.ask.last().unwrap().text.contains("offline"));
    }

    #[test]
    fn styles_are_measured_per_account() {
        let (_dir, mut znga) = znga();
        for text in ["haha ya", "ok cool", "omw lol", "sure"] {
            znga.learn("1", "mom", text);
        }
        znga.learn("2", "boss", "Thank you.");
        assert!(znga.remeasure(&["1".into(), "2".into()]));
        assert_eq!(znga.styles["1"].messages, 4);
        assert!(!znga.styles.contains_key("2"), "too few to measure");
        assert!(!znga.remeasure(&["1".into()]), "unchanged");
    }

    #[test]
    fn drafts_lose_wrapping_quotes_and_labels() {
        assert_eq!(clean_draft("Draft: \"hey!\""), "hey!");
        assert_eq!(clean_draft("“see you”"), "see you");
        assert_eq!(clean_draft("it's fine"), "it's fine");
    }
}
