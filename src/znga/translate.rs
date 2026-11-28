//! Translating messages from other people (REQ-ZANS-004).
//!
//! Which messages look like another language is decided here, on this
//! computer, from the letters' scripts and common words: nothing is sent
//! until the person clicks Translate. The translations are kept for the
//! session and shown in a popup over the chat.

use std::collections::{HashMap, HashSet};

/// The language a message seems to be in, when it can tell.
pub fn language_of(text: &str) -> Option<&'static str> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    let mut letters = 0usize;
    for c in strip_noise(text).chars() {
        if !c.is_alphabetic() {
            continue;
        }
        letters += 1;
        *counts.entry(script(c)).or_default() += 1;
    }
    if letters < 4 {
        return None;
    }
    let (top, count) = counts
        .iter()
        .max_by_key(|(_, n)| **n)
        .map(|(s, n)| (*s, *n))?;
    // Mostly one script, or mixed text that says nothing clear.
    if count * 10 < letters * 6 {
        return None;
    }
    match top {
        "Latin" => latin_language(text),
        "Other" => None,
        other => Some(other),
    }
}

/// Links, mentions, emoji shortcodes and numbers say nothing about the
/// language.
fn strip_noise(text: &str) -> String {
    text.split_whitespace()
        .filter(|word| {
            let lower = word.to_lowercase();
            !(lower.starts_with("http")
                || lower.starts_with("www.")
                || word.starts_with('@')
                || word.contains('@') && word.contains('.')
                || (word.starts_with(':') && word.ends_with(':')))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn script(c: char) -> &'static str {
    match c as u32 {
        0x0041..=0x024F | 0x1E00..=0x1EFF => "Latin",
        0x0370..=0x03FF => "Greek",
        0x0400..=0x04FF => "Russian",
        0x0590..=0x05FF => "Hebrew",
        0x0600..=0x06FF | 0x0750..=0x077F => "Arabic",
        0x0900..=0x097F => "Hindi",
        0x0980..=0x09FF => "Bengali",
        0x0A00..=0x0A7F => "Punjabi",
        0x0A80..=0x0AFF => "Gujarati",
        0x0B80..=0x0BFF => "Tamil",
        0x0C00..=0x0C7F => "Telugu",
        0x0C80..=0x0CFF => "Kannada",
        0x0D00..=0x0D7F => "Malayalam",
        0x0E00..=0x0E7F => "Thai",
        0x3040..=0x30FF => "Japanese",
        0x4E00..=0x9FFF | 0x3400..=0x4DBF => "Chinese",
        0xAC00..=0xD7AF | 0x1100..=0x11FF => "Korean",
        _ => "Other",
    }
}

/// Common short words of languages written in Latin letters.
const WORDS: &[(&str, &[&str])] = &[
    (
        "English",
        &[
            "the", "and", "you", "is", "are", "to", "of", "it", "in", "that", "for", "on", "with",
            "this", "my", "me", "we", "be", "have", "do", "what", "will", "can", "not", "your",
            "was", "at", "i", "a", "so", "but", "just", "how", "when", "ok", "yes", "no", "thanks",
        ],
    ),
    (
        "Spanish",
        &[
            "el", "la", "de", "que", "y", "en", "los", "las", "es", "por", "para", "con", "una",
            "un", "no", "se", "lo", "como", "pero", "muy", "gracias", "hola", "estoy", "mañana",
            "bien", "qué", "está", "tengo", "nos", "del",
        ],
    ),
    (
        "French",
        &[
            "le", "la", "les", "de", "des", "et", "est", "un", "une", "je", "tu", "vous", "nous",
            "pas", "que", "pour", "avec", "dans", "sur", "merci", "bonjour", "oui", "très",
            "c'est", "suis", "du", "au", "mais", "ce",
        ],
    ),
    (
        "Portuguese",
        &[
            "o", "a", "os", "as", "de", "que", "e", "do", "da", "em", "um", "uma", "não", "para",
            "com", "obrigado", "obrigada", "você", "estou", "muito", "bom", "dia", "isso", "tá",
            "mas", "por",
        ],
    ),
    (
        "German",
        &[
            "der", "die", "das", "und", "ist", "nicht", "ich", "du", "sie", "wir", "mit", "auf",
            "für", "ein", "eine", "zu", "danke", "ja", "nein", "gut", "auch", "aber", "noch",
            "bitte", "heute", "morgen",
        ],
    ),
    (
        "Italian",
        &[
            "il", "lo", "la", "di", "che", "e", "è", "non", "per", "un", "una", "sono", "con",
            "grazie", "ciao", "anche", "ma", "molto", "bene", "domani", "oggi", "sei", "della",
        ],
    ),
    (
        "Dutch",
        &[
            "de", "het", "een", "en", "van", "is", "niet", "ik", "je", "wij", "met", "op", "voor",
            "dank", "ja", "nee", "goed", "ook", "maar", "morgen", "vandaag",
        ],
    ),
    (
        "Indonesian",
        &[
            "yang", "dan", "di", "ini", "itu", "dengan", "untuk", "tidak", "saya", "kamu", "ada",
            "akan", "sudah", "belum", "terima", "kasih", "apa", "bisa", "juga", "besok",
        ],
    ),
];

/// A language written in Latin letters, from its common words. Needs
/// three or more words, so "ok" or "lol" never count.
fn latin_language(text: &str) -> Option<&'static str> {
    let words: Vec<String> = strip_noise(text)
        .split(|c: char| !(c.is_alphabetic() || c == '\''))
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();
    if words.len() < 3 {
        return None;
    }
    let score = |list: &[&str]| words.iter().filter(|w| list.contains(&w.as_str())).count();
    let english = score(WORDS[0].1);
    let (best, best_score) = WORDS[1..]
        .iter()
        .map(|(name, list)| (*name, score(list)))
        .max_by_key(|(_, s)| *s)?;
    if english >= best_score || english * 4 >= words.len() {
        return Some("English");
    }
    // At least two common words, and a fair share of the message.
    (best_score >= 2 && best_score * 5 >= words.len()).then_some(best)
}

/// Whether `text` seems to be in another language than `language`, the
/// person's ("English" when unset); the language found when it does.
pub fn foreign(text: &str, language: &str) -> Option<&'static str> {
    let wanted = if language.trim().is_empty() {
        "english".to_owned()
    } else {
        language.trim().to_lowercase()
    };
    let found = language_of(text)?;
    (found.to_lowercase() != wanted).then_some(found)
}

/// Messages found in another language, as (message id, language).
pub type Found = Vec<(String, &'static str)>;

/// A message in the translation popup.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub message: String,
    pub sender: String,
    pub at: i64,
    pub original: String,
}

/// The popup over a chat.
#[derive(Clone, Debug, PartialEq)]
pub struct Popup {
    pub account: String,
    pub chat: String,
    pub messages: Vec<Shown>,
}

/// Translations for this session, and the popup.
#[derive(Default)]
pub struct Translations {
    pub popup: Option<Popup>,
    /// Translations by (chat, message id).
    pub done: HashMap<(String, String), String>,
    /// Asked for and not back yet.
    pub pending: HashSet<(String, String)>,
    /// Chats whose offer the person closed this session.
    pub declined: HashSet<String>,
    /// The last request failed: what to say in the popup.
    pub error: Option<String>,
    /// The last scan of a chat: (chat, newest message id, message count,
    /// the messages found in another language with that language).
    pub scanned: Option<(String, String, usize, Found)>,
}

impl Translations {
    pub fn translation(&self, chat: &str, message: &str) -> Option<&str> {
        self.done
            .get(&(chat.to_owned(), message.to_owned()))
            .map(String::as_str)
    }

    /// Stores what came back for `messages` (by their order in the
    /// request).
    pub fn arrived(&mut self, chat: &str, messages: &[String], answer: &str) {
        let parsed = parse(answer, messages.len());
        for (index, message) in messages.iter().enumerate() {
            let key = (chat.to_owned(), message.clone());
            self.pending.remove(&key);
            if let Some(Some(text)) = parsed.get(index) {
                self.done.insert(key, text.clone());
            }
        }
    }

    pub fn failed(&mut self, chat: &str, messages: &[String], error: String) {
        for message in messages {
            self.pending.remove(&(chat.to_owned(), message.clone()));
        }
        self.error = Some(error);
    }
}

/// The numbered translations in an answer: `[1] …` lines, a line without
/// a number continuing the one before. With one message, the whole answer
/// is its translation.
pub fn parse(answer: &str, count: usize) -> Vec<Option<String>> {
    let mut out: Vec<Option<String>> = vec![None; count];
    let mut current: Option<usize> = None;
    for line in answer.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[')
            && let Some(end) = rest.find(']')
            && let Ok(n) = rest[..end].trim().parse::<usize>()
            && (1..=count).contains(&n)
        {
            let text = rest[end + 1..].trim().to_owned();
            out[n - 1] = Some(text);
            current = Some(n - 1);
            continue;
        }
        if let Some(index) = current
            && let Some(Some(text)) = out.get_mut(index)
        {
            text.push('\n');
            text.push_str(trimmed);
        }
    }
    if count == 1 && out[0].is_none() && !answer.trim().is_empty() {
        out[0] = Some(answer.trim().to_owned());
    }
    for text in out.iter_mut().flatten() {
        *text = text.trim().to_owned();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other_languages_are_found_and_english_is_not() {
        assert_eq!(
            foreign("¿Hola, cómo estás? Nos vemos mañana en la casa", ""),
            Some("Spanish")
        );
        assert_eq!(foreign("நாளை காலை நான் வருகிறேன்", ""), Some("Tamil"));
        assert_eq!(foreign("मैं कल सुबह आऊँगा", "English"), Some("Hindi"));
        assert_eq!(
            foreign("Merci pour le dîner, c'est très gentil", ""),
            Some("French")
        );
        assert_eq!(foreign("See you tomorrow at the station, thanks", ""), None);
        assert_eq!(foreign("Ok", ""), None);
        assert_eq!(foreign("lol 😂😂", ""), None);
        assert_eq!(foreign("https://example.com/a/b 12345 @Ada", ""), None);
        assert_eq!(foreign("Abc", ""), None);
    }

    #[test]
    fn the_persons_own_language_is_not_offered() {
        assert_eq!(
            foreign("¿Hola, cómo estás? Nos vemos mañana", "Spanish"),
            None
        );
        assert_eq!(
            foreign("See you tomorrow at the station", "Spanish"),
            Some("English")
        );
    }

    #[test]
    fn numbered_translations_are_read_back() {
        let found = parse("[1] Hello, how are you?\n[2] See you\ntomorrow", 2);
        assert_eq!(
            found,
            vec![
                Some("Hello, how are you?".into()),
                Some("See you\ntomorrow".into())
            ]
        );
        assert_eq!(parse("Just one", 1), vec![Some("Just one".into())]);
        assert_eq!(
            parse("[5] out of range", 1),
            vec![Some("[5] out of range".into())]
        );
    }

    #[test]
    fn arrived_translations_are_kept_and_reused() {
        let mut t = Translations::default();
        let messages = vec!["m1".to_owned(), "m2".to_owned()];
        t.pending.insert(("c".into(), "m1".into()));
        t.arrived("c", &messages, "[1] Hello\n[2] Bye");
        assert_eq!(t.translation("c", "m1"), Some("Hello"));
        assert_eq!(t.translation("c", "m2"), Some("Bye"));
        assert!(t.pending.is_empty());
    }
}
