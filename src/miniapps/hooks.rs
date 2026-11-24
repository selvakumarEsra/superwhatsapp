//! Where mini-apps meet the rest of ZapFast. Upstream code calls these
//! from one-line `// miniapps hook`s; everything else stays in
//! `src/miniapps/`.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use super::host::{Host, Incoming, Out, Saved, World};
use super::store::{Job, Store};
use super::{Activity, ChatKind, Control, From};
use crate::app::App;
use crate::model::Action;
use crate::model::Content;
use crate::znga::Purpose;
use crate::znga::provider::{Effort, Prompt, Turn};

/// The mini-apps' state in the app.
pub struct MiniApps {
    pub host: Host,
    store: Store,
    path: PathBuf,
    pub open: bool,
    /// The install screen in progress (REQ-MINIAPP-002.A2).
    pub sheet: Option<Sheet>,
    /// Text typed in card fields, by (install, card, field).
    pub fields: HashMap<(String, String, String), String>,
    pub notice: Option<String>,
}

/// An install waiting for the person to confirm its permissions.
#[derive(Clone, Debug, PartialEq)]
pub struct Sheet {
    pub account: String,
    pub chat: String,
    pub chat_title: String,
    pub kind: ChatKind,
    pub app: String,
}

impl MiniApps {
    pub fn load(config_dir: &Path, waker: crate::backend::Waker) -> Self {
        let path = config_dir.join("miniapps.json");
        let saved: Saved = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Self {
            host: Host::new(saved, crate::util::now()),
            store: Store::new(waker),
            path,
            open: false,
            sheet: None,
            fields: HashMap::new(),
            notice: None,
        }
    }
}

/// Whether apps may run in a chat: it exists, is not locked, not excluded
/// from Znga and not a broadcast list (REQ-MINIAPP-002.A3).
pub fn usable(app: &App, account: &str, chat: &str) -> bool {
    if chat.ends_with("@broadcast") || chat.ends_with("@newsletter") {
        return false;
    }
    if app.znga.config.is_excluded(account, chat) {
        return false;
    }
    let Some(known) = app
        .accounts
        .iter()
        .find(|known| known.id.as_str() == account)
    else {
        return false;
    };
    !known
        .chats
        .iter()
        .any(|c| c.id == chat && (c.locked || c.kind == crate::model::ChatKind::Broadcast))
}

/// Znga's activity for an account, visible chats only (REQ-GOAL-001.A2).
fn activity(app: &App, account: &str, zone: &jiff::tz::TimeZone) -> Activity {
    let now = crate::util::now();
    let today = jiff::Timestamp::from_second(now)
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH)
        .to_zoned(zone.clone())
        .date();
    let day_start = |date: jiff::civil::Date| {
        date.to_zoned(zone.clone())
            .map(|zoned| zoned.timestamp().as_second())
            .unwrap_or(now)
    };
    let start = day_start(today);
    let after_tomorrow = today
        .checked_add(jiff::Span::new().days(2))
        .map(day_start)
        .unwrap_or(now + 2 * 86_400);
    let tomorrow = today
        .checked_add(jiff::Span::new().days(1))
        .map(day_start)
        .unwrap_or(now + 86_400);
    let promises = app
        .znga
        .promises
        .open()
        .into_iter()
        .map(|(_, promise)| promise)
        .filter(|p| p.account == account && usable(app, account, &p.chat))
        .filter(|p| p.due.is_some_and(|due| due < after_tomorrow))
        .map(|p| match p.owes {
            crate::znga::promises::Owes::Me => format!("You promised {}: {}", p.chat_title, p.text),
            crate::znga::promises::Owes::Them => {
                format!("{} asked you: {}", p.chat_title, p.text)
            }
        })
        .collect();
    let plans = app
        .znga
        .events
        .iter()
        .filter(|found| (start..tomorrow).contains(&found.event.start))
        .map(|found| format!("{} ({})", found.event.title, found.chat_title))
        .collect();
    let recent = app
        .znga
        .activity
        .recent(30)
        .into_iter()
        .filter(|seen| seen.account == account && usable(app, account, &seen.chat))
        .map(|seen| format!("{}: {}: {}", seen.chat_title, seen.sender_name, seen.text))
        .collect();
    Activity {
        promises,
        plans,
        recent,
    }
}

/// Runs `f` with the host and the world around it, then carries out what
/// the host asked for.
fn with_host<R>(app: &mut App, f: impl FnOnce(&mut Host, &World) -> R) -> R {
    let mut host = std::mem::replace(
        &mut app.miniapps.host,
        Host::with_apps(Saved::default(), 0, Vec::new()),
    );
    let zone = jiff::tz::TimeZone::system();
    let result = {
        let shared: &App = app;
        let usable = |account: &str, chat: &str| usable(shared, account, chat);
        let activity = |account: &str| activity(shared, account, &zone);
        let world = World {
            now: crate::util::now(),
            zone: &zone,
            znga_on: shared.znga.config.enabled && shared.znga.blocked().is_none(),
            usable: &usable,
            activity: &activity,
        };
        f(&mut host, &world)
    };
    app.miniapps.host = host;
    carry_out(app);
    result
}

fn archive_of(app: &App, account: &str) -> Option<PathBuf> {
    app.accounts
        .iter()
        .find(|known| known.id.as_str() == account)
        .map(|known| known.dirs.archive_db())
}

/// Does what the host asked: sends, Znga requests, disk, the audit log.
fn carry_out(app: &mut App) {
    for entry in app.miniapps.host.take_audit() {
        app.znga.audit.log(entry);
    }
    let mut failed = Vec::new();
    for out in app.miniapps.host.take_out() {
        match out {
            Out::Send {
                account,
                chat,
                text,
            } => {
                if let Some(account) = app
                    .accounts
                    .iter()
                    .find(|known| known.id.as_str() == account)
                    .map(|known| known.id.clone())
                {
                    app.deferred_account_actions.push((
                        account,
                        Action::SendText {
                            chat,
                            text,
                            quoting: None,
                        },
                    ));
                }
            }
            Out::Znga {
                key,
                tag,
                system,
                user,
            } => {
                let prompt = Prompt {
                    system,
                    turns: vec![Turn::user(user)],
                    max_tokens: 600,
                    effort: Effort::Low,
                };
                let purpose = Purpose::App {
                    key: key.clone(),
                    tag: tag.clone(),
                };
                if let Err(error) = app.znga.submit(prompt, purpose) {
                    failed.push((key, tag, error));
                }
            }
            Out::Load { key, account } => match archive_of(app, &account) {
                Some(archive) => app.miniapps.store.send(Job::Load { archive, key }),
                None => failed_load(app, key),
            },
            Out::Put {
                key,
                account,
                name,
                value,
            } => {
                if let Some(archive) = archive_of(app, &account) {
                    app.miniapps.store.send(Job::Put {
                        archive,
                        key,
                        name,
                        value,
                    });
                }
            }
            Out::Drop { key, account } => {
                if let Some(archive) = archive_of(app, &account) {
                    app.miniapps.store.send(Job::Drop { archive, key });
                }
            }
        }
    }
    for (key, tag, error) in failed {
        with_host(app, |host, world| {
            host.answered(world, &key, &tag, Err(error))
        });
    }
}

fn failed_load(app: &mut App, key: String) {
    with_host(app, |host, world| {
        host.loaded(world, &key, Err("its account is not available".into()))
    });
}

/// Every frame: store replies, wake-ups, and saving the install list.
pub fn tick(app: &mut App) {
    for reply in app.miniapps.store.poll() {
        with_host(app, |host, world| {
            host.loaded(world, &reply.key, reply.data)
        });
    }
    if app.is_linked() && !app.miniapps.host.saved.installs.is_empty() {
        with_host(app, |host, world| host.tick(world));
    }
    if std::mem::take(&mut app.miniapps.host.dirty)
        && let Ok(text) = serde_json::to_string_pretty(&app.miniapps.host.saved)
    {
        app.files
            .write(&app.miniapps.path, text, fastframe_file::Access::Default);
    }
}

/// Messages arriving in a chat (REQ-MINIAPP-003).
pub fn observe_messages(
    app: &mut App,
    chat: &str,
    messages: &[crate::model::Message],
    older: bool,
) {
    if older || messages.is_empty() || app.miniapps.host.saved.installs.is_empty() {
        return;
    }
    let account = app.account();
    let account_id = account.id.as_str().to_owned();
    if !app
        .miniapps
        .host
        .saved
        .installs
        .iter()
        .any(|i| i.account == account_id && i.chat == chat)
    {
        return;
    }
    let own = account
        .me
        .as_deref()
        .and_then(crate::znga::proactive::own_chat)
        .is_some_and(|own| own == chat);
    let incoming: Vec<Incoming> = messages
        .iter()
        .filter_map(|message| {
            let Content::Text { text, .. } = &message.content else {
                return None;
            };
            let from = if message.from_me {
                From::Me
            } else {
                From::Member {
                    id: message.sender.clone(),
                    name: message
                        .sender_name
                        .clone()
                        .unwrap_or_else(|| message.sender.clone()),
                }
            };
            Some(Incoming {
                id: &message.id,
                text,
                from,
                sent_at: message.timestamp,
                quoted: message.quoted.as_ref().map(|q| q.summary.as_str()),
            })
        })
        .collect();
    if incoming.is_empty() {
        return;
    }
    with_host(app, |host, world| {
        host.observe(world, &account_id, chat, own, &incoming, older)
    });
}

/// Whether a message in the own chat is a mini-app's rather than a
/// question for Znga (REQ-MINIAPP-008).
pub fn claims(app: &App, account: &str, chat: &str, message: &crate::model::Message) -> bool {
    let Content::Text { text, .. } = &message.content else {
        return false;
    };
    let quoted = message.quoted.as_ref().map(|q| q.summary.as_str());
    app.miniapps.host.claims(account, chat, text, quoted)
}

/// Znga answered an app's request.
pub fn znga_answered(app: &mut App, key: &str, tag: &str, answer: Result<String, String>) {
    with_host(app, |host, world| host.answered(world, key, tag, answer));
}

/// The person confirmed the install screen.
pub fn install(app: &mut App, sheet: Sheet) {
    let result = with_host(app, |host, world| {
        host.install(world, &sheet.account, &sheet.chat, sheet.kind, &sheet.app)
    });
    app.miniapps.notice = result.err();
}

pub fn uninstall(app: &mut App, key: &str) {
    app.miniapps.host.uninstall(key);
    app.miniapps
        .fields
        .retain(|(install, _, _), _| install != key);
    carry_out(app);
}

pub fn click(
    app: &mut App,
    key: &str,
    card: &str,
    control: Control,
    fields: BTreeMap<String, String>,
) {
    with_host(app, |host, world| {
        host.click(world, key, card, control, fields)
    });
}

pub fn restart(app: &mut App, key: &str) {
    app.miniapps.host.restart(key, crate::util::now());
}

/// Installs an app with sample data for demo screenshots: no store, no
/// sends.
#[cfg(any(test, feature = "demo"))]
pub(crate) fn demo_install(app: &mut App, chat: &str, data: BTreeMap<String, String>) {
    let account = app.account().id.as_str().to_owned();
    let mut host = std::mem::replace(
        &mut app.miniapps.host,
        Host::with_apps(Saved::default(), 0, Vec::new()),
    );
    let zone = jiff::tz::TimeZone::system();
    {
        let shared: &App = app;
        let usable = |_: &str, _: &str| true;
        let activity = |_: &str| Activity::default();
        let world = World {
            now: crate::util::now(),
            zone: &zone,
            znga_on: shared.znga.config.enabled,
            usable: &usable,
            activity: &activity,
        };
        if host
            .install(&world, &account, chat, ChatKind::Own, "goal")
            .is_ok()
        {
            let key = format!("{account}|{chat}|goal");
            host.loaded(&world, &key, Ok(data));
            // A wake-up redraws the cards from the sample data.
            host.restart(&key, world.now);
            host.tick(&world);
        }
    }
    host.take_out();
    host.take_audit();
    host.dirty = false;
    app.miniapps.host = host;
}
