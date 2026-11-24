//! Synthetic mini-app states for demo screenshots: `--demo-page miniapps`
//! (Goal of the day in the own chat, mid-day) and `miniapps-add` (the
//! install screen). Sample content only; nothing is stored or sent.

use std::collections::BTreeMap;

use super::ChatKind;
use super::goal::{Day, History, Sent, State};
use super::hooks::{self, Sheet};
use crate::app::App;

/// Applies a mini-apps demo flag; `false` when `part` is not one.
pub fn apply(app: &mut App, part: &str) -> bool {
    match part {
        "miniapps" => {
            app.miniapps.open = true;
            install_goal(app);
            true
        }
        "miniapps-add" => {
            app.miniapps.open = true;
            let account = app.account().id.as_str().to_owned();
            let Some(chat) = own_chat(app) else {
                return true;
            };
            app.miniapps.sheet = Some(Sheet {
                account,
                chat,
                chat_title: "your own chat".into(),
                kind: ChatKind::Own,
                app: "goal".into(),
            });
            true
        }
        _ => false,
    }
}

fn own_chat(app: &App) -> Option<String> {
    app.account()
        .me
        .as_deref()
        .and_then(crate::znga::proactive::own_chat)
}

fn install_goal(app: &mut App) {
    let Some(chat) = own_chat(app) else {
        return;
    };
    let zone = jiff::tz::TimeZone::system();
    let today = jiff::Timestamp::from_second(crate::util::now())
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH)
        .to_zoned(zone)
        .date();
    let mut history = History::default();
    for (back, goal, done) in [
        (3, "Send the quarterly numbers to Grace", true),
        (2, "Finish the engine notes", true),
        (1, "Book the Berlin meetup room", false),
    ] {
        let Ok(date) = today.checked_sub(jiff::Span::new().days(back)) else {
            continue;
        };
        let mut day = Day::new(date, goal, &[]);
        day.done = done;
        day.progress = if done { 100 } else { 40 };
        history.set(day);
    }
    let steps = [
        "Outline the three sections".to_owned(),
        "Draft the intro".to_owned(),
        "Send it to Ada for a read".to_owned(),
    ];
    let mut day = Day::new(today, "Write the Rust Berlin talk abstract", &steps);
    day.steps[0].1 = true;
    day.progress = 33;
    history.set(day);
    let state = State {
        sent: Sent {
            date: today.to_string(),
            count: 1,
            proposed: true,
            ..Sent::default()
        },
        notice: Some("Goal set. Tick steps here or tell me how it goes.".into()),
        ..State::default()
    };
    let mut data = BTreeMap::new();
    if let (Ok(state), Ok(history)) = (
        serde_json::to_string(&state),
        serde_json::to_string(&history),
    ) {
        data.insert("state".to_owned(), state);
        data.insert("history".to_owned(), history);
    }
    hooks::demo_install(app, &chat, data);
}
