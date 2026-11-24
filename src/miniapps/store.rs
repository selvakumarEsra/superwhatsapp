//! Mini-app data (REQ-MINIAPP-006): `miniapps.db`, an encrypted SQLite
//! file beside each account's archive, unlocked with the archive's key like
//! `znga.db`. Each install's values are kept under its own key. All disk
//! work runs on the store thread.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};

use rusqlite::{Connection, params};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS app_values (
    install TEXT NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (install, name)
);";

/// One account's `miniapps.db`.
pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn new(connection: Connection) -> rusqlite::Result<Self> {
        connection.execute_batch(SCHEMA)?;
        Ok(Self { connection })
    }

    /// Opens (or creates) `miniapps.db` beside `archive`, encrypted with
    /// the archive's key (REQ-MINIAPP-006.A2).
    pub fn open_beside(archive: &Path) -> Result<Self, String> {
        let path = archive.with_file_name("miniapps.db");
        let connection = crate::archive::Archive::znga_connection(&path)
            .map_err(|error| format!("could not open mini-app data: {error}"))?;
        Self::new(connection).map_err(|error| error.to_string())
    }

    /// One install's values only (REQ-MINIAPP-006.A3).
    pub fn load(&self, install: &str) -> rusqlite::Result<BTreeMap<String, String>> {
        let mut statement = self
            .connection
            .prepare("SELECT name, value FROM app_values WHERE install = ?1")?;
        let rows = statement.query_map([install], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    pub fn put(&self, install: &str, name: &str, value: Option<&str>) -> rusqlite::Result<()> {
        match value {
            Some(value) => self.connection.execute(
                "INSERT INTO app_values (install, name, value) VALUES (?1, ?2, ?3)
                 ON CONFLICT (install, name) DO UPDATE SET value = excluded.value",
                params![install, name, value],
            )?,
            None => self.connection.execute(
                "DELETE FROM app_values WHERE install = ?1 AND name = ?2",
                params![install, name],
            )?,
        };
        Ok(())
    }

    pub fn drop_install(&self, install: &str) -> rusqlite::Result<()> {
        self.connection
            .execute("DELETE FROM app_values WHERE install = ?1", [install])?;
        Ok(())
    }
}

pub enum Job {
    Load {
        archive: PathBuf,
        key: String,
    },
    Put {
        archive: PathBuf,
        key: String,
        name: String,
        value: Option<String>,
    },
    Drop {
        archive: PathBuf,
        key: String,
    },
}

pub struct Reply {
    pub key: String,
    pub data: Result<BTreeMap<String, String>, String>,
}

/// The store thread, started on first use.
pub struct Store {
    sender: Option<Sender<Job>>,
    replies: Option<Receiver<Reply>>,
    waker: crate::backend::Waker,
}

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
            .name("miniapps-store".into())
            .spawn(move || {
                let mut dbs: HashMap<PathBuf, Db> = HashMap::new();
                for job in inbox {
                    if let Some(reply) = run(&mut dbs, job) {
                        if outbox.send(reply).is_err() {
                            break;
                        }
                        waker.wake();
                    }
                }
            });
        if spawned.is_ok() {
            self.sender = Some(jobs);
            self.replies = Some(replies);
        } else {
            log::warn!("miniapps: could not start the store thread");
        }
    }
}

fn run(dbs: &mut HashMap<PathBuf, Db>, job: Job) -> Option<Reply> {
    let archive = match &job {
        Job::Load { archive, .. } | Job::Put { archive, .. } | Job::Drop { archive, .. } => {
            archive.clone()
        }
    };
    if !dbs.contains_key(&archive) {
        match Db::open_beside(&archive) {
            Ok(db) => {
                dbs.insert(archive.clone(), db);
            }
            Err(error) => {
                return match job {
                    Job::Load { key, .. } => Some(Reply {
                        key,
                        data: Err(error),
                    }),
                    _ => {
                        log::warn!("miniapps: {error}");
                        None
                    }
                };
            }
        }
    }
    let db = dbs.get(&archive)?;
    match job {
        Job::Load { key, .. } => {
            let data = db.load(&key).map_err(|error| error.to_string());
            Some(Reply { key, data })
        }
        Job::Put {
            key, name, value, ..
        } => {
            if let Err(error) = db.put(&key, &name, value.as_deref()) {
                log::warn!("miniapps: could not save a value: {error}");
            }
            None
        }
        Job::Drop { key, .. } => {
            if let Err(error) = db.drop_install(&key) {
                log::warn!("miniapps: could not delete an install's data: {error}");
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_kept_per_install() {
        let db = Db::new(Connection::open_in_memory().unwrap()).unwrap();
        db.put("a|me|goal", "state", Some("1")).unwrap();
        db.put("a|me|goal", "state", Some("2")).unwrap();
        db.put("a|g|other", "state", Some("x")).unwrap();
        let mine = db.load("a|me|goal").unwrap();
        assert_eq!(mine.len(), 1);
        assert_eq!(mine["state"], "2");
        db.drop_install("a|me|goal").unwrap();
        assert!(db.load("a|me|goal").unwrap().is_empty());
        assert_eq!(db.load("a|g|other").unwrap().len(), 1);
    }

    #[test]
    fn values_survive_a_restart_and_need_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("miniapps.db");
        let key = [7u8; 32];
        {
            let connection = crate::archive::Archive::encrypted_for_tests(&path, &key).unwrap();
            let db = Db::new(connection).unwrap();
            db.put("k", "state", Some("kept")).unwrap();
        }
        let connection = crate::archive::Archive::encrypted_for_tests(&path, &key).unwrap();
        let db = Db::new(connection).unwrap();
        assert_eq!(db.load("k").unwrap()["state"], "kept");
        assert!(crate::archive::Archive::encrypted_for_tests(&path, &[9u8; 32]).is_err());
    }
}
