//! Znga's durable memory (REQ-ZNGA-016): `znga.db`, an encrypted SQLite
//! file next to each account's archive and unlocked with the archive's own
//! OS-keyring key. It holds the Ask thread, facts about people, promises
//! and the Sentinel's audit log. A store thread also reads the archive for
//! Znga (seeding activity, archive search, relationship rhythms), so no
//! disk work runs on the interface thread.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};

use rusqlite::{Connection, params};

use super::archive_ask::{self, Hit, Visible};
use super::coach::{self, Rhythm};
use super::memory::Fact;
use super::proactive::Seen;
use super::promises::{Owes, Promise, State};
use super::provider::Turn;
use super::sentinel::{Entry, Origin};
use super::thread;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS ask_turns (
    position INTEGER PRIMARY KEY,
    from_user INTEGER NOT NULL,
    text TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS facts (
    chat TEXT NOT NULL,
    chat_title TEXT NOT NULL,
    text TEXT NOT NULL,
    at INTEGER NOT NULL,
    forgotten INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS promises (
    chat TEXT NOT NULL,
    chat_title TEXT NOT NULL,
    owes TEXT NOT NULL,
    text TEXT NOT NULL,
    due INTEGER,
    created INTEGER NOT NULL,
    state TEXT NOT NULL,
    reminded INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS thread (
    position INTEGER PRIMARY KEY,
    at INTEGER NOT NULL,
    from_user INTEGER NOT NULL,
    text TEXT NOT NULL,
    items TEXT NOT NULL,
    unasked INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS audit (
    at INTEGER NOT NULL,
    action TEXT NOT NULL,
    origin TEXT NOT NULL,
    allowed INTEGER NOT NULL,
    reason TEXT NOT NULL
);";

/// The most Ask turns kept.
pub const ASK_KEEP: usize = 200;

/// One account's `znga.db`.
pub struct Db {
    connection: Connection,
}

/// Everything loaded for one account.
#[derive(Default)]
pub struct Loaded {
    pub ask: Vec<Turn>,
    pub facts: Vec<Fact>,
    pub promises: Vec<Promise>,
    pub audit: Vec<Entry>,
    /// The Znga chat (REQ-ZCHAT-001).
    pub thread: Vec<thread::Entry>,
}

impl Db {
    pub fn new(connection: Connection) -> rusqlite::Result<Self> {
        connection.execute_batch(SCHEMA)?;
        // The cited messages of an answer (REQ-ZANS-001.A4), added after
        // the first release of the Znga chat.
        let has_refs = connection
            .prepare("SELECT 1 FROM pragma_table_info('thread') WHERE name = 'refs'")?
            .exists([])?;
        if !has_refs {
            connection
                .execute_batch("ALTER TABLE thread ADD COLUMN refs TEXT NOT NULL DEFAULT '[]'")?;
        }
        Ok(Self { connection })
    }

    /// Opens (or creates) `znga.db` beside `archive`, encrypted with the
    /// archive's key (REQ-ZNGA-016.A3).
    pub fn open_beside(archive: &Path) -> Result<Self, String> {
        let path = archive.with_file_name("znga.db");
        let connection = crate::archive::Archive::znga_connection(&path)
            .map_err(|error| format!("Znga's memory could not be opened: {error}"))?;
        Self::new(connection).map_err(|error| error.to_string())
    }

    pub fn load(&self, account: &str) -> rusqlite::Result<Loaded> {
        let c = &self.connection;
        let ask = c
            .prepare("SELECT from_user, text FROM ask_turns ORDER BY position")?
            .query_map([], |row| {
                Ok(Turn {
                    from_user: row.get(0)?,
                    text: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        let facts = c
            .prepare("SELECT chat, chat_title, text, at, forgotten FROM facts ORDER BY at")?
            .query_map([], |row| {
                Ok(Fact {
                    account: account.to_owned(),
                    chat: row.get(0)?,
                    chat_title: row.get(1)?,
                    text: row.get(2)?,
                    at: row.get(3)?,
                    forgotten: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        let promises = c
            .prepare(
                "SELECT chat, chat_title, owes, text, due, created, state, reminded
                 FROM promises ORDER BY created",
            )?
            .query_map([], |row| {
                Ok(Promise {
                    account: account.to_owned(),
                    chat: row.get(0)?,
                    chat_title: row.get(1)?,
                    owes: Owes::from_key(&row.get::<_, String>(2)?),
                    text: row.get(3)?,
                    due: row.get(4)?,
                    created: row.get(5)?,
                    state: State::from_key(&row.get::<_, String>(6)?),
                    reminded: row.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        let audit = c
            .prepare(
                "SELECT at, action, origin, allowed, reason FROM audit
                 ORDER BY rowid DESC LIMIT 500",
            )?
            .query_map([], |row| {
                Ok(Entry {
                    at: row.get(0)?,
                    action: row.get(1)?,
                    origin: Origin::from_key(&row.get::<_, String>(2)?),
                    allowed: row.get(3)?,
                    reason: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .rev()
            .collect();
        let thread = c
            .prepare(
                "SELECT position, at, from_user, text, items, unasked, refs
                 FROM thread ORDER BY position",
            )?
            .query_map([], |row| {
                let items: String = row.get(4)?;
                let refs: String = row.get(6)?;
                Ok(thread::Entry {
                    id: row.get::<_, i64>(0)? as u64 + 1,
                    at: row.get(1)?,
                    from_person: row.get(2)?,
                    text: row.get(3)?,
                    items: serde_json::from_str(&items).unwrap_or_default(),
                    unasked: row.get(5)?,
                    refs: serde_json::from_str(&refs).unwrap_or_default(),
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(Loaded {
            ask,
            facts,
            promises,
            audit,
            thread,
        })
    }

    pub fn save_ask(&mut self, turns: &[Turn]) -> rusqlite::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM ask_turns", [])?;
        let skip = turns.len().saturating_sub(ASK_KEEP);
        for (position, turn) in turns.iter().skip(skip).enumerate() {
            tx.execute(
                "INSERT INTO ask_turns (position, from_user, text) VALUES (?1, ?2, ?3)",
                params![position as i64, turn.from_user, turn.text],
            )?;
        }
        tx.commit()
    }

    pub fn save_thread(&mut self, entries: &[thread::Entry]) -> rusqlite::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM thread", [])?;
        let skip = entries.len().saturating_sub(thread::KEEP);
        for (position, entry) in entries.iter().skip(skip).enumerate() {
            let items = serde_json::to_string(&entry.items).unwrap_or_else(|_| "[]".into());
            let refs = serde_json::to_string(&entry.refs).unwrap_or_else(|_| "[]".into());
            tx.execute(
                "INSERT INTO thread (position, at, from_user, text, items, unasked, refs)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    position as i64,
                    entry.at,
                    entry.from_person,
                    entry.text,
                    items,
                    entry.unasked,
                    refs
                ],
            )?;
        }
        tx.commit()
    }

    pub fn save_facts(&mut self, facts: &[Fact]) -> rusqlite::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM facts", [])?;
        for fact in facts {
            tx.execute(
                "INSERT INTO facts (chat, chat_title, text, at, forgotten) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![fact.chat, fact.chat_title, fact.text, fact.at, fact.forgotten],
            )?;
        }
        tx.commit()
    }

    pub fn save_promises(&mut self, items: &[Promise]) -> rusqlite::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM promises", [])?;
        for p in items {
            tx.execute(
                "INSERT INTO promises (chat, chat_title, owes, text, due, created, state, reminded)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    p.chat,
                    p.chat_title,
                    p.owes.key(),
                    p.text,
                    p.due,
                    p.created,
                    p.state.key(),
                    p.reminded
                ],
            )?;
        }
        tx.commit()
    }

    pub fn append_audit(&mut self, entries: &[Entry]) -> rusqlite::Result<()> {
        let tx = self.connection.transaction()?;
        for e in entries {
            tx.execute(
                "INSERT INTO audit (at, action, origin, allowed, reason) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![e.at, e.action, e.origin.key(), e.allowed, e.reason],
            )?;
        }
        tx.execute(
            "DELETE FROM audit WHERE rowid NOT IN (SELECT rowid FROM audit ORDER BY rowid DESC LIMIT 500)",
            [],
        )?;
        tx.commit()
    }
}

/// Work for the store thread. `archive` is the account's archive path;
/// `znga.db` lives beside it.
pub enum Job {
    Load {
        account: String,
        archive: PathBuf,
    },
    SaveAsk {
        archive: PathBuf,
        turns: Vec<Turn>,
    },
    SaveFacts {
        archive: PathBuf,
        facts: Vec<Fact>,
    },
    SaveThread {
        archive: PathBuf,
        entries: Vec<thread::Entry>,
    },
    SavePromises {
        archive: PathBuf,
        items: Vec<Promise>,
    },
    Audit {
        archive: PathBuf,
        entries: Vec<Entry>,
    },
    /// Recent messages of visible chats, to start Znga's activity from.
    Seed {
        account: String,
        archive: PathBuf,
        visible: Visible,
        names: HashMap<String, String>,
    },
    Search {
        id: u64,
        archive: PathBuf,
        terms: Vec<String>,
        visible: Visible,
        names: HashMap<String, String>,
    },
    Rhythms {
        account: String,
        archive: PathBuf,
        chats: Vec<(String, String)>,
        now: i64,
    },
    /// The own chat's recent messages, for the self-chat thread.
    History {
        ask: u64,
        archive: PathBuf,
        chat: String,
    },
}

pub enum Reply {
    Loaded {
        account: String,
        loaded: Loaded,
    },
    Seeded {
        account: String,
        seen: Vec<Seen>,
    },
    Found {
        id: u64,
        hits: Result<Vec<Hit>, String>,
    },
    Rhythms {
        account: String,
        rhythms: Vec<Rhythm>,
    },
    /// (from me, text), oldest first.
    History {
        ask: u64,
        messages: Vec<(bool, String)>,
    },
    Failed(String),
}

/// The store thread's handle.
pub struct Store {
    sender: Option<Sender<Job>>,
    replies: Option<Receiver<Reply>>,
    waker: crate::backend::Waker,
}

/// Seeded activity: at most this many messages, from this many chats.
const SEED_MESSAGES: usize = 600;
const SEED_CHATS: usize = 40;
const SEED_PER_CHAT: usize = 30;

impl Store {
    pub fn new(waker: crate::backend::Waker) -> Self {
        Self {
            sender: None,
            replies: None,
            waker,
        }
    }

    pub fn send(&mut self, job: Job) {
        if self.sender.is_none() {
            self.start();
        }
        if let Some(sender) = &self.sender
            && sender.send(job).is_err()
        {
            self.sender = None;
        }
    }

    pub fn poll(&mut self) -> Vec<Reply> {
        let mut out = Vec::new();
        let Some(replies) = &self.replies else {
            return out;
        };
        loop {
            match replies.try_recv() {
                Ok(reply) => out.push(reply),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.sender = None;
                    self.replies = None;
                    break;
                }
            }
        }
        out
    }

    fn start(&mut self) {
        let (jobs, inbox) = channel::<Job>();
        let (outbox, replies) = channel::<Reply>();
        let waker = self.waker.clone();
        let spawned = std::thread::Builder::new()
            .name("znga-store".into())
            .spawn(move || {
                let mut dbs: HashMap<PathBuf, Db> = HashMap::new();
                for job in inbox {
                    if let Some(reply) = run(&mut dbs, job)
                        && outbox.send(reply).is_err()
                    {
                        break;
                    }
                    waker.wake();
                }
            });
        if spawned.is_ok() {
            self.sender = Some(jobs);
            self.replies = Some(replies);
        } else {
            log::warn!("znga: could not start its store thread");
        }
    }
}

fn db<'a>(dbs: &'a mut HashMap<PathBuf, Db>, archive: &Path) -> Result<&'a mut Db, String> {
    if !dbs.contains_key(archive) {
        let db = Db::open_beside(archive)?;
        dbs.insert(archive.to_owned(), db);
    }
    dbs.get_mut(archive)
        .ok_or_else(|| "Znga's memory is unavailable.".to_owned())
}

fn reader(archive: &Path) -> Result<crate::archive::Archive, String> {
    crate::archive::Archive::export_reader(archive)
        .map_err(|error| format!("Could not read the local archive: {error}"))
}

fn run(dbs: &mut HashMap<PathBuf, Db>, job: Job) -> Option<Reply> {
    let failed = |error: String| Some(Reply::Failed(error));
    match job {
        Job::Load { account, archive } => match db(dbs, &archive) {
            Ok(db) => match db.load(&account) {
                Ok(loaded) => Some(Reply::Loaded { account, loaded }),
                Err(error) => failed(error.to_string()),
            },
            Err(error) => failed(error),
        },
        Job::SaveAsk { archive, turns } => {
            let result =
                db(dbs, &archive).and_then(|db| db.save_ask(&turns).map_err(|e| e.to_string()));
            result.err().and_then(failed)
        }
        Job::SaveThread { archive, entries } => {
            let result = db(dbs, &archive)
                .and_then(|db| db.save_thread(&entries).map_err(|e| e.to_string()));
            result.err().and_then(failed)
        }
        Job::SaveFacts { archive, facts } => {
            let result =
                db(dbs, &archive).and_then(|db| db.save_facts(&facts).map_err(|e| e.to_string()));
            result.err().and_then(failed)
        }
        Job::SavePromises { archive, items } => {
            let result = db(dbs, &archive)
                .and_then(|db| db.save_promises(&items).map_err(|e| e.to_string()));
            result.err().and_then(failed)
        }
        Job::Audit { archive, entries } => {
            let result = db(dbs, &archive)
                .and_then(|db| db.append_audit(&entries).map_err(|e| e.to_string()));
            result.err().and_then(failed)
        }
        Job::Seed {
            account,
            archive,
            visible,
            names,
        } => match reader(&archive) {
            Ok(reader) => Some(Reply::Seeded {
                seen: seed(&reader, &account, &visible, &names),
                account,
            }),
            Err(error) => failed(error),
        },
        Job::Search {
            id,
            archive,
            terms,
            visible,
            names,
        } => Some(Reply::Found {
            id,
            hits: reader(&archive)
                .map(|reader| archive_ask::search(&reader, &terms, &visible, &names, 40)),
        }),
        Job::Rhythms {
            account,
            archive,
            chats,
            now,
        } => match reader(&archive) {
            Ok(reader) => {
                let rhythms = chats
                    .iter()
                    .filter_map(|(chat, title)| {
                        let messages = reader.messages(chat, None, 400).ok()?;
                        let times: Vec<(i64, bool)> =
                            messages.iter().map(|m| (m.timestamp, m.from_me)).collect();
                        coach::assess(&account, chat, title, &times, now)
                    })
                    .collect();
                Some(Reply::Rhythms { account, rhythms })
            }
            Err(error) => failed(error),
        },
        Job::History { ask, archive, chat } => {
            let messages = reader(&archive)
                .and_then(|reader| reader.messages(&chat, None, 60).map_err(|e| e.to_string()))
                .unwrap_or_default()
                .into_iter()
                .filter_map(|message| match message.content {
                    crate::model::Content::Text { text, .. } => Some((message.from_me, text)),
                    _ => None,
                })
                .collect();
            Some(Reply::History { ask, messages })
        }
    }
}

/// The recent messages of visible chats, oldest first (REQ-ZNGA-016.A2).
pub fn seed(
    archive: &crate::archive::Archive,
    account: &str,
    visible: &Visible,
    names: &HashMap<String, String>,
) -> Vec<Seen> {
    let mut chats: Vec<(i64, String)> = archive
        .chats()
        .unwrap_or_default()
        .into_iter()
        .filter(|chat| visible.contains_key(&chat.id) && !chat.locked)
        .map(|chat| (chat.last_activity, chat.id))
        .collect();
    chats.sort_by_key(|chat| std::cmp::Reverse(chat.0));
    let mut seen: Vec<Seen> = Vec::new();
    for (_, chat) in chats.into_iter().take(SEED_CHATS) {
        let Some((title, group)) = visible.get(&chat) else {
            continue;
        };
        for message in archive
            .messages(&chat, None, SEED_PER_CHAT)
            .unwrap_or_default()
        {
            if message.from_me || matches!(message.content, crate::model::Content::Call { .. }) {
                continue;
            }
            seen.push(Seen {
                account: account.to_owned(),
                chat: chat.clone(),
                id: message.id.clone(),
                chat_title: title.clone(),
                group: *group,
                sender: message.sender.clone(),
                sender_name: message
                    .sender_name
                    .clone()
                    .or_else(|| names.get(&message.sender).cloned())
                    .unwrap_or_else(|| title.clone()),
                text: message.content.full_summary(),
                at: message.timestamp,
                score: Default::default(),
            });
        }
    }
    seen.sort_by_key(|item| item.at);
    let skip = seen.len().saturating_sub(SEED_MESSAGES);
    seen.split_off(skip)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Db {
        Db::new(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn the_ask_thread_survives_a_reload() {
        let mut db = db();
        let turns = vec![Turn::user("when does Ravi land?"), Turn::assistant("At 9.")];
        db.save_ask(&turns).unwrap();
        assert_eq!(db.load("1").unwrap().ask, turns);
    }

    #[test]
    fn the_znga_chat_survives_a_reload() {
        let mut db = db();
        let mut chat = thread::Thread::default();
        chat.post_person("Do I owe anyone a reply?", 1);
        chat.post(
            "Ada asked about Sunday.",
            vec![thread::Item::new(thread::Card::CatchUp {
                chat: "ada".into(),
                chat_title: "Ada".into(),
            })],
            2,
            false,
        );
        db.save_thread(chat.entries()).unwrap();
        assert_eq!(db.load("1").unwrap().thread, chat.entries());
    }

    #[test]
    fn an_answers_links_survive_a_reload() {
        let mut db = db();
        let mut chat = thread::Thread::default();
        let cited =
            crate::znga::answer::Ref::new(1, "ada", "Ada", "m1", "Ada", 5, "ada@example.com");
        chat.post_cited("Ada gave it [1].", Vec::new(), vec![cited], 2, false);
        db.save_thread(chat.entries()).unwrap();
        let loaded = db.load("1").unwrap().thread;
        assert_eq!(loaded, chat.entries());
        assert_eq!(loaded[0].refs[0].message, "m1");
    }

    #[test]
    fn a_znga_db_from_before_links_gains_them() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE thread (position INTEGER PRIMARY KEY, at INTEGER NOT NULL,
                 from_user INTEGER NOT NULL, text TEXT NOT NULL, items TEXT NOT NULL,
                 unasked INTEGER NOT NULL);
                 INSERT INTO thread VALUES (0, 1, 0, 'old answer', '[]', 0);",
            )
            .unwrap();
        let db = Db::new(connection).unwrap();
        let loaded = db.load("1").unwrap().thread;
        assert_eq!(loaded[0].text, "old answer");
        assert!(loaded[0].refs.is_empty());
    }

    #[test]
    fn the_znga_chat_file_holds_no_readable_text() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("znga.db");
        let key = [7u8; 32];
        let open = || {
            Db::new(crate::archive::Archive::znga_connection_with_key(&path, &key).unwrap())
                .unwrap()
        };
        let mut chat = thread::Thread::default();
        chat.post_person("Where is the Zanzibar ticket?", 1);
        chat.post("In the Zanzibar group, on Monday.", Vec::new(), 2, false);
        let mut db = open();
        db.save_thread(chat.entries()).unwrap();
        drop(db);
        let mut bytes = std::fs::read(&path).unwrap();
        for extra in ["znga.db-wal", "znga.db-journal"] {
            if let Ok(more) = std::fs::read(dir.path().join(extra)) {
                bytes.extend(more);
            }
        }
        assert!(!bytes.windows(8).any(|w| w == b"Zanzibar"));
        assert_eq!(open().load("1").unwrap().thread, chat.entries());
    }

    #[test]
    fn facts_promises_and_audit_round_trip() {
        let mut db = db();
        let fact = Fact {
            account: "1".into(),
            chat: "ada".into(),
            chat_title: "Ada".into(),
            text: "Birthday 12 March".into(),
            at: 5,
            forgotten: true,
        };
        db.save_facts(std::slice::from_ref(&fact)).unwrap();
        let promise = Promise {
            account: "1".into(),
            chat: "ada".into(),
            chat_title: "Ada".into(),
            owes: Owes::Them,
            text: "Return the book".into(),
            due: Some(99),
            created: 1,
            state: State::Done,
            reminded: true,
        };
        db.save_promises(std::slice::from_ref(&promise)).unwrap();
        let entry = Entry {
            at: 3,
            action: "draft".into(),
            origin: Origin::OthersText,
            allowed: true,
            reason: "drafts wait".into(),
        };
        db.append_audit(std::slice::from_ref(&entry)).unwrap();
        let loaded = db.load("1").unwrap();
        assert_eq!(loaded.facts, vec![fact]);
        assert_eq!(loaded.promises, vec![promise]);
        assert_eq!(loaded.audit, vec![entry]);
    }

    #[test]
    fn seeding_reads_only_visible_chats() {
        let archive = crate::archive::Archive::in_memory().unwrap();
        for chat in ["a@s.whatsapp.net", "hidden@s.whatsapp.net"] {
            archive.ensure_chat(chat, "x").unwrap();
            archive
                .insert_message(
                    &crate::znga::archive_ask::tests_message(chat, "hello", 10),
                    None,
                )
                .unwrap();
        }
        let mut visible = Visible::new();
        visible.insert("a@s.whatsapp.net".into(), ("Ada".into(), false));
        let seen = seed(&archive, "1", &visible, &HashMap::new());
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].chat_title, "Ada");
        assert_eq!(seen[0].text, "hello");
    }
}
