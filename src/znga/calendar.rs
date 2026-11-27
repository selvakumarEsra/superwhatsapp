//! Plans in a chat as calendar files (REQ-ZNGA-022): the model finds the
//! events, this writes standard iCalendar text. No calendar account.

use serde::Deserialize;

use super::memory::json_array;
use super::promises::local_time;

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub title: String,
    /// Unix seconds.
    pub start: i64,
    pub end: i64,
    pub place: String,
    pub notes: String,
}

#[derive(Deserialize)]
struct Extracted {
    title: String,
    start: String,
    #[serde(default)]
    end: Option<String>,
    #[serde(default)]
    place: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

/// Events in an extraction answer:
/// `[{"title","start":"YYYY-MM-DDTHH:MM","end"?,"place"?,"notes"?}]`.
/// An event without an end lasts one hour (REQ-ZNGA-022.A3).
pub fn parse_events(answer: &str, zone: &jiff::tz::TimeZone) -> Vec<Event> {
    json_array::<Extracted>(answer)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let title = item.title.trim().to_owned();
            let start = local_time(&item.start, zone)?;
            if title.is_empty() {
                return None;
            }
            let end = item
                .end
                .as_deref()
                .and_then(|end| local_time(end, zone))
                .filter(|end| *end > start)
                .unwrap_or(start + 3600);
            Some(Event {
                title,
                start,
                end,
                place: item.place.unwrap_or_default().trim().to_owned(),
                notes: item.notes.unwrap_or_default().trim().to_owned(),
            })
        })
        .collect()
}

/// RFC 5545 text escaping (REQ-ZNGA-022.A2).
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(c),
        }
    }
    out
}

fn utc(seconds: i64) -> String {
    jiff::Timestamp::from_second(seconds)
        .map(|t| t.strftime("%Y%m%dT%H%M%SZ").to_string())
        .unwrap_or_default()
}

/// Folds a content line at 75 octets, as RFC 5545 asks.
fn fold(line: &str) -> String {
    let mut out = String::new();
    let mut width = 0;
    for c in line.chars() {
        let size = c.len_utf8();
        if width + size > 75 {
            out.push_str("\r\n ");
            width = 1;
        }
        out.push(c);
        width += size;
    }
    out
}

/// One event as a complete `.ics` file (REQ-ZNGA-022.A1).
pub fn ics(event: &Event, now: i64, uid: &str) -> String {
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_owned(),
        "VERSION:2.0".to_owned(),
        "PRODID:-//SuperWhatsapp//Znga//EN".to_owned(),
        "BEGIN:VEVENT".to_owned(),
        format!("UID:{uid}@znga.zapfast"),
        format!("DTSTAMP:{}", utc(now)),
        format!("DTSTART:{}", utc(event.start)),
        format!("DTEND:{}", utc(event.end)),
        format!("SUMMARY:{}", escape(&event.title)),
    ];
    if !event.place.is_empty() {
        lines.push(format!("LOCATION:{}", escape(&event.place)));
    }
    if !event.notes.is_empty() {
        lines.push(format!("DESCRIPTION:{}", escape(&event.notes)));
    }
    lines.push("END:VEVENT".to_owned());
    lines.push("END:VCALENDAR".to_owned());
    lines.iter().map(|line| fold(line) + "\r\n").collect()
}

/// A safe file name for an event.
pub fn file_name(event: &Event) -> String {
    let stem: String = event
        .title
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let stem: String = stem.chars().take(40).collect();
    format!(
        "{}.ics",
        if stem.is_empty() {
            "event".into()
        } else {
            stem
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zone() -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::UTC
    }

    #[test]
    fn an_event_becomes_a_valid_calendar_file() {
        let answer = r#"[{"title":"Dinner at Toit","start":"2026-11-21T19:00","place":"Toit, Indiranagar"}]"#;
        let events = parse_events(answer, &zone());
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.end - event.start, 3600);
        let text = ics(event, 0, "abc");
        assert!(text.starts_with("BEGIN:VCALENDAR\r\n"));
        assert_eq!(text.matches("BEGIN:VEVENT").count(), 1);
        assert!(text.contains("DTSTART:20261121T190000Z\r\n"));
        assert!(text.contains("DTEND:20261121T200000Z\r\n"));
        assert!(text.contains("SUMMARY:Dinner at Toit\r\n"));
        assert!(text.contains("LOCATION:Toit\\, Indiranagar\r\n"));
        assert!(text.ends_with("END:VCALENDAR\r\n"));
    }

    #[test]
    fn special_characters_are_escaped_and_long_lines_folded() {
        assert_eq!(escape("a,b;c\nd\\e"), "a\\,b\\;c\\nd\\\\e");
        let event = Event {
            title: "x".repeat(100),
            start: 0,
            end: 3600,
            place: String::new(),
            notes: String::new(),
        };
        let text = ics(&event, 0, "u");
        assert!(
            text.lines()
                .all(|line| line.trim_end_matches('\r').len() <= 75)
        );
    }

    #[test]
    fn events_without_a_title_or_time_are_skipped() {
        let answer = r#"[{"title":"","start":"2026-11-21T19:00"},{"title":"Call","start":"soon"}]"#;
        assert!(parse_events(answer, &zone()).is_empty());
        assert_eq!(
            file_name(&Event {
                title: "Dinner @ Toit!".into(),
                start: 0,
                end: 1,
                place: String::new(),
                notes: String::new()
            }),
            "Dinner-Toit.ics"
        );
    }
}
