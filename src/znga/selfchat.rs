//! Chatting with Znga in the person's own "Message yourself" chat
//! (REQ-ZNGA-012..014). Every text the person sends there is a question;
//! Znga answers in the same chat, starting with [`MARKER`] so its answers
//! are never taken for questions, and the thread is rebuilt from the chat's
//! own history in the archive.

use std::collections::{HashSet, VecDeque};

use super::provider::Turn;

/// What every Znga message in the own chat starts with.
pub const MARKER: &str = "✨ Znga";

/// Older messages than this when they arrive are not answered
/// (REQ-ZNGA-014.A3).
pub const MAX_AGE: i64 = 5 * 60;

/// Answers a day (REQ-ZNGA-014.A6).
pub const PER_DAY: u32 = 100;

/// The most self-chat turns put in a prompt (REQ-ZNGA-013.A4).
pub const MAX_TURNS: usize = 20;

/// `text` as Znga's answer in the own chat.
pub fn marked(text: &str) -> String {
    format!("{MARKER}\n{}", text.trim())
}

/// Whether a message in the own chat is one of Znga's.
pub fn is_znga(text: &str) -> bool {
    text.trim_start().starts_with(MARKER)
}

/// The self-chat thread as turns, from the chat's messages (oldest first,
/// as (from me, text)): Znga's marked messages are its turns, the person's
/// other texts are theirs (REQ-ZNGA-013).
pub fn turns(messages: &[(bool, String)]) -> Vec<Turn> {
    let mut turns: Vec<Turn> = messages
        .iter()
        .filter(|(from_me, text)| *from_me && !text.trim().is_empty())
        .map(|(_, text)| {
            if is_znga(text) {
                let body = text.trim_start()[MARKER.len()..]
                    .trim_start_matches(" briefing")
                    .trim();
                Turn::assistant(body)
            } else {
                Turn::user(text.trim())
            }
        })
        .collect();
    let skip = turns.len().saturating_sub(MAX_TURNS);
    turns.drain(..skip);
    turns
}

/// A question waiting for an answer.
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    pub account: String,
    pub chat: String,
    pub message: String,
    pub text: String,
}

/// What the daily cap allows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Allowance {
    Answer,
    /// The cap was just reached: send one notice instead.
    Notice,
    Refuse,
}

#[derive(Default)]
pub struct SelfChat {
    answered: HashSet<String>,
    answered_order: VecDeque<String>,
    pub queue: VecDeque<Question>,
    /// Whether an answer is being written.
    pub busy: bool,
    day: i64,
    sent: u32,
    noticed: bool,
}

impl SelfChat {
    /// Whether a message in the own chat is a new question
    /// (REQ-ZNGA-014.A1..A4). `since` is when the switch was turned on.
    #[allow(clippy::too_many_arguments)]
    pub fn accept(
        &mut self,
        id: &str,
        text: Option<&str>,
        from_me: bool,
        sent_at: i64,
        since: i64,
        now: i64,
        older: bool,
    ) -> bool {
        let Some(text) = text else {
            return false;
        };
        if older
            || !from_me
            || text.trim().is_empty()
            || is_znga(text)
            || sent_at < since
            || now - sent_at > MAX_AGE
            || self.answered.contains(id)
        {
            return false;
        }
        self.answered.insert(id.to_owned());
        self.answered_order.push_back(id.to_owned());
        if self.answered_order.len() > 1000
            && let Some(old) = self.answered_order.pop_front()
        {
            self.answered.remove(&old);
        }
        true
    }

    /// Takes one of today's answers (REQ-ZNGA-014.A6).
    pub fn take(&mut self, now: i64) -> Allowance {
        let day = now.div_euclid(86_400);
        if day != self.day {
            self.day = day;
            self.sent = 0;
            self.noticed = false;
        }
        if self.sent < PER_DAY {
            self.sent += 1;
            Allowance::Answer
        } else if !self.noticed {
            self.noticed = true;
            Allowance::Notice
        } else {
            Allowance::Refuse
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_new_live_texts_from_the_person_are_questions() {
        let mut chat = SelfChat::default();
        let now = 10_000;
        assert!(chat.accept("a", Some("what's on today?"), true, now - 5, 0, now, false));
        // The same message again (status update, re-sync).
        assert!(!chat.accept("a", Some("what's on today?"), true, now - 5, 0, now, false));
        // Znga's own answers and briefings.
        assert!(!chat.accept("b", Some(&marked("hello")), true, now, 0, now, false));
        assert!(!chat.accept("c", Some("✨ Znga briefing\n…"), true, now, 0, now, false));
        // Replayed or old history.
        assert!(!chat.accept("d", Some("old"), true, now - MAX_AGE - 1, 0, now, false));
        assert!(!chat.accept("e", Some("older page"), true, now, 0, now, true));
        // Before the switch was turned on.
        assert!(!chat.accept("f", Some("before"), true, now - 10, now - 5, now, false));
        // Not text.
        assert!(!chat.accept("g", None, true, now, 0, now, false));
    }

    #[test]
    fn the_hundred_and_first_answer_is_a_single_notice() {
        let mut chat = SelfChat::default();
        for _ in 0..PER_DAY {
            assert_eq!(chat.take(0), Allowance::Answer);
        }
        assert_eq!(chat.take(0), Allowance::Notice);
        assert_eq!(chat.take(0), Allowance::Refuse);
        assert_eq!(chat.take(86_400), Allowance::Answer);
    }

    #[test]
    fn the_thread_is_rebuilt_from_the_chat() {
        let messages = vec![
            (true, "who is Ada?".to_owned()),
            (true, marked("Your cousin.")),
            (true, "✨ Znga briefing\nAll quiet.".to_owned()),
            (true, "and her sister?".to_owned()),
        ];
        let turns = turns(&messages);
        assert_eq!(turns.len(), 4);
        assert!(turns[0].from_user && turns[0].text == "who is Ada?");
        assert!(!turns[1].from_user && turns[1].text == "Your cousin.");
        assert!(!turns[2].from_user && turns[2].text == "All quiet.");
        let many: Vec<(bool, String)> = (0..30).map(|n| (true, format!("q{n}"))).collect();
        assert_eq!(super::turns(&many).len(), MAX_TURNS);
    }
}
