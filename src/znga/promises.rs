//! Promises made in chats (REQ-ZNGA-018): what the person said they would
//! do, and what others asked of them, with a due time when one was said.
//! Kept in the encrypted `znga.db`.

use serde::Deserialize;

use super::memory::json_array;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owes {
    /// The person promised it.
    Me,
    /// Someone asked the person for it.
    Them,
}

impl Owes {
    pub fn key(self) -> &'static str {
        match self {
            Owes::Me => "me",
            Owes::Them => "them",
        }
    }

    pub fn from_key(key: &str) -> Self {
        if key == "them" { Owes::Them } else { Owes::Me }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Open,
    Done,
    Dismissed,
}

impl State {
    pub fn key(self) -> &'static str {
        match self {
            State::Open => "open",
            State::Done => "done",
            State::Dismissed => "dismissed",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "done" => State::Done,
            "dismissed" => State::Dismissed,
            _ => State::Open,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Promise {
    pub account: String,
    pub chat: String,
    pub chat_title: String,
    pub owes: Owes,
    pub text: String,
    /// Unix seconds, when a time was said.
    pub due: Option<i64>,
    pub created: i64,
    pub state: State,
    /// When the person was reminded, so it happens once.
    pub reminded: bool,
}

#[derive(Deserialize)]
struct Extracted {
    who: String,
    what: String,
    #[serde(default)]
    due: Option<String>,
}

/// Promises in an extraction answer:
/// `[{"who": "me"|"them", "what": "…", "due": "2026-11-20T18:00"|null}]`.
/// Due times are local wall-clock times.
pub fn parse(answer: &str, zone: &jiff::tz::TimeZone) -> Vec<(Owes, String, Option<i64>)> {
    json_array::<Extracted>(answer)
        .unwrap_or_default()
        .into_iter()
        .filter(|item| !item.what.trim().is_empty())
        .map(|item| {
            let owes = if item.who.trim().eq_ignore_ascii_case("them") {
                Owes::Them
            } else {
                Owes::Me
            };
            let due = item.due.as_deref().and_then(|due| local_time(due, zone));
            (owes, item.what.trim().to_owned(), due)
        })
        .collect()
}

/// A local `YYYY-MM-DD[THH:MM]` time; a date alone means 09:00.
pub fn local_time(text: &str, zone: &jiff::tz::TimeZone) -> Option<i64> {
    let text = text.trim();
    // A date alone means 09:00 that day (jiff would read it as midnight).
    let datetime: jiff::civil::DateTime = if text.contains(['T', ' ']) {
        text.parse().ok()?
    } else {
        text.parse::<jiff::civil::Date>().ok()?.at(9, 0, 0, 0)
    };
    Some(
        datetime
            .to_zoned(zone.clone())
            .ok()?
            .timestamp()
            .as_second(),
    )
}

fn normal(text: &str) -> String {
    text.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// How long before a due time the person is reminded.
pub const REMIND_BEFORE: i64 = 3 * 3600;

#[derive(Default)]
pub struct Promises {
    items: Vec<Promise>,
    dirty: bool,
}

impl Promises {
    /// Merges saved promises with any found before they loaded; marks the
    /// result dirty when both had some, so the merge is saved.
    pub fn restore(&mut self, items: Vec<Promise>) {
        let had = self.items.len();
        for item in items {
            let known = self.items.iter().any(|p| {
                p.account == item.account
                    && p.chat == item.chat
                    && normal(&p.text) == normal(&item.text)
            });
            if !known {
                self.items.push(item);
            }
        }
        if had > 0 {
            self.dirty = true;
        }
    }

    /// Adds promises not already known for that chat. Returns how many.
    pub fn add(
        &mut self,
        account: &str,
        chat: &str,
        chat_title: &str,
        found: Vec<(Owes, String, Option<i64>)>,
        now: i64,
    ) -> usize {
        let mut added = 0;
        for (owes, text, due) in found {
            let known = self.items.iter().any(|p| {
                p.account == account && p.chat == chat && normal(&p.text) == normal(&text)
            });
            if known {
                continue;
            }
            self.items.push(Promise {
                account: account.into(),
                chat: chat.into(),
                chat_title: chat_title.into(),
                owes,
                text,
                due,
                created: now,
                state: State::Open,
                reminded: false,
            });
            added += 1;
        }
        if added > 0 {
            self.dirty = true;
        }
        added
    }

    /// Open promises coming due within [`REMIND_BEFORE`] (or overdue) that
    /// were not reminded yet; marks them reminded (REQ-ZNGA-018.A3).
    pub fn due(&mut self, now: i64) -> Vec<Promise> {
        let mut out = Vec::new();
        for promise in &mut self.items {
            if promise.state == State::Open
                && !promise.reminded
                && promise.due.is_some_and(|due| due - now <= REMIND_BEFORE)
            {
                promise.reminded = true;
                out.push(promise.clone());
            }
        }
        if !out.is_empty() {
            self.dirty = true;
        }
        out
    }

    pub fn set_state(&mut self, index: usize, state: State) {
        if let Some(promise) = self.items.get_mut(index) {
            promise.state = state;
            self.dirty = true;
        }
    }

    /// Open promises, soonest first, undated last.
    pub fn open(&self) -> Vec<(usize, &Promise)> {
        let mut open: Vec<(usize, &Promise)> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, p)| p.state == State::Open)
            .collect();
        open.sort_by_key(|(_, p)| p.due.unwrap_or(i64::MAX));
        open
    }

    pub fn retain(&mut self, keep: impl Fn(&Promise) -> bool) {
        let before = self.items.len();
        self.items.retain(keep);
        if before != self.items.len() {
            self.dirty = true;
        }
    }

    pub fn all(&self) -> &[Promise] {
        &self.items
    }

    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc() -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::UTC
    }

    #[test]
    fn promises_are_parsed_with_who_and_when() {
        let answer = r#"[{"who":"me","what":"Send Ada the photos","due":"2026-11-20T18:00"},
                         {"who":"them","what":"Lend Ravi the drill","due":null},
                         {"who":"me","what":"  ","due":null}]"#;
        let found = parse(answer, &utc());
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].0, Owes::Me);
        assert_eq!(found[0].1, "Send Ada the photos");
        let expected: jiff::Timestamp = "2026-11-20T18:00:00Z".parse().unwrap();
        assert_eq!(found[0].2, Some(expected.as_second()));
        assert_eq!(found[1].0, Owes::Them);
        assert_eq!(found[1].2, None);
        assert_eq!(
            local_time("2026-11-20", &utc()),
            local_time("2026-11-20T09:00", &utc())
        );
    }

    #[test]
    fn the_same_promise_is_kept_once() {
        let mut promises = Promises::default();
        let found = || vec![(Owes::Me, "Send the photos".to_owned(), None)];
        assert_eq!(promises.add("1", "ada", "Ada", found(), 0), 1);
        assert_eq!(
            promises.add(
                "1",
                "ada",
                "Ada",
                vec![(Owes::Me, "send  the photos".into(), None)],
                1
            ),
            0
        );
        assert_eq!(promises.open().len(), 1);
    }

    #[test]
    fn a_due_promise_reminds_once_and_never_after_done() {
        let mut promises = Promises::default();
        promises.add(
            "1",
            "ada",
            "Ada",
            vec![(Owes::Me, "a".into(), Some(10_000))],
            0,
        );
        promises.add(
            "1",
            "bob",
            "Bob",
            vec![(Owes::Me, "b".into(), Some(100_000))],
            0,
        );
        promises.add(
            "1",
            "cat",
            "Cat",
            vec![(Owes::Me, "c".into(), Some(9_000))],
            0,
        );
        promises.set_state(2, State::Done);
        let now = 10_000 - REMIND_BEFORE + 1;
        let due = promises.due(now);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].text, "a");
        assert!(promises.due(now + 60).is_empty());
        assert!(promises.due(200_000).iter().all(|p| p.text == "b"));
    }
}
