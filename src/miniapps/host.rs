//! The app host (REQ-MINIAPP-002..008): installs, event routing, the
//! Sentinel and caps on every action, and isolation of failing apps.
//!
//! Pure state with time and the app's world passed in. What must happen in
//! the app (sending, asking Znga, disk) comes out as [`Out`]s that
//! [`super::hooks`] carries out.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};

use serde::{Deserialize, Serialize};

use super::{Activity, AppAction, Card, ChatKind, Control, Ctx, Event, From, MiniApp, Permission};
use crate::znga::sentinel::{self, Action, AppAct, Entry, Origin};

/// Messages older than this when they arrive are not delivered
/// (REQ-MINIAPP-003.A4).
pub const MAX_AGE: i64 = 5 * 60;
/// Messages one install may send a day (REQ-MINIAPP-004.A4).
pub const SENDS_PER_DAY: u32 = 20;
/// Seconds between two messages of one install.
pub const SEND_GAP: i64 = 10;
/// Bytes one install may store (REQ-MINIAPP-006.A4).
pub const STORE_CAP: usize = 1024 * 1024;
/// Events one delivery may cause before the app is stopped as looping.
const CHAIN: usize = 16;

/// One app in one chat on one account (REQ-MINIAPP-002).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Install {
    pub account: String,
    pub chat: String,
    pub app: String,
    pub kind: ChatKind,
    /// When the app asked to be woken.
    #[serde(default)]
    pub wake: Option<i64>,
    /// Why the app was turned off here (REQ-MINIAPP-007.A1).
    #[serde(default)]
    pub off: Option<String>,
}

impl Install {
    pub fn key(&self) -> String {
        key(&self.account, &self.chat, &self.app)
    }
}

pub fn key(account: &str, chat: &str, app: &str) -> String {
    format!("{account}|{chat}|{app}")
}

/// What is saved in `miniapps.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Saved {
    /// The one switch for all mini-apps (REQ-MINIAPP-002.A5).
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub installs: Vec<Install>,
}

fn yes() -> bool {
    true
}

impl Default for Saved {
    fn default() -> Self {
        Self {
            enabled: true,
            installs: Vec::new(),
        }
    }
}

/// Something the app must do for the host.
#[derive(Clone, Debug, PartialEq)]
pub enum Out {
    Send {
        account: String,
        chat: String,
        text: String,
    },
    /// Ask Znga on its background lane; answer with [`Host::answered`].
    Znga {
        key: String,
        tag: String,
        system: String,
        user: String,
    },
    /// Load an install's values; answer with [`Host::loaded`].
    Load { key: String, account: String },
    Put {
        key: String,
        account: String,
        name: String,
        value: Option<String>,
    },
    /// Delete every value of an install.
    Drop { key: String, account: String },
}

/// What the host is told about the app around it.
pub struct World<'a> {
    pub now: i64,
    pub zone: &'a jiff::tz::TimeZone,
    pub znga_on: bool,
    /// Whether (account, chat) may run apps: not locked, not excluded.
    pub usable: &'a dyn Fn(&str, &str) -> bool,
    /// Znga's activity for an account, from visible chats only.
    pub activity: &'a dyn Fn(&str) -> Activity,
}

/// A message as the host sees it.
pub struct Incoming<'a> {
    pub id: &'a str,
    pub text: &'a str,
    pub from: From,
    pub sent_at: i64,
    /// The text of the message it quotes, if any.
    pub quoted: Option<&'a str>,
}

#[derive(Default)]
struct Live {
    cards: Vec<Card>,
    /// `None` until loaded from the store.
    data: Option<BTreeMap<String, String>>,
    loading: bool,
    /// Events that arrived before the data loaded.
    pending: Vec<(Event, Origin)>,
    day: i64,
    sent_today: u32,
    last_send: Option<i64>,
}

pub struct Host {
    pub saved: Saved,
    apps: Vec<Box<dyn MiniApp>>,
    live: HashMap<String, Live>,
    /// Message keys already delivered (REQ-MINIAPP-003.A4).
    seen: HashSet<String>,
    seen_order: VecDeque<String>,
    /// Origins of Znga requests in flight, by (install, tag).
    asks: HashMap<(String, String), Origin>,
    /// When this run of ZapFast started: earlier wake-ups are late.
    started: i64,
    out: Vec<Out>,
    audit: Vec<Entry>,
    pub dirty: bool,
}

impl Host {
    pub fn new(saved: Saved, now: i64) -> Self {
        Self::with_apps(saved, now, super::registry())
    }

    pub fn with_apps(saved: Saved, now: i64, apps: Vec<Box<dyn MiniApp>>) -> Self {
        Self {
            saved,
            apps,
            live: HashMap::new(),
            seen: HashSet::new(),
            seen_order: VecDeque::new(),
            asks: HashMap::new(),
            started: now,
            out: Vec::new(),
            audit: Vec::new(),
            dirty: false,
        }
    }

    pub fn apps(&self) -> impl Iterator<Item = &'static super::Manifest> + '_ {
        self.apps.iter().map(|app| app.manifest())
    }

    fn app(&self, id: &str) -> Option<&dyn MiniApp> {
        self.apps
            .iter()
            .find(|app| app.manifest().id == id)
            .map(|app| app.as_ref())
    }

    pub fn take_out(&mut self) -> Vec<Out> {
        std::mem::take(&mut self.out)
    }

    pub fn take_audit(&mut self) -> Vec<Entry> {
        std::mem::take(&mut self.audit)
    }

    pub fn installs_on<'a>(&'a self, account: &'a str) -> impl Iterator<Item = &'a Install> + 'a {
        self.saved
            .installs
            .iter()
            .filter(move |install| install.account == account)
    }

    pub fn installed(&self, account: &str, chat: &str, app: &str) -> bool {
        self.saved
            .installs
            .iter()
            .any(|i| i.account == account && i.chat == chat && i.app == app)
    }

    pub fn cards(&self, key: &str) -> &[Card] {
        self.live.get(key).map_or(&[], |live| &live.cards)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.saved.enabled = enabled;
        self.dirty = true;
    }

    /// Installs `app` in a chat (REQ-MINIAPP-002). The install screen has
    /// already shown its permissions.
    pub fn install(
        &mut self,
        world: &World,
        account: &str,
        chat: &str,
        kind: ChatKind,
        app: &str,
    ) -> Result<(), String> {
        let Some(found) = self.app(app) else {
            return Err("There is no such mini-app.".into());
        };
        let manifest = found.manifest();
        if !manifest.kinds.contains(&kind) {
            return Err(format!(
                "{} can be added only to {}.",
                manifest.name,
                manifest
                    .kinds
                    .iter()
                    .map(|kind| kind.label())
                    .collect::<Vec<_>>()
                    .join(" or ")
            ));
        }
        if !(world.usable)(account, chat) {
            return Err("Mini-apps cannot be added to locked or hidden chats.".into());
        }
        if self.installed(account, chat, app) {
            return Err(format!("{} is already in this chat.", manifest.name));
        }
        let install = Install {
            account: account.into(),
            chat: chat.into(),
            app: app.into(),
            kind,
            wake: None,
            off: None,
        };
        let key = install.key();
        self.saved.installs.push(install);
        self.dirty = true;
        // Anything left from an earlier install is gone.
        self.out.push(Out::Drop {
            key: key.clone(),
            account: account.into(),
        });
        self.live.insert(
            key.clone(),
            Live {
                data: Some(BTreeMap::new()),
                ..Live::default()
            },
        );
        self.run(world, &key, Event::Installed, Origin::Person);
        Ok(())
    }

    /// Stops the app in that chat at once and deletes its data
    /// (REQ-MINIAPP-002.A4).
    pub fn uninstall(&mut self, key: &str) {
        let Some(index) = self.saved.installs.iter().position(|i| i.key() == key) else {
            return;
        };
        let install = self.saved.installs.remove(index);
        self.live.remove(key);
        self.asks.retain(|(k, _), _| k != key);
        self.dirty = true;
        self.out.push(Out::Drop {
            key: key.into(),
            account: install.account,
        });
    }

    /// Turns a failed install back on; it is woken at once so it can
    /// set its schedule again.
    pub fn restart(&mut self, key: &str, now: i64) {
        if let Some(install) = self.saved.installs.iter_mut().find(|i| i.key() == key) {
            install.off = None;
            install.wake = Some(now);
            self.dirty = true;
        }
        if let Some(live) = self.live.get_mut(key) {
            live.cards.clear();
        }
    }

    fn mark_seen(&mut self, key: String) -> bool {
        if !self.seen.insert(key.clone()) {
            return false;
        }
        self.seen_order.push_back(key);
        if self.seen_order.len() > 2000
            && let Some(old) = self.seen_order.pop_front()
        {
            self.seen.remove(&old);
        }
        true
    }

    /// Delivers commands and quote-replies addressed to an app installed
    /// in this chat; nothing else (REQ-MINIAPP-003).
    pub fn observe(
        &mut self,
        world: &World,
        account: &str,
        chat: &str,
        own_chat: bool,
        messages: &[Incoming],
        older: bool,
    ) {
        if !self.saved.enabled || older || !(world.usable)(account, chat) {
            return;
        }
        let installs: Vec<Install> = self
            .saved
            .installs
            .iter()
            .filter(|i| i.account == account && i.chat == chat && i.off.is_none())
            .cloned()
            .collect();
        if installs.is_empty() {
            return;
        }
        for message in messages {
            if world.now - message.sent_at > MAX_AGE {
                continue;
            }
            for install in &installs {
                let Some(manifest) = self.app(&install.app).map(|app| app.manifest()) else {
                    continue;
                };
                let event = if let Some(text) = manifest.command_in(message.text) {
                    Event::Command {
                        text: text.into(),
                        from: message.from.clone(),
                    }
                } else if message
                    .quoted
                    .is_some_and(|quoted| quoted.trim_start().starts_with(&manifest.marker()))
                {
                    Event::Reply {
                        text: message.text.trim().into(),
                        from: message.from.clone(),
                    }
                } else {
                    continue;
                };
                let origin = match message.from {
                    From::Me if own_chat => Origin::OwnChat,
                    From::Me => Origin::Person,
                    From::Member { .. } => Origin::OthersText,
                };
                if origin == Origin::OthersText && !manifest.allows(Permission::AnswerMembers) {
                    continue;
                }
                if !self.mark_seen(format!("{}|{}", install.key(), message.id)) {
                    continue;
                }
                self.run(world, &install.key(), event, origin);
            }
        }
    }

    /// Whether a text in a chat is a mini-app's rather than a question for
    /// Znga (REQ-MINIAPP-008).
    pub fn claims(&self, account: &str, chat: &str, text: &str, quoted: Option<&str>) -> bool {
        let is_app_message = |text: &str| {
            self.apps
                .iter()
                .any(|app| text.trim_start().starts_with(&app.manifest().marker()))
        };
        if is_app_message(text) {
            return true;
        }
        if !self.saved.enabled {
            return false;
        }
        self.saved
            .installs
            .iter()
            .filter(|i| i.account == account && i.chat == chat && i.off.is_none())
            .filter_map(|i| self.app(&i.app))
            .any(|app| {
                let manifest = app.manifest();
                manifest.command_in(text).is_some()
                    || quoted.is_some_and(|q| q.trim_start().starts_with(&manifest.marker()))
            })
    }

    /// Loads data for installs that need it and delivers due wake-ups.
    pub fn tick(&mut self, world: &World) {
        if !self.saved.enabled {
            return;
        }
        let installs: Vec<Install> = self.saved.installs.clone();
        for install in installs {
            if install.off.is_some() || !(world.usable)(&install.account, &install.chat) {
                continue;
            }
            let key = install.key();
            self.ensure_loading(&key, &install.account);
            if let Some(at) = install.wake
                && at <= world.now
            {
                if let Some(i) = self.saved.installs.iter_mut().find(|i| i.key() == key) {
                    i.wake = None;
                    self.dirty = true;
                }
                let late = at < self.started;
                self.run(world, &key, Event::Wake { at, late }, Origin::Schedule);
            }
        }
    }

    fn ensure_loading(&mut self, key: &str, account: &str) {
        let live = self.live.entry(key.to_owned()).or_default();
        if live.data.is_none() && !live.loading {
            live.loading = true;
            self.out.push(Out::Load {
                key: key.into(),
                account: account.into(),
            });
        }
    }

    /// An install's values arrived from the store.
    pub fn loaded(
        &mut self,
        world: &World,
        key: &str,
        data: Result<BTreeMap<String, String>, String>,
    ) {
        let Some(live) = self.live.get_mut(key) else {
            return;
        };
        live.loading = false;
        match data {
            Ok(data) => {
                live.data = Some(data);
                let pending = std::mem::take(&mut live.pending);
                for (event, origin) in pending {
                    self.run(world, key, event, origin);
                }
            }
            Err(error) => self.fail(key, format!("Its data could not be read: {error}")),
        }
    }

    /// A click on a card (REQ-MINIAPP-005.A2).
    pub fn click(
        &mut self,
        world: &World,
        key: &str,
        card: &str,
        control: Control,
        fields: BTreeMap<String, String>,
    ) {
        let event = Event::Click {
            card: card.into(),
            control,
            fields,
        };
        self.run(world, key, event, Origin::Person);
    }

    /// Znga answered an app's request (REQ-MINIAPP-007.A2).
    pub fn answered(
        &mut self,
        world: &World,
        key: &str,
        tag: &str,
        answer: Result<String, String>,
    ) {
        let origin = self
            .asks
            .remove(&(key.to_owned(), tag.to_owned()))
            .unwrap_or(Origin::Schedule);
        let event = Event::Answer {
            tag: tag.into(),
            answer,
        };
        self.run(world, key, event, origin);
    }

    /// Hands `event` to its app and carries out what it asks, isolating
    /// any failure to this install (REQ-MINIAPP-007).
    pub fn run(&mut self, world: &World, key: &str, event: Event, origin: Origin) {
        if !self.saved.enabled {
            return;
        }
        let Some(install) = self.saved.installs.iter().find(|i| i.key() == key).cloned() else {
            return;
        };
        if install.off.is_some() || !(world.usable)(&install.account, &install.chat) {
            return;
        }
        self.ensure_loading(key, &install.account);
        let Some(live) = self.live.get_mut(key) else {
            return;
        };
        if live.data.is_none() {
            live.pending.push((event, origin));
            return;
        }
        let mut queue = VecDeque::from([(event, origin)]);
        let mut handled = 0;
        while let Some((event, origin)) = queue.pop_front() {
            handled += 1;
            if handled > CHAIN {
                self.fail(key, "It kept asking for more without stopping.".into());
                return;
            }
            let Some(app) = self.app(&install.app) else {
                return;
            };
            let data = self
                .live
                .get(key)
                .and_then(|live| live.data.clone())
                .unwrap_or_default();
            let ctx = Ctx {
                now: world.now,
                zone: world.zone,
                znga_on: world.znga_on,
                data: &data,
            };
            let actions = match catch_unwind(AssertUnwindSafe(|| app.handle(&event, &ctx))) {
                Ok(actions) => actions,
                Err(_) => {
                    self.fail(key, "It stopped with an internal error.".into());
                    return;
                }
            };
            for action in actions {
                if let Some(next) = self.apply(world, &install, action, origin) {
                    queue.push_back((next, origin));
                }
                if self
                    .saved
                    .installs
                    .iter()
                    .find(|i| i.key() == key)
                    .is_none_or(|i| i.off.is_some())
                {
                    return;
                }
            }
        }
    }

    fn fail(&mut self, key: &str, error: String) {
        log::warn!(target: "zapfast::miniapps", "{key} turned off: {error}");
        if let Some(install) = self.saved.installs.iter_mut().find(|i| i.key() == key) {
            install.off = Some(error.clone());
            self.dirty = true;
        }
        if let Some(live) = self.live.get_mut(key) {
            live.cards = vec![Card {
                id: "error".into(),
                title: "Turned off".into(),
                lines: vec![error],
                ..Card::default()
            }];
        }
    }

    /// Checks one action with the Sentinel and logs the decision
    /// (REQ-MINIAPP-004.A5).
    #[allow(clippy::too_many_arguments)]
    fn sentinel(
        &mut self,
        now: i64,
        name: &str,
        act: AppAct,
        install: &Install,
        declared: bool,
        members: bool,
        origin: Origin,
    ) -> bool {
        let action = Action::App {
            app: name,
            act,
            chat: &install.chat,
            home: &install.chat,
            declared,
            members,
        };
        let verdict = sentinel::check(&action, origin);
        self.audit.push(Entry {
            at: now,
            action: action.label(),
            origin,
            allowed: verdict.allowed,
            reason: verdict.reason.into(),
        });
        verdict.allowed
    }

    fn refuse(&mut self, now: i64, name: &str, act: AppAct, origin: Origin, reason: &str) {
        self.audit.push(Entry {
            at: now,
            action: format!("{name}: {}", act.label()),
            origin,
            allowed: false,
            reason: reason.into(),
        });
    }

    /// Carries out one action (REQ-MINIAPP-004). Returns an event the
    /// action causes at once.
    pub fn apply(
        &mut self,
        world: &World,
        install: &Install,
        action: AppAction,
        origin: Origin,
    ) -> Option<Event> {
        let manifest = self.app(&install.app)?.manifest();
        let key = install.key();
        let members = manifest.allows(Permission::AnswerMembers);
        let now = world.now;
        match action {
            AppAction::Send { text } => {
                let declared = manifest.allows(Permission::Send);
                if !self.sentinel(
                    now,
                    manifest.name,
                    AppAct::Send,
                    install,
                    declared,
                    members,
                    origin,
                ) {
                    return Some(Event::Refused {
                        action: "send".into(),
                        reason: "not allowed".into(),
                    });
                }
                let day = now.div_euclid(86_400);
                let live = self.live.entry(key.clone()).or_default();
                if live.day != day {
                    live.day = day;
                    live.sent_today = 0;
                }
                let reason = if live.sent_today >= SENDS_PER_DAY {
                    Some("20 messages a day at most")
                } else if live.last_send.is_some_and(|last| now - last < SEND_GAP) {
                    Some("one message every 10 seconds at most")
                } else {
                    None
                };
                if let Some(reason) = reason {
                    self.refuse(now, manifest.name, AppAct::Send, origin, reason);
                    return Some(Event::Refused {
                        action: "send".into(),
                        reason: reason.into(),
                    });
                }
                live.sent_today += 1;
                live.last_send = Some(now);
                self.out.push(Out::Send {
                    account: install.account.clone(),
                    chat: install.chat.clone(),
                    text: format!("{}\n{}", manifest.marker(), text.trim()),
                });
                None
            }
            AppAction::Cards(cards) => {
                self.live.entry(key).or_default().cards = cards;
                None
            }
            AppAction::AskZnga { tag, system, user } => {
                let declared = manifest.allows(Permission::UseZnga);
                if !self.sentinel(
                    now,
                    manifest.name,
                    AppAct::UseZnga,
                    install,
                    declared,
                    members,
                    origin,
                ) {
                    return Some(Event::Answer {
                        tag,
                        answer: Err("Znga may not be asked for this.".into()),
                    });
                }
                if !world.znga_on {
                    return Some(Event::Answer {
                        tag,
                        answer: Err("Znga is off.".into()),
                    });
                }
                self.asks.insert((key.clone(), tag.clone()), origin);
                self.out.push(Out::Znga {
                    key,
                    tag,
                    system,
                    user,
                });
                None
            }
            AppAction::ReadActivity { tag } => {
                let declared = manifest.allows(Permission::ReadActivity);
                let activity = if self.sentinel(
                    now,
                    manifest.name,
                    AppAct::ReadActivity,
                    install,
                    declared,
                    members,
                    origin,
                ) {
                    (world.activity)(&install.account)
                } else {
                    Activity::default()
                };
                Some(Event::Activity { tag, activity })
            }
            AppAction::WakeAt(at) => {
                if let Some(i) = self.saved.installs.iter_mut().find(|i| i.key() == key) {
                    i.wake = Some(at);
                    self.dirty = true;
                }
                None
            }
            AppAction::Store { key: name, value } => {
                if !self.sentinel(
                    now,
                    manifest.name,
                    AppAct::Store,
                    install,
                    true,
                    members,
                    origin,
                ) {
                    return None;
                }
                let live = self.live.entry(key.clone()).or_default();
                let data = live.data.get_or_insert_with(BTreeMap::new);
                let size: usize = data
                    .iter()
                    .filter(|(k, _)| **k != name)
                    .map(|(k, v)| k.len() + v.len())
                    .sum::<usize>()
                    + value.as_ref().map_or(0, |v| name.len() + v.len());
                if size > STORE_CAP {
                    self.refuse(now, manifest.name, AppAct::Store, origin, "1 MB at most");
                    return Some(Event::StoreFull { key: name });
                }
                match &value {
                    Some(v) => data.insert(name.clone(), v.clone()),
                    None => data.remove(&name),
                };
                self.out.push(Out::Put {
                    key,
                    account: install.account.clone(),
                    name,
                    value,
                });
                None
            }
            AppAction::Fail(error) => {
                self.fail(&key, error);
                None
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::miniapps::Manifest;

    static ECHO: Manifest = Manifest {
        id: "echo",
        name: "Echo",
        icon: "🔁",
        description: "Says it back.",
        command: "echo",
        kinds: &[ChatKind::Own, ChatKind::Direct, ChatKind::Group],
        permissions: &[Permission::Send, Permission::UseZnga],
    };

    static GROUPY: Manifest = Manifest {
        id: "groupy",
        name: "Groupy",
        icon: "👥",
        description: "Answers everyone.",
        command: "groupy",
        kinds: &[ChatKind::Group],
        permissions: &[Permission::Send, Permission::AnswerMembers],
    };

    struct Echo(&'static Manifest);

    impl MiniApp for Echo {
        fn manifest(&self) -> &'static Manifest {
            self.0
        }

        fn handle(&self, event: &Event, _ctx: &Ctx) -> Vec<AppAction> {
            match event {
                Event::Command { text, from } => match text.as_str() {
                    "boom" => panic!("boom"),
                    "fail" => vec![AppAction::Fail("broken".into())],
                    "loop" => vec![AppAction::ReadActivity { tag: "x".into() }],
                    "ask" => vec![AppAction::AskZnga {
                        tag: "q".into(),
                        system: "s".into(),
                        user: "u".into(),
                    }],
                    "big" => vec![AppAction::Store {
                        key: "big".into(),
                        value: Some("x".repeat(STORE_CAP + 1)),
                    }],
                    "wake" => vec![AppAction::WakeAt(1_000)],
                    _ => {
                        let who = match from {
                            From::Me => "me".to_owned(),
                            From::Member { name, .. } => name.clone(),
                        };
                        vec![
                            AppAction::Send {
                                text: format!("{who}: {text}"),
                            },
                            AppAction::Store {
                                key: "last".into(),
                                value: Some(text.clone()),
                            },
                        ]
                    }
                },
                Event::Activity { .. } => vec![AppAction::ReadActivity { tag: "x".into() }],
                Event::Reply { text, .. } => vec![AppAction::Send {
                    text: format!("reply {text}"),
                }],
                Event::Answer { answer, .. } => vec![AppAction::Send {
                    text: match answer {
                        Ok(text) => text.clone(),
                        Err(error) => format!("error {error}"),
                    },
                }],
                Event::Wake { late, .. } => vec![AppAction::Send {
                    text: format!("woke late={late}"),
                }],
                Event::StoreFull { .. } => vec![AppAction::Cards(vec![Card {
                    id: "full".into(),
                    ..Card::default()
                }])],
                Event::Click { control, .. } => vec![AppAction::Cards(vec![Card {
                    id: format!("{control:?}"),
                    ..Card::default()
                }])],
                _ => Vec::new(),
            }
        }
    }

    pub fn host() -> Host {
        Host::with_apps(
            Saved::default(),
            0,
            vec![Box::new(Echo(&ECHO)), Box::new(Echo(&GROUPY))],
        )
    }

    pub fn with_world<R>(now: i64, znga_on: bool, f: impl FnOnce(&World) -> R) -> R {
        let zone = jiff::tz::TimeZone::UTC;
        let usable = |_: &str, chat: &str| chat != "locked";
        let activity = |_: &str| Activity::default();
        f(&World {
            now,
            zone: &zone,
            znga_on,
            usable: &usable,
            activity: &activity,
        })
    }

    fn me(id: &str, text: &str, at: i64) -> Incoming<'static> {
        Incoming {
            id: Box::leak(id.to_owned().into_boxed_str()),
            text: Box::leak(text.to_owned().into_boxed_str()),
            from: From::Me,
            sent_at: at,
            quoted: None,
        }
    }

    fn sends(host: &mut Host) -> Vec<String> {
        host.take_out()
            .into_iter()
            .filter_map(|out| match out {
                Out::Send { text, .. } => Some(text),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn nothing_is_installed_and_uninstalled_apps_hear_nothing() {
        let mut host = host();
        assert!(host.saved.installs.is_empty());
        with_world(100, true, |w| {
            host.observe(w, "a", "me", true, &[me("1", "/echo hi", 100)], false)
        });
        assert!(sends(&mut host).is_empty());
    }

    #[test]
    fn installs_respect_chat_kinds_and_locked_chats() {
        let mut host = host();
        with_world(0, true, |w| {
            assert!(
                host.install(w, "a", "g", ChatKind::Direct, "groupy")
                    .is_err()
            );
            assert!(
                host.install(w, "a", "locked", ChatKind::Own, "echo")
                    .is_err()
            );
            assert!(host.install(w, "a", "me", ChatKind::Own, "echo").is_ok());
            assert!(host.install(w, "a", "me", ChatKind::Own, "echo").is_err());
        });
        assert_eq!(host.saved.installs.len(), 1);
    }

    fn installed() -> Host {
        let mut host = host();
        with_world(0, true, |w| {
            host.install(w, "a", "me", ChatKind::Own, "echo")
        })
        .unwrap();
        host.take_out();
        host
    }

    #[test]
    fn commands_reach_the_app_once_and_other_text_does_not() {
        let mut host = installed();
        with_world(100, true, |w| {
            host.observe(
                w,
                "a",
                "me",
                true,
                &[
                    me("1", "/echo done", 100),
                    me("1", "/echo done", 100),
                    me("2", "done", 100),
                ],
                false,
            )
        });
        assert_eq!(sends(&mut host), vec!["🔁 Echo\nme: done"]);
    }

    #[test]
    fn old_replayed_and_history_messages_are_not_delivered() {
        let mut host = installed();
        with_world(1_000, true, |w| {
            host.observe(w, "a", "me", true, &[me("1", "/echo a", 600)], false);
            host.observe(w, "a", "me", true, &[me("2", "/echo b", 1_000)], true);
        });
        assert!(sends(&mut host).is_empty());
    }

    #[test]
    fn quote_replies_reach_the_app_only_for_its_own_messages() {
        let mut host = installed();
        with_world(100, true, |w| {
            let mut reply = me("1", "half way", 100);
            reply.quoted = Some("🔁 Echo\nhow is it going?");
            let mut other = me("2", "nope", 100);
            other.quoted = Some("✨ Znga\nhello");
            host.observe(w, "a", "me", true, &[reply, other], false);
        });
        assert_eq!(sends(&mut host), vec!["🔁 Echo\nreply half way"]);
    }

    #[test]
    fn other_people_reach_only_apps_that_answer_members() {
        let mut host = host();
        with_world(0, true, |w| {
            host.install(w, "a", "g", ChatKind::Group, "groupy")
                .unwrap();
            host.install(w, "a", "g", ChatKind::Group, "echo").unwrap();
        });
        host.take_out();
        let ravi = Incoming {
            id: "1",
            text: "/echo hi",
            from: From::Member {
                id: "r".into(),
                name: "Ravi".into(),
            },
            sent_at: 20,
            quoted: None,
        };
        let ravi2 = Incoming {
            text: "/groupy hi",
            id: "2",
            ..ravi
        };
        let ravi = Incoming {
            id: "1",
            text: "/echo hi",
            from: From::Member {
                id: "r".into(),
                name: "Ravi".into(),
            },
            sent_at: 20,
            quoted: None,
        };
        with_world(20, true, |w| {
            host.observe(w, "a", "g", false, &[ravi, ravi2], false)
        });
        assert_eq!(sends(&mut host), vec!["👥 Groupy\nRavi: hi"]);
    }

    #[test]
    fn sends_are_capped_per_day_and_every_ten_seconds() {
        let mut host = installed();
        let mut n = 0;
        for i in 0..30 {
            let at = 1_000 + i * 11;
            with_world(at, true, |w| {
                host.observe(
                    w,
                    "a",
                    "me",
                    true,
                    &[me(&i.to_string(), "/echo x", at)],
                    false,
                )
            });
            n += sends(&mut host).len();
        }
        assert_eq!(n, SENDS_PER_DAY as usize);
        let mut host = installed();
        with_world(100, true, |w| {
            host.observe(
                w,
                "a",
                "me",
                true,
                &[me("1", "/echo a", 100), me("2", "/echo b", 100)],
                false,
            )
        });
        assert_eq!(sends(&mut host).len(), 1);
        let refused: Vec<Entry> = host
            .take_audit()
            .into_iter()
            .filter(|e| !e.allowed)
            .collect();
        assert!(refused.iter().any(|e| e.action == "Echo: send"));
    }

    #[test]
    fn every_action_is_logged_with_the_app_name() {
        let mut host = installed();
        host.take_audit();
        with_world(100, true, |w| {
            host.observe(w, "a", "me", true, &[me("1", "/echo hi", 100)], false)
        });
        let audit = host.take_audit();
        assert!(audit.iter().any(|e| e.action == "Echo: send" && e.allowed));
        assert!(audit.iter().any(|e| e.action == "Echo: store data"));
        assert!(audit.iter().all(|e| e.origin == Origin::OwnChat));
    }

    #[test]
    fn an_undeclared_permission_is_refused() {
        let mut host = installed();
        with_world(100, true, |w| {
            host.observe(w, "a", "me", true, &[me("1", "/echo loop", 100)], false)
        });
        // ReadActivity is not in Echo's manifest: refused, logged, and the
        // app that keeps asking is stopped.
        assert!(
            host.take_audit()
                .iter()
                .any(|e| e.action == "Echo: read activity" && !e.allowed)
        );
        assert!(host.saved.installs[0].off.is_some());
    }

    #[test]
    fn a_failing_app_is_turned_off_alone() {
        let mut host = host();
        with_world(0, true, |w| {
            host.install(w, "a", "me", ChatKind::Own, "echo").unwrap();
            host.install(w, "a", "g", ChatKind::Group, "groupy")
                .unwrap();
        });
        host.take_out();
        with_world(100, true, |w| {
            host.observe(w, "a", "me", true, &[me("1", "/echo boom", 100)], false);
            host.observe(w, "a", "g", false, &[me("2", "/groupy hi", 100)], false);
            host.observe(w, "a", "me", true, &[me("3", "/echo hi", 100)], false);
        });
        assert!(host.saved.installs[0].off.is_some());
        assert_eq!(
            host.cards(&host.saved.installs[0].key())[0].title,
            "Turned off"
        );
        assert_eq!(sends(&mut host), vec!["👥 Groupy\nme: hi"]);
    }

    #[test]
    fn znga_errors_come_back_to_the_app() {
        let mut host = installed();
        with_world(100, false, |w| {
            host.observe(w, "a", "me", true, &[me("1", "/echo ask", 100)], false)
        });
        assert_eq!(sends(&mut host), vec!["🔁 Echo\nerror Znga is off."]);
        with_world(200, true, |w| {
            host.observe(w, "a", "me", true, &[me("2", "/echo ask", 200)], false)
        });
        let out = host.take_out();
        assert!(matches!(&out[0], Out::Znga { tag, .. } if tag == "q"));
        let key = host.saved.installs[0].key();
        with_world(300, true, |w| {
            host.answered(w, &key, "q", Err("timed out".into()))
        });
        assert_eq!(sends(&mut host), vec!["🔁 Echo\nerror timed out"]);
    }

    #[test]
    fn stores_are_capped_and_the_app_is_told() {
        let mut host = installed();
        with_world(100, true, |w| {
            host.observe(w, "a", "me", true, &[me("1", "/echo big", 100)], false)
        });
        let key = host.saved.installs[0].key();
        assert_eq!(host.cards(&key)[0].id, "full");
        assert!(!host.take_out().iter().any(|o| matches!(o, Out::Put { .. })));
    }

    #[test]
    fn uninstalling_stops_the_app_and_drops_its_data() {
        let mut host = installed();
        let key = host.saved.installs[0].key();
        host.uninstall(&key);
        assert_eq!(
            host.take_out(),
            vec![Out::Drop {
                key: key.clone(),
                account: "a".into()
            }]
        );
        with_world(100, true, |w| {
            host.observe(w, "a", "me", true, &[me("1", "/echo hi", 100)], false)
        });
        assert!(sends(&mut host).is_empty());
    }

    #[test]
    fn the_master_switch_stops_everything_and_keeps_installs() {
        let mut host = installed();
        host.set_enabled(false);
        with_world(100, true, |w| {
            host.observe(w, "a", "me", true, &[me("1", "/echo hi", 100)], false);
            host.tick(w);
        });
        assert!(host.take_out().is_empty());
        assert!(!host.claims("a", "me", "/echo hi", None));
        assert_eq!(host.saved.installs.len(), 1);
    }

    #[test]
    fn locked_chats_stop_their_installs() {
        let mut host = host();
        with_world(0, true, |w| {
            host.install(w, "a", "c", ChatKind::Direct, "echo")
        })
        .unwrap();
        host.take_out();
        let zone = jiff::tz::TimeZone::UTC;
        let usable = |_: &str, _: &str| false;
        let activity = |_: &str| Activity::default();
        let world = World {
            now: 100,
            zone: &zone,
            znga_on: true,
            usable: &usable,
            activity: &activity,
        };
        host.observe(&world, "a", "c", false, &[me("1", "/echo hi", 100)], false);
        assert!(sends(&mut host).is_empty());
    }

    #[test]
    fn wake_ups_missed_while_closed_are_delivered_once_marked_late() {
        let mut saved = Saved::default();
        saved.installs.push(Install {
            account: "a".into(),
            chat: "me".into(),
            app: "echo".into(),
            kind: ChatKind::Own,
            wake: Some(50),
            off: None,
        });
        let mut host = Host::with_apps(saved, 100, vec![Box::new(Echo(&ECHO))]);
        let key = host.saved.installs[0].key();
        with_world(100, true, |w| host.tick(w));
        assert!(matches!(&host.take_out()[0], Out::Load { .. }));
        with_world(100, true, |w| host.loaded(w, &key, Ok(BTreeMap::new())));
        assert_eq!(sends(&mut host), vec!["🔁 Echo\nwoke late=true"]);
        with_world(200, true, |w| host.tick(w));
        assert!(sends(&mut host).is_empty());
    }

    #[test]
    fn clicks_deliver_one_event_with_their_control() {
        let mut host = installed();
        let key = host.saved.installs[0].key();
        with_world(100, true, |w| {
            host.click(w, &key, "c", Control::Step(1, true), BTreeMap::new())
        });
        assert_eq!(host.cards(&key).len(), 1);
        assert_eq!(host.cards(&key)[0].id, "Step(1, true)");
    }

    #[test]
    fn app_commands_and_messages_are_claimed_from_znga() {
        let host = installed();
        assert!(host.claims("a", "me", "/echo done", None));
        assert!(host.claims("a", "me", "🔁 Echo\nhi", None));
        assert!(host.claims("a", "me", "ok", Some("🔁 Echo\nhow?")));
        assert!(!host.claims("a", "me", "what's up?", None));
        assert!(!host.claims("a", "me", "/groupy hi", None));
        assert!(!host.claims("a", "other", "/echo hi", None));
    }
}
