//! Facts about people (REQ-ZNGA-019), kept on this computer only, in the
//! encrypted `znga.db`. Extracted from chats Znga may see; the person can
//! read, add and delete them, and a deleted fact never comes back.

use serde::Deserialize;

/// The JSON array in a model answer, or `None`.
pub fn json_array<T: for<'de> Deserialize<'de>>(answer: &str) -> Option<Vec<T>> {
    let start = answer.find('[')?;
    let end = answer.rfind(']')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&answer[start..=end]).ok()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub account: String,
    /// The contact's chat id.
    pub chat: String,
    pub chat_title: String,
    pub text: String,
    pub at: i64,
    /// Deleted by the person: kept as a tombstone so extraction does not
    /// bring it back.
    pub forgotten: bool,
}

#[derive(Deserialize)]
struct Extracted {
    fact: String,
}

/// Facts in an extraction answer: `[{"fact": "…"}]`.
pub fn parse_facts(answer: &str) -> Vec<String> {
    json_array::<Extracted>(answer)
        .unwrap_or_default()
        .into_iter()
        .map(|item| item.fact.trim().trim_end_matches('.').to_owned())
        .filter(|fact| !fact.is_empty() && fact.chars().count() <= 200)
        .collect()
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

/// Everything Znga remembers about people, across accounts.
#[derive(Default)]
pub struct People {
    facts: Vec<Fact>,
    dirty: bool,
}

/// At most this many live facts per contact; the oldest give way.
const PER_CONTACT: usize = 20;

impl People {
    pub fn restore(&mut self, facts: Vec<Fact>) {
        for fact in facts {
            if !self.contains(&fact.account, &fact.chat, &fact.text) {
                self.facts.push(fact);
            }
        }
    }

    fn contains(&self, account: &str, chat: &str, text: &str) -> bool {
        let wanted = normal(text);
        self.facts
            .iter()
            .any(|f| f.account == account && f.chat == chat && normal(&f.text) == wanted)
    }

    /// Adds facts not already known or forgotten. Returns how many were new.
    pub fn add(
        &mut self,
        account: &str,
        chat: &str,
        chat_title: &str,
        texts: &[String],
        at: i64,
    ) -> usize {
        let mut added = 0;
        for text in texts {
            if self.contains(account, chat, text) {
                continue;
            }
            self.facts.push(Fact {
                account: account.into(),
                chat: chat.into(),
                chat_title: chat_title.into(),
                text: text.clone(),
                at,
                forgotten: false,
            });
            added += 1;
        }
        let mut live: Vec<usize> = self
            .facts
            .iter()
            .enumerate()
            .filter(|(_, f)| f.account == account && f.chat == chat && !f.forgotten)
            .map(|(i, _)| i)
            .collect();
        while live.len() > PER_CONTACT {
            let oldest = live.remove(0);
            self.facts.remove(oldest);
            for index in &mut live {
                *index -= 1;
            }
        }
        if added > 0 {
            self.dirty = true;
        }
        added
    }

    /// Forgets a fact; extraction will not add it again.
    pub fn forget(&mut self, account: &str, chat: &str, text: &str) {
        for fact in &mut self.facts {
            if fact.account == account && fact.chat == chat && fact.text == text {
                fact.forgotten = true;
                self.dirty = true;
            }
        }
    }

    /// Drops everything about chats Znga may no longer see.
    pub fn retain(&mut self, keep: impl Fn(&Fact) -> bool) {
        let before = self.facts.len();
        self.facts.retain(keep);
        if self.facts.len() != before {
            self.dirty = true;
        }
    }

    /// The live facts about one contact, oldest first.
    pub fn about(&self, account: &str, chat: &str) -> Vec<String> {
        self.facts
            .iter()
            .filter(|f| f.account == account && f.chat == chat && !f.forgotten)
            .map(|f| f.text.clone())
            .collect()
    }

    /// Live facts for one account, grouped by contact title.
    pub fn for_account(&self, account: &str) -> Vec<&Fact> {
        let mut facts: Vec<&Fact> = self
            .facts
            .iter()
            .filter(|f| f.account == account && !f.forgotten)
            .collect();
        facts.sort_by(|a, b| a.chat_title.cmp(&b.chat_title).then(a.at.cmp(&b.at)));
        facts
    }

    pub fn all(&self) -> &[Fact] {
        &self.facts
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

    #[test]
    fn facts_are_parsed_from_the_answer() {
        let answer = "Here you go:\n[{\"fact\": \"Birthday is 12 March.\"}, {\"fact\": \"\"}]";
        assert_eq!(parse_facts(answer), vec!["Birthday is 12 March"]);
        assert!(parse_facts("nothing").is_empty());
    }

    #[test]
    fn a_fact_is_stored_once_and_a_forgotten_one_stays_forgotten() {
        let mut people = People::default();
        let facts = vec!["Allergic to peanuts".to_owned()];
        assert_eq!(people.add("1", "ada", "Ada", &facts, 1), 1);
        assert_eq!(
            people.add("1", "ada", "Ada", &["allergic to peanuts!".into()], 2),
            0
        );
        assert_eq!(people.about("1", "ada"), facts);
        // Another account's contact of the same id is someone else.
        assert!(people.about("2", "ada").is_empty());
        people.forget("1", "ada", "Allergic to peanuts");
        assert!(people.about("1", "ada").is_empty());
        assert_eq!(people.add("1", "ada", "Ada", &facts, 3), 0);
        assert!(people.about("1", "ada").is_empty());
    }

    #[test]
    fn each_contact_keeps_its_newest_facts() {
        let mut people = People::default();
        for n in 0..25 {
            people.add("1", "ada", "Ada", &[format!("fact {n}")], n);
        }
        let about = people.about("1", "ada");
        assert_eq!(about.len(), PER_CONTACT);
        assert_eq!(about[0], "fact 5");
    }
}
