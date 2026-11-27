//! Translating someone else's message in place (REQ-ZANS-004): a message
//! that looks like another language offers Translate; Znga's translation
//! shows in a popup beside it. Nothing is sent and the chat is unchanged.
//! Translations are kept in memory while the app runs, never on disk.

use std::collections::HashMap;

use crate::app::App;

use super::Purpose;

/// Which message: (account, chat, message id).
pub type Key = (String, String, String);

/// Where one message's translation is.
#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Pending,
    Done { text: String, from: String },
    Failed(String),
}

/// Every translation asked for since the app started, and the one popup
/// that is open.
#[derive(Debug, Default)]
pub struct Translations {
    pub states: HashMap<Key, State>,
    pub open: Option<Key>,
    /// Requests sent to the engine, for tests and the audit.
    pub requests: usize,
}

/// Common short words of English, to tell an English message apart.
const ENGLISH: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "is", "are", "was", "were", "be", "to", "of", "in", "on",
    "at", "for", "with", "it", "this", "that", "i", "you", "he", "she", "we", "they", "me", "my",
    "your", "our", "his", "her", "not", "no", "yes", "do", "does", "did", "have", "has", "will",
    "can", "what", "when", "where", "how", "why", "ok", "okay", "thanks", "thank", "please", "hi",
    "hello", "see", "go", "get", "just", "so", "if", "all", "there", "here", "now", "today",
    "tomorrow", "call", "let", "know", "good", "u", "im", "it's", "i'm", "don't",
];

/// Common short words of the Latin-script languages people chat in most,
/// so a message without accents is still recognised.
const OTHER: &[&str] = &[
    // Spanish, Portuguese
    "el",
    "la",
    "los",
    "las",
    "que",
    "de",
    "y",
    "es",
    "un",
    "una",
    "por",
    "para",
    "con",
    "muy",
    "pero",
    "como",
    "mañana",
    "hola",
    "gracias",
    "nos",
    "vemos",
    "não",
    "sim",
    "obrigado",
    "você",
    "eu",
    "está",
    "com",
    "uma",
    "também",
    // French
    "le",
    "les",
    "et",
    "est",
    "je",
    "tu",
    "vous",
    "nous",
    "pas",
    "oui",
    "merci",
    "bonjour",
    "avec",
    "pour",
    "dans",
    "c'est",
    "très",
    // German, Dutch
    "der",
    "die",
    "das",
    "und",
    "ist",
    "nicht",
    "ich",
    "du",
    "wir",
    "ja",
    "nein",
    "danke",
    "bitte",
    "morgen",
    "het",
    "een",
    "niet",
    "wat",
    "dank",
    // Italian
    "il",
    "di",
    "che",
    "sono",
    "ciao",
    "grazie",
    "domani",
    "perché",
    "anche",
    // Indonesian, Malay
    "dan",
    "yang",
    "tidak",
    "aku",
    "kamu",
    "saya",
    "terima",
    "kasih",
    "besok",
    "ini",
    "itu",
    // Turkish
    "ve",
    "bir",
    "bu",
    "evet",
    "hayır",
    "teşekkürler",
    "yarın",
    "ben",
    "sen",
];

/// Whether `text` looks like it is not in `target`. Only English can be
/// recognised on this computer; for any other target language every
/// message with words is offered, and Znga says if it was already there.
pub fn foreign(text: &str, target: &str) -> bool {
    let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() < 3 {
        return false;
    }
    let words: Vec<String> = text
        .split(|c: char| !(c.is_alphabetic() || c == '\''))
        .filter(|word| !word.is_empty())
        .map(|word| word.to_lowercase())
        .collect();
    if !target.trim().eq_ignore_ascii_case("english") {
        return words.len() >= 2;
    }
    // Scripts other than Latin (Arabic, Cyrillic, Devanagari, CJK, …).
    let other_script = letters.iter().filter(|c| **c > '\u{024F}').count();
    if other_script * 10 >= letters.len() * 3 {
        return true;
    }
    if words.len() < 2 {
        return false;
    }
    let english = words
        .iter()
        .filter(|w| ENGLISH.contains(&w.as_str()))
        .count();
    let other = words.iter().filter(|w| OTHER.contains(&w.as_str())).count();
    let accented = letters.iter().filter(|c| !c.is_ascii()).count();
    if english * 5 >= words.len() && english >= other {
        return false;
    }
    other > english || accented * 20 > letters.len() || text.contains(['¿', '¡'])
}

fn message_text(app: &App, chat: &str, message: &str) -> Option<String> {
    let message = app.conversations.get(chat)?.message(message)?;
    if message.from_me {
        return None;
    }
    use crate::model::Content;
    let text = match &message.content {
        Content::Text { text, .. } | Content::Interactive { text, .. } => Some(text.as_str()),
        Content::Image { caption, .. } | Content::Video { caption, .. } => caption.as_deref(),
        _ => None,
    }?;
    Some(text.to_owned()).filter(|text| !text.trim().is_empty())
}

/// The language translations go into: Znga's "Translate to" setting.
pub fn target(app: &App) -> String {
    let language = app.znga.translate_to.trim();
    if language.is_empty() {
        "English".to_owned()
    } else {
        language.to_owned()
    }
}

/// Whether Translate is offered on this message (REQ-ZANS-004.A2, A4):
/// Znga is on, may see the chat, and the text looks like another language.
pub fn offered(app: &App, chat: &str, message: &str) -> bool {
    let account = app.account().id.as_str().to_owned();
    super::chat::enabled(app)
        && super::learn::may_see(app, &account, chat)
        && message_text(app, chat, message).is_some_and(|text| foreign(&text, &target(app)))
}

/// Shows the translation of one message, asking Znga for it the first
/// time (REQ-ZANS-004.A1, A3). `again` retries a failed one.
pub fn request(app: &mut App, chat: &str, message: &str, again: bool) {
    let account = app.account().id.as_str().to_owned();
    let key: Key = (account.clone(), chat.to_owned(), message.to_owned());
    app.znga.translations.open = Some(key.clone());
    if !super::chat::enabled(app) {
        app.znga.translations.states.insert(
            key,
            State::Failed("Znga is off. Turn it on in Settings › Znga to translate.".into()),
        );
        return;
    }
    if !super::learn::may_see(app, &account, chat) {
        app.znga.translations.states.insert(
            key,
            State::Failed("Znga does not read this chat, so it cannot translate here.".into()),
        );
        return;
    }
    match app.znga.translations.states.get(&key) {
        Some(State::Done { .. } | State::Pending) => return,
        Some(State::Failed(_)) if !again => return,
        _ => {}
    }
    let Some(text) = message_text(app, chat, message) else {
        app.znga
            .translations
            .states
            .insert(key, State::Failed("There is no text to translate.".into()));
        return;
    };
    let prompt = super::prompt::translate_message(&text, &target(app));
    let purpose = Purpose::Translate {
        account,
        chat: chat.to_owned(),
        message: message.to_owned(),
    };
    match app.znga.submit(prompt, purpose) {
        Ok(()) => {
            app.znga.translations.requests += 1;
            app.znga.translations.states.insert(key, State::Pending);
        }
        Err(error) => {
            let error = super::thread::explain_error(&error, app.znga.config.engine);
            app.znga
                .translations
                .states
                .insert(key, State::Failed(error));
        }
    }
}

/// Closes the popup; the translation is kept for next time.
pub fn close(app: &mut App) {
    app.znga.translations.open = None;
}

impl Translations {
    /// The engine's answer for `key`.
    pub fn finish(&mut self, key: Key, result: Result<String, String>) {
        let state = match result {
            Ok(answer) => {
                let (from, text) = parse(&answer);
                if text.is_empty() {
                    State::Failed("Znga gave no translation. Try again.".into())
                } else {
                    State::Done { text, from }
                }
            }
            Err(error) => State::Failed(format!("Znga could not translate this: {error}")),
        };
        self.states.insert(key, state);
    }

    /// The open popup's message and state.
    pub fn shown(&self) -> Option<(&Key, &State)> {
        let key = self.open.as_ref()?;
        Some((key, self.states.get(key)?))
    }
}

/// "FROM: Spanish" on the first line, then the translation.
pub fn parse(answer: &str) -> (String, String) {
    let answer = answer.trim();
    let (first, rest) = answer.split_once('\n').unwrap_or((answer, ""));
    match first.trim().strip_prefix("FROM:") {
        Some(from) => (from.trim().to_owned(), rest.trim().to_owned()),
        None => (String::new(), answer.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_is_not_offered_and_other_languages_are() {
        for english in [
            "Are we still on for 5 tomorrow?",
            "ok thanks",
            "Can you call me when you get home",
            "Ravi",
            "👍",
        ] {
            assert!(!foreign(english, "English"), "{english}");
        }
        for other in [
            "¿Nos vemos mañana a las 5?",
            "Je suis en retard, désolé",
            "Ich komme morgen nicht",
            "Terima kasih banyak ya",
            "Привет, как дела?",
            "明天见",
            "कल मिलते हैं",
        ] {
            assert!(foreign(other, "English"), "{other}");
        }
        // Another target: every message with words may be offered.
        assert!(foreign("See you tomorrow", "Spanish"));
        assert!(!foreign("ok", "Spanish"));
    }

    #[test]
    fn the_answer_names_its_language() {
        assert_eq!(
            parse("FROM: Spanish\nShall we meet tomorrow at 5?"),
            ("Spanish".into(), "Shall we meet tomorrow at 5?".into())
        );
        assert_eq!(parse("Hello"), (String::new(), "Hello".into()));
    }

    fn app() -> (tempfile::TempDir, App) {
        use crate::model::{Chat, Content, Delivery, Message};
        let root = tempfile::tempdir().unwrap();
        let (mut app, _events) = App::headless(
            crate::paths::AppDirs::under(root.path()),
            crate::settings::Settings::default(),
        );
        app.znga.chat_mode = true;
        app.znga.config.enabled = true;
        app.znga.runner.fake = Some(Box::new(|_| {
            Ok("FROM: Spanish\nShall we meet tomorrow at 5?".to_owned())
        }));
        app.account_mut().chats = vec![
            Chat::new("ana@s.whatsapp.net".to_owned(), "Ana".to_owned()),
            Chat::new("kim@s.whatsapp.net".to_owned(), "Kim".to_owned()),
        ];
        let account = app.account().id.as_str().to_owned();
        app.znga
            .config
            .excluded_chats
            .insert(super::super::config::chat_key(
                &account,
                "kim@s.whatsapp.net",
            ));
        for (chat, id, text) in [
            ("ana@s.whatsapp.net", "es", "¿Nos vemos mañana a las 5?"),
            ("ana@s.whatsapp.net", "en", "See you at 5 then"),
            ("kim@s.whatsapp.net", "kim", "¿Nos vemos mañana a las 5?"),
        ] {
            app.conversations
                .entry(chat.to_owned())
                .or_default()
                .messages
                .push(Message {
                    id: id.into(),
                    chat: chat.into(),
                    sender: chat.into(),
                    sender_name: None,
                    from_me: false,
                    timestamp: 0,
                    content: Content::text(text),
                    status: Delivery::None,
                    delivered_at: None,
                    read_at: None,
                    quoted: None,
                    reactions: Vec::new(),
                    history_order: None,
                    edited: false,
                    mentions: Vec::new(),
                    forwarded: false,
                    thumbnail: None,
                });
        }
        (root, app)
    }

    #[test]
    fn a_translation_is_shown_once_and_reused() {
        let (_root, mut app) = app();
        assert!(offered(&app, "ana@s.whatsapp.net", "es"));
        assert!(!offered(&app, "ana@s.whatsapp.net", "en"));
        // A chat Znga may not see offers nothing (REQ-ZANS-004.A4).
        assert!(!offered(&app, "kim@s.whatsapp.net", "kim"));
        request(&mut app, "ana@s.whatsapp.net", "es", false);
        super::super::hooks::tick(&mut app);
        let (_, state) = app.znga.translations.shown().unwrap();
        assert_eq!(
            state,
            &State::Done {
                text: "Shall we meet tomorrow at 5?".into(),
                from: "Spanish".into()
            }
        );
        // Closed and opened again: no second request (REQ-ZANS-004.A3).
        close(&mut app);
        assert!(app.znga.translations.shown().is_none());
        request(&mut app, "ana@s.whatsapp.net", "es", false);
        assert_eq!(app.znga.translations.requests, 1);
        assert!(app.znga.translations.shown().is_some());
        // The conversation is unchanged and nothing was sent.
        assert_eq!(app.conversations["ana@s.whatsapp.net"].messages.len(), 2);
        assert!(
            app.actions
                .iter()
                .all(|a| !matches!(a, crate::model::Action::SendText { .. }))
        );
        // Excluded chat: says why instead of asking the engine.
        request(&mut app, "kim@s.whatsapp.net", "kim", false);
        assert!(matches!(
            app.znga.translations.shown(),
            Some((_, State::Failed(_)))
        ));
        assert_eq!(app.znga.translations.requests, 1);
        // Znga off: offered nowhere.
        app.znga.config.enabled = false;
        assert!(!offered(&app, "ana@s.whatsapp.net", "es"));
    }

    #[test]
    fn a_failed_translation_can_be_tried_again() {
        let (_root, mut app) = app();
        app.znga.runner.fake = Some(Box::new(|_| Err("offline".to_owned())));
        request(&mut app, "ana@s.whatsapp.net", "es", false);
        super::super::hooks::tick(&mut app);
        assert!(matches!(
            app.znga.translations.shown(),
            Some((_, State::Failed(_)))
        ));
        app.znga.runner.fake = Some(Box::new(|_| Ok("FROM: Spanish\nAt 5?".to_owned())));
        request(&mut app, "ana@s.whatsapp.net", "es", true);
        super::super::hooks::tick(&mut app);
        assert!(matches!(
            app.znga.translations.shown(),
            Some((_, State::Done { .. }))
        ));
        assert_eq!(app.znga.translations.requests, 2);
    }
}
