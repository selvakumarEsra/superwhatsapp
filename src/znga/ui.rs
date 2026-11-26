//! Znga's button on the rail and Settings › Znga (setup, memory, promises,
//! the Sentinel log). The chat itself is in [`super::chat_ui`].
//! Views only edit Znga's own fields and queue work through [`super::hooks`].

use egui::{Color32, Pos2, Rect, RichText, Sense, Ui, Vec2, vec2};

use crate::app::App;

use super::config::{Engine, chat_key};
use super::privacy::{ChatFacts, visible};
use super::promises::{Owes, State};
use super::{Purpose, keys};

/// Paints Znga's mark: a four-point sparkle with a small one beside it.
pub fn paint_sparkle(painter: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    let star = |c: Pos2, s: f32| {
        let w = s * 0.3;
        painter.add(egui::Shape::convex_polygon(
            vec![
                c + vec2(0.0, -s),
                c + vec2(w, 0.0),
                c + vec2(0.0, s),
                c + vec2(-w, 0.0),
            ],
            color,
            egui::Stroke::NONE,
        ));
        painter.add(egui::Shape::convex_polygon(
            vec![
                c + vec2(-s, 0.0),
                c + vec2(0.0, -w),
                c + vec2(s, 0.0),
                c + vec2(0.0, w),
            ],
            color,
            egui::Stroke::NONE,
        ));
    };
    let main = size * 0.36;
    star(center + vec2(-size * 0.08, size * 0.08), main);
    star(center + vec2(size * 0.3, -size * 0.3), size * 0.14);
}

/// The rail's row height and button size (`ui::rail`'s private constants).
const RAIL_CELL: f32 = 48.0;
const RAIL_PLATE: f32 = 36.0;

/// Where Znga's button sits: on the rail under Calls, or above Starred while
/// the chat list is hidden (its avatars fill the space under Calls). Read
/// from where the rail drew those buttons last frame, so the rail itself
/// stays untouched.
fn launcher_rect(ctx: &egui::Context, list_hidden: bool) -> Option<Rect> {
    use crate::ui::rail::{Destination, destination_id};
    let anchor = if list_hidden {
        ctx.read_response(destination_id(Destination::Starred))?
            .rect
            .center()
            - vec2(0.0, RAIL_CELL)
    } else {
        ctx.read_response(destination_id(Destination::Calls))?
            .rect
            .center()
            + vec2(0.0, RAIL_CELL)
    };
    Some(Rect::from_center_size(anchor, Vec2::splat(RAIL_PLATE)))
}

/// Znga's button on the rail, drawn in a layer of its own: a widget inside
/// upstream's rail would shift its widget ids (and so the accessibility
/// order), and a button over the conversation would cover messages.
fn launcher(app: &mut App, ctx: &egui::Context) {
    let Some(rect) = launcher_rect(ctx, !app.sidebar_visible) else {
        return;
    };
    let palette = app.palette;
    egui::Area::new(egui::Id::new("znga-launcher"))
        .fixed_pos(rect.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(rect.size(), Sense::click());
            let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
            let lit = super::chat_ui::showing(app);
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, lit, "Znga")
            });
            if ui.is_rect_visible(rect) {
                let hovered = response.hovered() || response.has_focus();
                if lit {
                    crate::ui::widgets::highlight_card(ui, &palette, rect, palette.surface_active);
                } else if hovered {
                    crate::ui::widgets::highlight_card(ui, &palette, rect, palette.surface_hover);
                }
                let tint = if app.znga.config.enabled {
                    palette.accent
                } else if lit || hovered {
                    palette.text
                } else {
                    palette.secondary
                };
                paint_sparkle(ui.painter(), rect.center(), 24.0, tint);
                let unread = super::chat_ui::unread(app);
                if unread > 0 {
                    crate::ui::widgets::unread_indicator(
                        ui,
                        &palette,
                        rect.right_top() + vec2(-3.0, 5.0),
                        unread,
                        false,
                        false,
                    );
                }
            }
            let hint = if app.znga.config.enabled {
                "Znga: your AI companion"
            } else {
                "Znga: set up your AI companion"
            };
            if response.on_hover_text(hint).clicked() {
                super::chat_ui::launch(app);
            }
        });
}

fn time_label(at: i64) -> String {
    jiff::Timestamp::from_second(at)
        .map(|t| {
            t.to_zoned(jiff::tz::TimeZone::system())
                .strftime("%a %H:%M")
                .to_string()
        })
        .unwrap_or_default()
}

fn card(ui: &mut Ui, palette: &crate::theme::Palette, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(palette.surface)
        .stroke(egui::Stroke::new(1.0, palette.outline))
        .corner_radius(10.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
    ui.add_space(6.0);
}

/// Znga's button on the rail. Called once a frame after the app's views.
/// Znga lives in its own chat and in Settings, never a window of its own
/// (specs/znga-chat.md).
pub fn show(app: &mut App, ctx: &egui::Context) {
    // Znga talks in its chat in this app (REQ-ZCHAT-001).
    app.znga.chat_mode = true;
    if app.app_lock.is_locked() || !app.is_linked() {
        return;
    }
    launcher(app, ctx);
}

/// Settings › Znga: the engine and everything Znga does, what it
/// remembers, open promises and the Sentinel log (REQ-ZCHAT-007).
pub fn settings(app: &mut App, ui: &mut Ui) {
    ui.add_space(6.0);
    setup(app, ui);
    if app.znga.config.enabled {
        speaking_first(app, ui);
        open_promises(app, ui);
        memory(app, ui);
    }
}

/// What Znga says on its own beyond the switches in Setup: digests,
/// reminders and quiet hours (REQ-ZCHAT-005).
fn speaking_first(app: &mut App, ui: &mut Ui) {
    let palette = app.palette;
    let mut changed = false;
    card(ui, &palette, |ui| {
        ui.label(RichText::new("Speaking first").strong());
        let config = &mut app.znga.config;
        changed |= ui
            .checkbox(&mut config.digests, "Morning and evening digest")
            .changed();
        ui.add_enabled_ui(config.digests, |ui| {
            ui.horizontal(|ui| {
                changed |= time_field(ui, "Morning", &mut config.digest_morning);
                changed |= time_field(ui, "Evening", &mut config.digest_evening);
            });
        });
        changed |= ui
            .checkbox(
                &mut config.reminders,
                "Reminders: promises, plans and friends drifting away",
            )
            .changed();
        changed |= ui
            .checkbox(
                &mut config.quiet_hours,
                "Quiet hours (Needs you still comes through)",
            )
            .changed();
        ui.add_enabled_ui(config.quiet_hours, |ui| {
            ui.horizontal(|ui| {
                changed |= time_field(ui, "From", &mut config.quiet_from);
                changed |= time_field(ui, "To", &mut config.quiet_to);
            });
        });
    });
    if changed {
        app.znga.mark_dirty();
    }
}

/// An HH:MM field; only a valid time is kept.
fn time_field(ui: &mut Ui, label: &str, value: &mut String) -> bool {
    ui.label(label);
    let id = ui.id().with(("znga-time", label));
    let mut text = ui
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| value.clone());
    let response = ui.add(egui::TextEdit::singleline(&mut text).desired_width(52.0));
    let mut changed = false;
    if response.changed() {
        if super::thread::parse_hhmm(text.trim()).is_some() {
            *value = text.trim().to_owned();
            changed = true;
        }
        ui.data_mut(|d| d.insert_temp(id, text));
    } else if !response.has_focus() {
        ui.data_mut(|d| d.remove::<String>(id));
    }
    changed
}

/// Open promises on this account, with Done and Not a promise.
fn open_promises(app: &mut App, ui: &mut Ui) {
    let palette = app.palette;
    let account = app.account().id.as_str().to_owned();
    let open: Vec<(usize, String, String, bool)> = app
        .znga
        .promises
        .all()
        .iter()
        .enumerate()
        .filter(|(_, p)| p.account == account && p.state == State::Open)
        .map(|(i, p)| (i, p.chat_title.clone(), p.text.clone(), p.owes == Owes::Me))
        .collect();
    let mut set = None;
    card(ui, &palette, |ui| {
        ui.label(RichText::new("Open promises").strong());
        if open.is_empty() {
            ui.label(RichText::new("None.").small().color(palette.secondary));
        }
        for (index, title, text, mine) in &open {
            ui.horizontal_wrapped(|ui| {
                let who = if title.is_empty() {
                    "Reminder".to_owned()
                } else if *mine {
                    format!("You promised {title}")
                } else {
                    format!("{title} is waiting on you")
                };
                ui.label(RichText::new(who).strong());
                ui.label(text);
                if ui.small_button("Done").clicked() {
                    set = Some((*index, State::Done));
                }
                if ui.small_button("Not a promise").clicked() {
                    set = Some((*index, State::Dismissed));
                }
            });
        }
    });
    if let Some((index, state)) = set {
        app.znga.promises.set_state(index, state);
    }
}

pub fn explain_edit(
    app: &App,
    ui: &mut Ui,
    item: &crate::extras::keep::Kept,
) -> Option<(String, String, String)> {
    if !app.znga.config.enabled {
        return None;
    }
    let after = item.replaced_by.as_ref()?;
    let key = format!("{}/{}/{}", item.chat, item.id, item.version);
    let palette = app.palette;
    match app.znga.explanations.get(&key) {
        Some(Ok(text)) => {
            ui.label(RichText::new(format!("✨ {text}")).color(palette.secondary));
            None
        }
        Some(Err(error)) => {
            ui.label(
                RichText::new(format!("✨ {error}"))
                    .small()
                    .color(palette.warning),
            );
            None
        }
        None if app
            .znga
            .waiting_for(|p| matches!(p, Purpose::ExplainEdit { key: k } if *k == key)) =>
        {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().size(12.0));
                ui.label(
                    RichText::new("Explaining…")
                        .small()
                        .color(palette.secondary),
                );
            });
            None
        }
        None => ui
            .small_button("✨ Explain")
            .on_hover_text("What this edit changed, in a sentence")
            .clicked()
            .then(|| {
                (
                    key,
                    crate::extras::keep::copy_text(&item.content),
                    crate::extras::keep::copy_text(after),
                )
            }),
    }
}

/// What Znga remembers about people, and the Sentinel's log
/// (REQ-ZNGA-019, REQ-ZNGA-017).
fn memory(app: &mut App, ui: &mut Ui) {
    let palette = app.palette;
    let account = app.account().id.as_str().to_owned();
    card(ui, &palette, |ui| {
        ui.label(RichText::new("What Znga remembers about people").strong());
        ui.label(
            RichText::new(
                "Learned from your twin chats, kept encrypted on this computer only. \
                 Delete anything; it will not come back.",
            )
            .small()
            .color(palette.secondary),
        );
    });
    let facts: Vec<super::memory::Fact> = app
        .znga
        .people
        .for_account(&account)
        .into_iter()
        .cloned()
        .collect();
    if facts.is_empty() {
        ui.label(
            RichText::new("Nothing yet. Mark close ones as Twin chats in Setup.")
                .color(palette.secondary),
        );
    }
    let mut forget = None;
    let mut last_title = String::new();
    for fact in &facts {
        if fact.chat_title != last_title {
            ui.add_space(4.0);
            ui.label(RichText::new(&fact.chat_title).strong());
            last_title = fact.chat_title.clone();
        }
        ui.horizontal(|ui| {
            ui.label(format!("• {}", fact.text));
            if ui.small_button("×").on_hover_text("Forget this").clicked() {
                forget = Some(fact.clone());
            }
        });
    }
    if let Some(fact) = forget {
        app.znga
            .people
            .forget(&fact.account, &fact.chat, &fact.text);
    }
    if let Some(chat) = app.open_chat.clone() {
        let title = app.chat_title_of(&chat);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.znga.chat_filter)
                    .hint_text(format!("Add a fact about {title}"))
                    .desired_width(220.0),
            );
            if ui.button("Add").clicked() {
                let text = std::mem::take(&mut app.znga.chat_filter);
                if !text.trim().is_empty() {
                    let now = crate::util::now();
                    app.znga
                        .people
                        .add(&account, &chat, &title, &[text.trim().to_owned()], now);
                }
            }
        });
    }
    ui.add_space(8.0);
    ui.collapsing(
        "Sentinel log: everything Znga did or was stopped from doing",
        |ui| {
            let mut shown = 0;
            for entry in app.znga.audit.entries().rev().take(100) {
                shown += 1;
                let (mark, color) = if entry.allowed {
                    ("✓", palette.secondary)
                } else {
                    ("✗", palette.danger)
                };
                ui.label(
                    RichText::new(format!(
                        "{mark} {} · {} · for {} · {}",
                        time_label(entry.at),
                        entry.action,
                        entry.origin.label(),
                        entry.reason
                    ))
                    .small()
                    .color(color),
                );
            }
            if shown == 0 {
                ui.label(
                    RichText::new("Nothing yet.")
                        .small()
                        .color(palette.secondary),
                );
            }
        },
    );
}

fn setup(app: &mut App, ui: &mut Ui) {
    let palette = app.palette;
    ui.spacing_mut().slider_width = 150.0;
    let mut changed = false;
    let mut engine_changed = false;
    card(ui, &palette, |ui| {
        changed |= ui
            .checkbox(
                &mut app.znga.config.enabled,
                RichText::new("Turn on Znga").strong(),
            )
            .changed();
        ui.label(
            RichText::new(
                "Znga reads messages as they arrive in chats you have not excluded. It \
                 drafts; you decide what is sent.",
            )
            .small()
            .color(palette.secondary),
        );
    });

    card(ui, &palette, |ui| {
        ui.label(RichText::new("Engine").strong());
        let config = &mut app.znga.config;
        egui::ComboBox::from_id_salt("znga-engine")
            .selected_text(config.engine.label())
            .width(ui.available_width() - 8.0)
            .show_ui(ui, |ui| {
                for engine in Engine::ALL {
                    if ui
                        .selectable_value(&mut config.engine, engine, engine.label())
                        .changed()
                    {
                        engine_changed = true;
                    }
                }
            });
        if engine_changed {
            config.model.clear();
            config.endpoint.clear();
            config.cloud_consent = false;
        }
        let engine = config.engine;
        egui::Grid::new("znga-engine-grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Model");
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut config.model)
                            .hint_text(engine.default_model()),
                    )
                    .changed();
                ui.end_row();
                ui.label(if engine == Engine::ClaudeCode {
                    "Program"
                } else {
                    "Address"
                });
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut config.endpoint)
                            .hint_text(engine.default_endpoint()),
                    )
                    .changed();
                ui.end_row();
            });
        match engine {
            Engine::Local => {
                ui.label(
                    RichText::new(
                        "Runs on this computer: install Ollama (ollama.com), then run \
                         `ollama pull llama3.2`. Nothing leaves your machine.",
                    )
                    .small()
                    .color(palette.secondary),
                );
            }
            Engine::ClaudeCode => {
                let found = super::provider::find_claude(&config.endpoint);
                ui.label(
                    RichText::new(match &found {
                        Some(path) => format!("Found Claude Code at {}", path.display()),
                        None => "Claude Code was not found. Install it, or enter the full path \
                                 to `claude` above."
                            .to_owned(),
                    })
                    .small()
                    .color(if found.is_some() {
                        palette.secondary
                    } else {
                        palette.warning
                    }),
                );
                ui.label(
                    RichText::new(
                        "Uses your installed Claude Code and its own sign-in (your saved \
                         login, or CLAUDE_CODE_OAUTH_TOKEN in its environment). SuperWhatsapp \
                         never reads or stores the token. Meant for your own use on this \
                         computer; requests count toward your Claude plan.",
                    )
                    .small()
                    .color(palette.secondary),
                );
            }
            _ => {}
        }
    });

    let engine = app.znga.config.engine;
    if engine.needs_key() {
        card(ui, &palette, |ui| {
            ui.label(RichText::new("API key").strong());
            match keys::source(engine) {
                Some(source) => ui.label(RichText::new(format!("Using a key {source}.")).small()),
                None => ui.label(RichText::new("No key yet.").small().color(palette.warning)),
            };
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.znga.key_input)
                        .password(true)
                        .hint_text("Paste your API key")
                        .desired_width(200.0),
                );
                if ui.button("Save").clicked() {
                    let key = std::mem::take(&mut app.znga.key_input);
                    match keys::store(engine, &key) {
                        Ok(()) => app
                            .znga
                            .set_status("Key saved in the system keychain.", false),
                        Err(error) => app.znga.set_status(error, true),
                    }
                    app.znga.refresh_key();
                }
                if ui.small_button("Forget").clicked() {
                    keys::forget(engine);
                    app.znga.refresh_key();
                }
            });
            if engine == Engine::Claude {
                ui.label(
                    RichText::new(
                        "Use an API key from console.anthropic.com. To use your Claude \
                         subscription instead, pick \"Claude Code on this computer\".",
                    )
                    .small()
                    .color(palette.secondary),
                );
            }
        });
    }

    if app.znga.config.is_cloud() {
        card(ui, &palette, |ui| {
            ui.label(RichText::new("Privacy").strong().color(palette.warning));
            ui.label(
                "With this engine, the text of recent messages from chats you have not \
                 excluded is sent to the provider to write briefings and drafts. Locked \
                 chats are never sent.",
            );
            changed |= ui
                .checkbox(
                    &mut app.znga.config.cloud_consent,
                    "I understand: let this engine read my messages",
                )
                .changed();
        });
    }

    ui.horizontal(|ui| {
        if ui.button("Test connection").clicked() {
            let prompt = super::provider::Prompt {
                system: "Reply with the single word: ready".into(),
                turns: vec![super::provider::Turn::user("Are you there?")],
                max_tokens: 1000,
                effort: super::provider::Effort::Low,
            };
            match app.znga.submit(prompt, Purpose::Test) {
                Ok(()) => app.znga.set_status("Testing…", false),
                Err(error) => app.znga.set_status(error, true),
            }
        }
    });
    ui.add_space(6.0);

    card(ui, &palette, |ui| {
        ui.label(RichText::new("Proactive").strong());
        let config = &mut app.znga.config;
        changed |= ui
            .checkbox(&mut config.alerts, "Alert me when a message needs me")
            .changed();
        changed |= ui
            .checkbox(&mut config.briefings, "Brief me after I've been away")
            .changed();
        ui.add_enabled_ui(config.briefings, |ui| {
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.briefing_min_messages, 1..=50)
                        .text("messages at least"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.briefing_interval_minutes, 5..=240)
                        .text("minutes apart"),
                )
                .changed();
        });
        changed |= ui
            .checkbox(
                &mut config.self_notes,
                "Also send briefings to my own chat (to read on my phone)",
            )
            .changed();
        if config.self_notes {
            ui.label(
                RichText::new("At most six a day, only to your own \"Message yourself\" chat.")
                    .small()
                    .color(palette.secondary),
            );
        }
        changed |= ui
            .checkbox(&mut config.include_groups, "Include group chats")
            .changed();
        let was = config.self_chat;
        changed |= ui
            .checkbox(
                &mut config.self_chat,
                "Chat with Znga in my own \"Message yourself\" chat",
            )
            .changed();
        if config.self_chat && !was {
            config.self_chat_since = crate::util::now();
        }
        if config.self_chat {
            ui.label(
                RichText::new(
                    "Every text you send yourself, from your phone or here, is a question \
                     for Znga; it answers in the same chat (at most 100 a day). It never \
                     writes to anyone else.",
                )
                .small()
                .color(palette.secondary),
            );
        }
        changed |= ui
            .checkbox(&mut config.promises, "Track promises in twin chats")
            .changed();
        changed |= ui
            .checkbox(
                &mut config.memory,
                "Remember facts about people in twin chats",
            )
            .changed();
    });

    card(ui, &palette, |ui| {
        ui.label(RichText::new("Voice notes").strong());
        let config = &mut app.znga.config;
        egui::Grid::new("znga-voice-grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Transcription");
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut config.voice_endpoint)
                            .hint_text(super::voice::DEFAULT_ENDPOINT),
                    )
                    .changed();
                ui.end_row();
                ui.label("Model");
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut config.voice_model)
                            .hint_text(super::voice::DEFAULT_MODEL),
                    )
                    .changed();
                ui.end_row();
            });
        let local = super::voice::is_local(&config.voice_endpoint());
        ui.label(
            RichText::new(if local {
                "Runs on this computer: start a whisper.cpp server with \
                 `--inference-path /v1/audio/transcriptions --port 8080`."
            } else {
                "This service receives the voice notes you summarise. It uses your OpenAI \
                 key and needs the privacy consent above."
            })
            .small()
            .color(if local {
                palette.secondary
            } else {
                palette.warning
            }),
        );
    });

    card(ui, &palette, |ui| {
        ui.label(RichText::new("Your twin").strong());
        let config = &mut app.znga.config;
        egui::Grid::new("znga-twin-grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Your name");
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut config.your_name)
                            .hint_text("From your profile"),
                    )
                    .changed();
                ui.end_row();
            });
        ui.label("How you write, in your words (optional)");
        changed |= ui
            .add(
                egui::TextEdit::multiline(&mut config.voice_notes)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY)
                    .hint_text("I keep it short, say \"haha\" a lot, and call my mom \"ma\""),
            )
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.checkin_days, 2..=30).text("days before a check-in"))
            .changed();
        let account = app.account().id.as_str().to_owned();
        let learned = app.znga.style_for(&account);
        ui.label(
            RichText::new(if learned.messages >= 3 {
                format!(
                    "Learned from {} of your messages: {}",
                    learned.messages,
                    learned.describe()
                )
            } else {
                "Znga learns your style from messages you send in chats you open.".to_owned()
            })
            .small()
            .color(palette.secondary),
        );
    });

    changed |= personas(app, ui);
    changed |= chat_choices(app, ui);

    if changed || engine_changed {
        app.znga.mark_dirty();
    }
    if engine_changed {
        app.znga.refresh_key();
    }
}

/// A persona per linked account (REQ-ZNGA-025).
fn personas(app: &mut App, ui: &mut Ui) -> bool {
    let palette = app.palette;
    if app.accounts.len() < 2 {
        return false;
    }
    let mut changed = false;
    let accounts: Vec<(String, String)> = app
        .accounts
        .iter()
        .map(|account| {
            let label = account
                .me_name
                .clone()
                .or_else(|| account.me.clone())
                .unwrap_or_else(|| account.id.as_str().to_owned());
            (account.id.as_str().to_owned(), label)
        })
        .collect();
    card(ui, &palette, |ui| {
        ui.label(RichText::new("Personas").strong());
        ui.label(
            RichText::new(
                "How Znga sounds and behaves on each linked account, so work and \
                 personal stay apart. Empty fields use the settings above.",
            )
            .small()
            .color(palette.secondary),
        );
        for (id, label) in accounts {
            ui.collapsing(label, |ui| {
                let persona = app.znga.config.personas.entry(id.clone()).or_default();
                egui::Grid::new(format!("znga-persona-{id}"))
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Name");
                        changed |= ui.text_edit_singleline(&mut persona.your_name).changed();
                        ui.end_row();
                        ui.label("Tone");
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut persona.tone)
                                    .hint_text("brief and professional"),
                            )
                            .changed();
                        ui.end_row();
                    });
                ui.label("How you write here");
                changed |= ui
                    .add(
                        egui::TextEdit::multiline(&mut persona.voice_notes)
                            .desired_rows(2)
                            .desired_width(f32::INFINITY),
                    )
                    .changed();
                changed |= ui
                    .checkbox(
                        &mut persona.briefings,
                        "Proactive briefings for this account",
                    )
                    .changed();
            });
        }
    });
    changed
}

/// Twin and Exclude switches for the active account's recent chats.
fn chat_choices(app: &mut App, ui: &mut Ui) -> bool {
    let palette = app.palette;
    let account = app.account().id.as_str().to_owned();
    let filter = crate::util::search_key(app.znga.chat_filter.trim());
    let mut chats: Vec<(String, String, i64, bool)> = app
        .account()
        .chats
        .iter()
        .filter(|chat| !chat.locked && chat.kind != crate::model::ChatKind::Broadcast)
        .map(|chat| {
            let facts = ChatFacts::of(&account, chat);
            (
                chat.id.clone(),
                app.chat_title(chat),
                chat.last_activity,
                visible(&app.znga.config, facts) || app.znga.config.is_excluded(&account, &chat.id),
            )
        })
        .filter(|(_, title, _, shown)| {
            *shown && (filter.is_empty() || crate::util::search_key(title).contains(&filter))
        })
        .collect();
    chats.sort_by_key(|(id, _, last, _)| (!app.znga.config.is_twin(&account, id), -last));
    chats.truncate(40);
    let mut changed = false;
    card(ui, &palette, |ui| {
        ui.label(RichText::new("Your people").strong());
        ui.label(
            RichText::new(
                "Twin: Znga drafts replies as messages arrive and suggests check-ins. \
                 Exclude: Znga never reads this chat.",
            )
            .small()
            .color(palette.secondary),
        );
        ui.add(egui::TextEdit::singleline(&mut app.znga.chat_filter).hint_text("Search chats"));
        egui::Grid::new("znga-chats")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                for (id, title, _, _) in &chats {
                    let key = chat_key(&account, id);
                    let config = &mut app.znga.config;
                    let mut twin = config.twin_chats.contains(&key);
                    let mut excluded = config.excluded_chats.contains(&key);
                    ui.label(title);
                    if ui.checkbox(&mut twin, "Twin").changed() {
                        changed = true;
                        if twin {
                            config.twin_chats.insert(key.clone());
                            config.excluded_chats.remove(&key);
                        } else {
                            config.twin_chats.remove(&key);
                        }
                    }
                    if ui.checkbox(&mut excluded, "Exclude").changed() {
                        changed = true;
                        if excluded {
                            config.excluded_chats.insert(key.clone());
                            config.twin_chats.remove(&key);
                        } else {
                            config.excluded_chats.remove(&key);
                        }
                    }
                    ui.end_row();
                }
            });
    });
    if changed {
        // Forget what an excluded chat said.
        let excluded: Vec<String> = app.znga.config.excluded_chats.iter().cloned().collect();
        for key in excluded {
            if let Some((account, chat)) = key.split_once('/') {
                app.znga.activity.forget_chat(account, chat);
                app.znga.drafts.clear_chat(account, chat);
            }
        }
    }
    changed
}
