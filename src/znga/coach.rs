//! The relationship coach (REQ-ZNGA-024): how a close chat's rhythm has
//! changed, from message times alone, and why it may be drifting.

/// What the archive says about one chat's rhythm.
#[derive(Clone, Debug, PartialEq)]
pub struct Rhythm {
    pub account: String,
    pub chat: String,
    pub chat_title: String,
    /// Usual gap between conversations, in seconds (median).
    pub usual_gap: i64,
    /// Silence so far, in seconds.
    pub silence: i64,
    /// Whether the person wrote the last message.
    pub last_from_me: bool,
    /// Share of conversations the person started, 0..=1.
    pub started_by_me: f32,
    pub drifting: bool,
}

/// Messages closer than this belong to the same conversation.
const SAME_CONVERSATION: i64 = 6 * 3600;
const MIN_SILENCE: i64 = 7 * 86_400;

/// Reads a chat's rhythm from its message times (oldest first) and who
/// sent each (REQ-ZNGA-024.A1).
pub fn assess(
    account: &str,
    chat: &str,
    chat_title: &str,
    messages: &[(i64, bool)],
    now: i64,
) -> Option<Rhythm> {
    let (last_at, last_from_me) = *messages.last()?;
    // Conversation starts: the first message, and every message after a
    // long enough pause.
    let mut starts: Vec<(i64, bool)> = vec![messages[0]];
    for pair in messages.windows(2) {
        if pair[1].0 - pair[0].0 >= SAME_CONVERSATION {
            starts.push(pair[1]);
        }
    }
    let mut gaps: Vec<i64> = starts.windows(2).map(|w| w[1].0 - w[0].0).collect();
    if gaps.len() < 2 {
        return None;
    }
    gaps.sort_unstable();
    let usual_gap = gaps[gaps.len() / 2].max(1);
    let silence = (now - last_at).max(0);
    let started_by_me =
        starts.iter().filter(|(_, mine)| *mine).count() as f32 / starts.len() as f32;
    Some(Rhythm {
        account: account.into(),
        chat: chat.into(),
        chat_title: chat_title.into(),
        usual_gap,
        silence,
        last_from_me,
        started_by_me,
        drifting: silence >= MIN_SILENCE && silence > 2 * usual_gap,
    })
}

fn days(seconds: i64) -> String {
    let days = (seconds as f64 / 86_400.0).round() as i64;
    match days {
        0 => "less than a day".into(),
        1 => "1 day".into(),
        n => format!("{n} days"),
    }
}

impl Rhythm {
    /// Why the chat looks like it is drifting (REQ-ZNGA-024.A2).
    pub fn reason(&self) -> String {
        let last = if self.last_from_me {
            "You wrote last".to_owned()
        } else {
            format!("{} wrote last", self.chat_title)
        };
        let starter = if self.started_by_me >= 0.65 {
            " You usually start your conversations."
        } else if self.started_by_me <= 0.35 {
            " They usually start your conversations."
        } else {
            ""
        };
        format!(
            "{last}, {} ago. You usually talk every {}.{starter}",
            days(self.silence),
            days(self.usual_gap)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400;

    fn weekly(weeks: i64, mine: bool) -> Vec<(i64, bool)> {
        (0..weeks).map(|w| (w * 7 * DAY, mine)).collect()
    }

    #[test]
    fn a_long_silence_against_a_weekly_rhythm_is_drifting() {
        let messages = weekly(10, false);
        let last = messages.last().unwrap().0;
        let rhythm = assess("1", "ada", "Ada", &messages, last + 20 * DAY).unwrap();
        assert!(rhythm.drifting);
        assert_eq!(rhythm.usual_gap, 7 * DAY);
        let reason = rhythm.reason();
        assert!(reason.contains("Ada wrote last, 20 days ago"), "{reason}");
        assert!(reason.contains("every 7 days"), "{reason}");
        assert!(reason.contains("They usually start"), "{reason}");
    }

    #[test]
    fn a_short_or_normal_silence_is_not_drifting() {
        let messages = weekly(10, true);
        let last = messages.last().unwrap().0;
        assert!(
            !assess("1", "a", "A", &messages, last + 10 * DAY)
                .unwrap()
                .drifting
        );
        // Monthly friends: 40 days is normal-ish, under twice the gap.
        let monthly: Vec<(i64, bool)> = (0..6).map(|m| (m * 30 * DAY, true)).collect();
        let last = monthly.last().unwrap().0;
        assert!(
            !assess("1", "a", "A", &monthly, last + 40 * DAY)
                .unwrap()
                .drifting
        );
        // Too little history says nothing.
        assert!(assess("1", "a", "A", &[(0, true), (DAY, false)], 100 * DAY).is_none());
    }
}
