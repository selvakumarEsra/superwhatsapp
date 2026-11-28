//! Znga's answers as something to read and act on (specs/znga-answers.md):
//! the messages an answer cites (REQ-ZANS-001), its light formatting
//! (REQ-ZANS-002), and the plain text Copy and Forward use (REQ-ZANS-003).
//!
//! Every message Znga is given for a question gets a number; the answer
//! cites `[3]` or `[3, 5]`, and the chat turns those into links. The
//! numbers and what they point at are saved with the answer.

use serde::{Deserialize, Serialize};

/// A message an answer may cite, by its number in that question.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ref {
    pub n: u32,
    pub chat: String,
    pub chat_title: String,
    /// The message id in its chat.
    pub message: String,
    pub sender: String,
    /// Unix seconds.
    pub at: i64,
    /// The message's first line, clipped.
    pub line: String,
}

impl Ref {
    pub fn new(
        n: u32,
        chat: &str,
        chat_title: &str,
        message: &str,
        sender: &str,
        at: i64,
        text: &str,
    ) -> Self {
        Self {
            n,
            chat: chat.to_owned(),
            chat_title: chat_title.to_owned(),
            message: message.to_owned(),
            sender: sender.to_owned(),
            at,
            line: first_line(text, 120),
        }
    }

    /// "chat, date", as a citation is written in plain text.
    pub fn label(&self) -> String {
        format!("{}, {}", self.chat_title, day(self.at))
    }
}

fn first_line(text: &str, max: usize) -> String {
    let line = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    if line.chars().count() <= max {
        line.to_owned()
    } else {
        let mut out: String = line.chars().take(max).collect();
        out.push('…');
        out
    }
}

/// A short local date: "12 Nov", with the year when it is not this one.
pub fn day(at: i64) -> String {
    let zone = jiff::tz::TimeZone::system();
    let Ok(stamp) = jiff::Timestamp::from_second(at) else {
        return String::new();
    };
    let zoned = stamp.to_zoned(zone.clone());
    let this_year = jiff::Timestamp::now().to_zoned(zone).year();
    if zoned.year() == this_year {
        zoned.strftime("%-d %b").to_string()
    } else {
        zoned.strftime("%-d %b %Y").to_string()
    }
}

/// A piece of a line of an answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Span {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    /// `[n]`: a cited message.
    Cite(u32),
}

/// A line of an answer, as it is laid out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: u8,
        spans: Vec<Span>,
    },
    Paragraph(Vec<Span>),
    Bullet {
        depth: u8,
        spans: Vec<Span>,
    },
    Numbered {
        depth: u8,
        number: String,
        spans: Vec<Span>,
    },
    Quote(Vec<Span>),
    Rule,
    /// A blank line between paragraphs.
    Gap,
}

fn depth_of(line: &str) -> u8 {
    let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
    (indent / 2).min(3) as u8
}

/// The answer's lines, with headings, lists, quotes and rules recognised.
pub fn blocks(text: &str) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            if !matches!(out.last(), None | Some(Block::Gap)) {
                out.push(Block::Gap);
            }
            continue;
        }
        if matches!(trimmed, "---" | "***" | "___") {
            out.push(Block::Rule);
            continue;
        }
        let hashes = trimmed.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&hashes) && trimmed[hashes..].starts_with(' ') {
            out.push(Block::Heading {
                level: hashes as u8,
                spans: spans(trimmed[hashes..].trim()),
            });
            continue;
        }
        if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("• "))
        {
            out.push(Block::Bullet {
                depth: depth_of(line),
                spans: spans(rest.trim()),
            });
            continue;
        }
        let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
        if digits > 0 && digits <= 3 {
            let after = &trimmed[digits..];
            if let Some(rest) = after
                .strip_prefix(". ")
                .or_else(|| after.strip_prefix(") "))
            {
                out.push(Block::Numbered {
                    depth: depth_of(line),
                    number: format!("{}.", &trimmed[..digits]),
                    spans: spans(rest.trim()),
                });
                continue;
            }
        }
        if let Some(rest) = trimmed
            .strip_prefix("> ")
            .or_else(|| trimmed.strip_prefix('>'))
        {
            out.push(Block::Quote(spans(rest.trim())));
            continue;
        }
        out.push(Block::Paragraph(spans(trimmed)));
    }
    while matches!(out.last(), Some(Block::Gap)) {
        out.pop();
    }
    out
}

/// The numbers in a citation that starts at `rest` (just after `[`), and
/// how many bytes it spans including the closing `]`: `[3]`, `[#3]`,
/// `[3, 5]`, `[3][5]` (as two).
fn citation(rest: &str) -> Option<(Vec<u32>, usize)> {
    let end = rest.find(']')?;
    let inside = &rest[..end];
    if inside.is_empty() || inside.len() > 24 {
        return None;
    }
    let mut numbers = Vec::new();
    for part in inside.split(',') {
        let part = part.trim().trim_start_matches('#').trim();
        if part.is_empty() || part.len() > 4 || !part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        numbers.push(part.parse().ok()?);
    }
    Some((numbers, end + 1))
}

fn push_text(out: &mut Vec<Span>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(Span::Text(last)) = out.last_mut() {
        last.push_str(text);
    } else {
        out.push(Span::Text(text.to_owned()));
    }
}

/// `*word*` or `_word_` at the start of `rest`: the inner text and what
/// follows. `snake_case` and a marker after a letter stay as written.
fn italic<'a>(rest: &'a str, before: &[Span]) -> Option<(&'a str, &'a str)> {
    let marker = rest.chars().next().filter(|c| matches!(c, '*' | '_'))?;
    let after_letter =
        matches!(before.last(), Some(Span::Text(t)) if t.ends_with(char::is_alphanumeric));
    if after_letter {
        return None;
    }
    let after = &rest[1..];
    if after.starts_with([' ', marker]) {
        return None;
    }
    let end = after.find(marker)?;
    if end == 0 || after[..end].ends_with(' ') {
        return None;
    }
    let tail = &after[end + 1..];
    if tail.starts_with(char::is_alphanumeric) {
        return None;
    }
    Some((&after[..end], tail))
}

/// Emphasis, code and citations in one line. A marker without its closing
/// partner is shown as written (REQ-ZANS-002.A3).
pub fn spans(line: &str) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::new();
    let mut rest = line;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("**")
            && let Some(end) = after.find("**")
            && end > 0
        {
            out.push(Span::Bold(after[..end].to_owned()));
            rest = &after[end + 2..];
            continue;
        }
        if let Some(after) = rest.strip_prefix('`')
            && let Some(end) = after.find('`')
            && end > 0
        {
            out.push(Span::Code(after[..end].to_owned()));
            rest = &after[end + 1..];
            continue;
        }
        if let Some((inner, after)) = italic(rest, &out) {
            out.push(Span::Italic(inner.to_owned()));
            rest = after;
            continue;
        }
        if let Some(after) = rest.strip_prefix('[')
            && let Some((numbers, used)) = citation(after)
        {
            for n in numbers {
                out.push(Span::Cite(n));
            }
            rest = &after[used..];
            continue;
        }
        let next = rest
            .char_indices()
            .skip(1)
            .find(|(_, c)| matches!(c, '*' | '_' | '`' | '['))
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        push_text(&mut out, &rest[..next]);
        rest = &rest[next..];
    }
    out
}

/// The messages an answer cites, each once, in the order first cited.
/// Numbers that were not given are left out (REQ-ZANS-001.A2).
pub fn cited<'a>(text: &str, refs: &'a [Ref]) -> Vec<&'a Ref> {
    let mut out: Vec<&Ref> = Vec::new();
    for block in blocks(text) {
        let spans = match block {
            Block::Heading { spans, .. }
            | Block::Paragraph(spans)
            | Block::Bullet { spans, .. }
            | Block::Numbered { spans, .. }
            | Block::Quote(spans) => spans,
            Block::Rule | Block::Gap => continue,
        };
        for span in spans {
            if let Span::Cite(n) = span
                && let Some(found) = refs.iter().find(|r| r.n == n)
                && !out.iter().any(|known| known.n == n)
            {
                out.push(found);
            }
        }
    }
    out
}

fn plain_spans(spans: &[Span], refs: &[Ref], out: &mut String) {
    let mut last_cite: Option<u32> = None;
    for span in spans {
        match span {
            Span::Text(text) | Span::Bold(text) | Span::Italic(text) | Span::Code(text) => {
                out.push_str(text);
                last_cite = None;
            }
            Span::Cite(n) => {
                if last_cite == Some(*n) {
                    continue;
                }
                last_cite = Some(*n);
                match refs.iter().find(|r| r.n == *n) {
                    Some(found) => {
                        if !out.ends_with(' ') && !out.is_empty() {
                            out.push(' ');
                        }
                        out.push_str(&format!("({})", found.label()));
                    }
                    None => out.push_str(&format!("[{n}]")),
                }
            }
        }
    }
}

/// The answer as plain text: no formatting markers, each citation written
/// as "(chat, date)" (REQ-ZANS-003.A2). For Copy, Forward and the person's
/// own chat, where links cannot go.
pub fn plain(text: &str, refs: &[Ref]) -> String {
    let mut out = String::new();
    for block in blocks(text) {
        let mut line = String::new();
        match &block {
            Block::Heading { spans, .. } | Block::Paragraph(spans) => {
                plain_spans(spans, refs, &mut line)
            }
            Block::Bullet { depth, spans } => {
                line.push_str(&"  ".repeat(*depth as usize));
                line.push_str("• ");
                plain_spans(spans, refs, &mut line);
            }
            Block::Numbered {
                depth,
                number,
                spans,
            } => {
                line.push_str(&"  ".repeat(*depth as usize));
                line.push_str(number);
                line.push(' ');
                plain_spans(spans, refs, &mut line);
            }
            Block::Quote(spans) => {
                line.push_str("> ");
                plain_spans(spans, refs, &mut line);
            }
            Block::Rule => line.push_str("———"),
            Block::Gap => {}
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(n: u32, chat: &str) -> Ref {
        Ref::new(
            n,
            chat,
            chat,
            &format!("m{n}"),
            "Ada",
            1_700_000_000,
            "ada@example.com\nsecond",
        )
    }

    #[test]
    fn formatting_is_recognised_and_markers_are_not_kept() {
        let text =
            "## Emails\n\n- **Ada** gave `ada@example.com` [1]\n- Bo [2, 3]\n\n1. First\n2. Second";
        let found = blocks(text);
        assert_eq!(
            found[0],
            Block::Heading {
                level: 2,
                spans: vec![Span::Text("Emails".into())]
            }
        );
        assert_eq!(found[1], Block::Gap);
        assert_eq!(
            found[2],
            Block::Bullet {
                depth: 0,
                spans: vec![
                    Span::Bold("Ada".into()),
                    Span::Text(" gave ".into()),
                    Span::Code("ada@example.com".into()),
                    Span::Text(" ".into()),
                    Span::Cite(1),
                ]
            }
        );
        assert_eq!(
            found[3],
            Block::Bullet {
                depth: 0,
                spans: vec![Span::Text("Bo ".into()), Span::Cite(2), Span::Cite(3)]
            }
        );
        assert!(matches!(&found[5], Block::Numbered { number, .. } if number == "1."));
        assert!(matches!(&found[6], Block::Numbered { number, .. } if number == "2."));
    }

    #[test]
    fn an_unclosed_marker_is_shown_as_written() {
        assert_eq!(
            spans("a ** b and *c"),
            vec![Span::Text("a ** b and *c".into())]
        );
        assert_eq!(
            spans("see [the link] here"),
            vec![Span::Text("see [the link] here".into())]
        );
        assert_eq!(
            spans("snake_case_name"),
            vec![Span::Text("snake_case_name".into())]
        );
        assert_eq!(
            spans("an *italic* word"),
            vec![
                Span::Text("an ".into()),
                Span::Italic("italic".into()),
                Span::Text(" word".into())
            ]
        );
    }

    #[test]
    fn citations_list_given_messages_once_in_order() {
        let refs = vec![r(1, "Ada"), r(2, "Team"), r(3, "Ada")];
        let text = "Ada [3] and the team [2]; Ada again [3] and nothing [9].";
        let found: Vec<u32> = cited(text, &refs).iter().map(|r| r.n).collect();
        assert_eq!(found, vec![3, 2]);
    }

    #[test]
    fn plain_text_writes_citations_as_chat_and_date() {
        let refs = vec![r(1, "Ada")];
        let text = "## Found\n- **Ada**: `ada@example.com` [1]\n- unknown [7]";
        let out = plain(text, &refs);
        let date = day(1_700_000_000);
        assert_eq!(
            out,
            format!("Found\n• Ada: ada@example.com (Ada, {date})\n• unknown [7]")
        );
    }

    #[test]
    fn a_ref_keeps_the_first_line_only() {
        assert_eq!(r(1, "Ada").line, "ada@example.com");
    }
}
