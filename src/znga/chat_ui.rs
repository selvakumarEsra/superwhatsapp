//! The Znga chat in the egui app (specs/znga-chat.md): its row at the top
//! of the chat list, the chat itself in place of the conversation, and
//! Znga's tools in every chat's composer. The logic lives in
//! [`super::chat`]; this only draws and forwards clicks.

use egui::{Align, Color32, Layout, Pos2, RichText, Sense, Ui, Vec2, pos2, vec2};

use crate::app::App;
use crate::model::ChatFilter;
use crate::theme;
use crate::ui::widgets;

use super::chat;
use super::thread::Entry;

/// The Znga avatar: the sparkle on an accent circle.
fn avatar(ui: &Ui, center: Pos2, size: f32, palette: &theme::Palette) {
    ui.painter()
        .circle_filled(center, size / 2.0, palette.accent);
    super::ui::paint_sparkle(ui.painter(), center, size * 0.62, palette.on_accent);
}

/// The Znga row above the chats while Znga is on (REQ-ZCHAT-001.A1). It
/// stays out of Archived and the filters other than All.
pub fn row(app: &mut App, ui: &mut Ui) {
    if !chat::enabled(app) || app.show_archived || app.chat_filter != ChatFilter::All {
        return;
    }
    let palette = app.palette;
    let account = app.account().id.as_str().to_owned();
    let (unread, last) = app.znga.threads.get(&account).map_or((0, None), |t| {
        (t.unread, t.entries().last().map(|e| (e.at, e.text.clone())))
    });
    let selected = showing(app);
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), theme::ROW_HEIGHT),
        Sense::click(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, "Znga")
    });
    if ui.is_rect_visible(rect) {
        if selected {
            widgets::row_highlight(ui, &palette, rect, palette.surface_active);
        } else if response.hovered() {
            widgets::row_highlight(ui, &palette, rect, palette.surface_hover);
        }
        avatar(
            ui,
            pos2(rect.left() + 38.0, rect.center().y),
            48.0,
            &palette,
        );
        let left = rect.left() + 76.0;
        let right = rect.right() - 14.0;
        let stamp = last
            .as_ref()
            .map(|(at, _)| crate::util::chat_stamp(app.locale, *at))
            .unwrap_or_default();
        let stamp_color = if unread > 0 {
            palette.accent
        } else {
            palette.dim
        };
        let stamp = ui
            .painter()
            .layout_no_wrap(stamp, theme::regular(11.5), stamp_color);
        let top = rect.top() + 14.0;
        ui.painter().galley(
            pos2(right - stamp.size().x, top + 1.0),
            stamp.clone(),
            stamp_color,
        );
        let name_font = if unread > 0 {
            theme::semibold(14.5)
        } else {
            theme::medium(14.5)
        };
        ui.painter().text(
            pos2(left, top),
            egui::Align2::LEFT_TOP,
            "Znga",
            name_font,
            palette.text,
        );
        let line_y = rect.top() + 38.0;
        let mut text_right = right;
        if unread > 0 {
            let width = widgets::unread_indicator(
                ui,
                &palette,
                pos2(right - 10.0, line_y + 8.0),
                unread,
                false,
                false,
            );
            text_right -= width + 6.0;
        }
        let preview = last
            .map(|(_, text)| text.lines().next().unwrap_or("").to_owned())
            .unwrap_or_else(|| "Ask me anything about your chats.".into());
        let line = widgets::line(
            ui,
            &preview,
            theme::regular(13.5),
            palette.secondary,
            (text_right - left).max(0.0),
            1,
        );
        line.paint(ui, pos2(left, line_y), palette.secondary);
    }
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        chat::open_thread(app);
    }
}

/// Whether the Znga chat takes the conversation's place this frame.
pub fn showing(app: &App) -> bool {
    app.znga.chat_open
        && app.page == crate::model::Page::Chats
        && app.current_chat().is_none()
        && chat::enabled(app)
}

/// The Znga chat, drawn where the conversation goes (REQ-ZCHAT-001, 002,
/// 003).
pub fn show(app: &mut App, ui: &mut Ui) {
    let palette = app.palette;
    let account = app.account().id.as_str().to_owned();
    app.znga
        .threads
        .entry(account.clone())
        .or_default()
        .mark_read();
    ui.painter().rect_filled(ui.max_rect(), 0.0, palette.chat);
    egui::Panel::top("znga-chat-header")
        .show_separator_line(true)
        .frame(
            egui::Frame::new()
                .fill(palette.panel)
                .inner_margin(egui::Margin::symmetric(14, 8)),
        )
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
                avatar(ui, rect.center(), 40.0, &palette);
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Znga")
                            .font(theme::semibold(15.0))
                            .color(palette.text),
                    );
                    ui.label(
                        RichText::new("Your AI companion. This chat stays on this computer.")
                            .font(theme::regular(12.5))
                            .color(palette.secondary),
                    );
                });
            });
        });
    egui::Panel::bottom("znga-chat-composer")
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(Color32::TRANSPARENT)
                .inner_margin(egui::Margin {
                    left: 8,
                    right: 8,
                    top: 0,
                    bottom: 8,
                }),
        )
        .show(ui, |ui| composer(app, ui));
    let entries: Vec<Entry> = app
        .znga
        .threads
        .get(&account)
        .map(|t| t.entries().to_vec())
        .unwrap_or_default();
    let writing = chat::writing(app);
    let mut clicked: Option<(u64, usize, &'static str)> = None;
    let mut ask: Option<&'static str> = None;
    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(egui::Margin::symmetric(18, 10)))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("znga-chat")
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let width = ui.available_width();
                    if entries.is_empty() && writing.is_none() {
                        ask = empty(ui, &palette);
                    }
                    for entry in &entries {
                        if let Some(click) = bubble(ui, &palette, entry, width) {
                            clicked = Some(click);
                        }
                        ui.add_space(8.0);
                    }
                    if let Some(partial) = &writing
                        && writing_bubble(ui, &palette, partial, width)
                    {
                        chat::stop(app);
                    }
                });
        });
    if let Some((entry, index, action)) = clicked {
        chat::card_action(app, entry, index, action);
    }
    if let Some(text) = ask {
        chat::send_message(app, text);
    }
}

/// What to ask first, on an empty Znga chat.
fn empty(ui: &mut Ui, palette: &theme::Palette) -> Option<&'static str> {
    let mut picked = None;
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("Ask Znga about your chats, or tell it what to prepare.")
                .font(theme::regular(14.0))
                .color(palette.secondary),
        );
        ui.add_space(10.0);
        for text in [
            "What needs me today?",
            "Catch me up on my chats",
            "Who have I not talked to in a while?",
        ] {
            if ui.button(text).clicked() {
                picked = Some(text);
            }
        }
    });
    picked
}

fn bubble_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(10.0)
        .inner_margin(egui::Margin::symmetric(10, 7))
}

/// One message; returns the card button clicked, as (entry, card, action).
fn bubble(
    ui: &mut Ui,
    palette: &theme::Palette,
    entry: &Entry,
    width: f32,
) -> Option<(u64, usize, &'static str)> {
    let mut clicked = None;
    let max = (width * 0.75).clamp(240.0, 560.0);
    let layout = if entry.from_person {
        Layout::top_down(Align::Max)
    } else {
        Layout::top_down(Align::Min)
    };
    ui.with_layout(layout, |ui| {
        let fill = if entry.from_person {
            palette.bubble_out
        } else {
            palette.bubble_in
        };
        bubble_frame(fill).show(ui, |ui| {
            ui.set_max_width(max);
            if entry.unasked {
                ui.label(
                    RichText::new("✨ Znga noticed")
                        .font(theme::semibold(12.5))
                        .color(palette.accent),
                );
            }
            if !entry.text.is_empty() {
                ui.label(
                    RichText::new(&entry.text)
                        .font(theme::regular(14.0))
                        .color(palette.text),
                );
            }
            for (index, item) in entry.items.iter().enumerate() {
                if let Some(action) = card(ui, palette, item) {
                    clicked = Some((entry.id, index, action));
                }
            }
            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                ui.label(
                    RichText::new(clock(entry.at))
                        .font(theme::regular(11.0))
                        .color(palette.dim),
                );
            });
        });
    });
    clicked
}

/// One card with its buttons, or its outcome once acted on (REQ-ZCHAT-003).
fn card(ui: &mut Ui, palette: &theme::Palette, item: &super::thread::Item) -> Option<&'static str> {
    let (kind, title, body, meta) = item.card.view();
    let mut clicked = None;
    ui.add_space(4.0);
    egui::Frame::new()
        .fill(palette.surface)
        .stroke(egui::Stroke::new(1.0, palette.outline))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(kind_label(kind))
                    .font(theme::semibold(11.0))
                    .color(palette.accent),
            );
            if !title.is_empty() {
                ui.label(
                    RichText::new(title)
                        .font(theme::semibold(13.5))
                        .color(palette.text),
                );
            }
            if !body.is_empty() {
                ui.label(
                    RichText::new(body)
                        .font(theme::regular(13.5))
                        .color(palette.text),
                );
            }
            if !meta.is_empty() {
                ui.label(
                    RichText::new(meta)
                        .font(theme::regular(12.0))
                        .color(palette.secondary),
                );
            }
            ui.add_space(4.0);
            match &item.outcome {
                Some(outcome) => {
                    ui.label(
                        RichText::new(outcome)
                            .font(theme::medium(12.5))
                            .color(palette.secondary),
                    );
                }
                None => {
                    ui.horizontal_wrapped(|ui| {
                        for (n, (id, label)) in item.card.actions().iter().enumerate() {
                            let button = if n == 0 {
                                egui::Button::new(RichText::new(*label).color(palette.on_accent))
                                    .fill(palette.accent)
                            } else {
                                egui::Button::new(RichText::new(*label).color(palette.text))
                                    .frame(false)
                            };
                            if ui.add(button).clicked() {
                                clicked = Some(*id);
                            }
                        }
                    });
                }
            }
        });
    clicked
}

fn kind_label(kind: &str) -> &'static str {
    match kind {
        "needsYou" => "NEEDS YOU",
        "briefing" => "BRIEFING",
        "draft" => "DRAFT",
        "catchUp" => "CATCH-UP",
        "plan" => "PLAN",
        "voice" => "VOICE NOTE",
        "promise" => "PROMISE",
        "drift" => "CHECK IN",
        _ => "",
    }
}

/// Znga's answer as it is written, with Stop; returns whether Stop was
/// clicked (REQ-ZCHAT-002.A2).
fn writing_bubble(ui: &mut Ui, palette: &theme::Palette, partial: &str, width: f32) -> bool {
    let mut stop = false;
    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        bubble_frame(palette.bubble_in).show(ui, |ui| {
            ui.set_max_width((width * 0.75).clamp(240.0, 560.0));
            if partial.is_empty() {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new().size(14.0));
                    ui.label(RichText::new("Znga is thinking…").color(palette.secondary));
                });
            } else {
                ui.label(
                    RichText::new(partial)
                        .font(theme::regular(14.0))
                        .color(palette.text),
                );
            }
            stop = ui.small_button("Stop").clicked();
        });
    });
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(150));
    stop
}

fn clock(at: i64) -> String {
    jiff::Timestamp::from_second(at)
        .map(|t| {
            t.to_zoned(jiff::tz::TimeZone::system())
                .strftime("%H:%M")
                .to_string()
        })
        .unwrap_or_default()
}

/// The Znga chat's own field, drawn as every chat's composer: the rounded
/// field with "Type a message" and the round send button. What is written
/// goes to Znga only, never to WhatsApp.
fn composer(app: &mut App, ui: &mut Ui) {
    let palette = app.palette;
    let busy = chat::writing(app).is_some();
    let size = widgets::MESSAGE_TEXT_SIZE;
    // One text line plus padding, as the chat composer's row.
    let line = (ui.fonts_mut(|f| f.row_height(&theme::regular(size))) + 14.0)
        .round()
        .max(36.0);
    let id = egui::Id::new("znga-composer");
    let mut send = false;
    widgets::raised(
        ui,
        &palette,
        crate::ui::conversation::composer_pill(&palette),
        |ui| {
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), line),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.add_space(14.0);
                    let width =
                        (ui.available_width() - line - ui.spacing().item_spacing.x - 8.0).max(0.0);
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut app.znga.chat_draft)
                            .id(id)
                            .frame(egui::Frame::NONE)
                            .margin(egui::Margin::ZERO)
                            .hint_text(
                                RichText::new("Type a message")
                                    .color(palette.dim)
                                    .font(theme::regular(size)),
                            )
                            .font(theme::regular(size))
                            .text_color(palette.text)
                            .desired_rows(1)
                            .desired_width(width)
                            .return_key(Some(egui::KeyboardShortcut::new(
                                egui::Modifiers::SHIFT,
                                egui::Key::Enter,
                            ))),
                    );
                    if response.has_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
                    {
                        send = true;
                    }
                    let ready = !busy && !app.znga.chat_draft.trim().is_empty();
                    let (fill, hover, icon) = if ready {
                        (palette.accent, palette.accent_hover, palette.on_accent)
                    } else {
                        (palette.surface, palette.surface_hover, palette.dim)
                    };
                    if theme::circle_button(ui, theme::Icon::Send, line, fill, hover, icon, "Send")
                        .clicked()
                    {
                        send = true;
                    }
                },
            );
        },
    );
    if send && !busy {
        let text = std::mem::take(&mut app.znga.chat_draft);
        let text = text.trim();
        if !text.is_empty() {
            chat::send_message(app, text);
        }
        ui.memory_mut(|m| m.request_focus(id));
    }
}

// ---------------------------------------------------------------------
// Znga in every chat's composer (REQ-ZCHAT-004).

/// Znga's entries in the composer's plus menu; hidden where Znga may not
/// read the chat. Rewrites need text in the composer.
pub fn composer_menu(app: &mut App, ui: &mut Ui, chat_id: &str) {
    if !chat::tools_for(app, chat_id) {
        return;
    }
    let palette = app.palette;
    let has_text = !app.composer.trim().is_empty();
    ui.separator();
    ui.label(
        RichText::new("✨ Znga")
            .font(theme::semibold(12.0))
            .color(palette.accent),
    );
    let language = if app.znga.translate_to.trim().is_empty() {
        "English".to_owned()
    } else {
        app.znga.translate_to.trim().to_owned()
    };
    let translate = format!("Translate to {language}");
    let tools: [(&str, &str, bool); 8] = [
        ("suggest", "Suggest reply", true),
        ("catchUp", "Catch me up", true),
        ("plans", "Find plans", true),
        ("friendlier", "Friendlier", has_text),
        ("shorter", "Shorter", has_text),
        ("formal", "More formal", has_text),
        ("spelling", "Fix spelling", has_text),
        ("translate", &translate, has_text),
    ];
    for (id, label, on) in tools {
        if widgets::menu_item_enabled(ui, &palette, None, label, on) {
            let text = app.composer.clone();
            chat::tool(app, chat_id, id, &text, &language);
        }
    }
}

/// Above the composer: Znga's suggestion or rewrite with Use and Dismiss,
/// then Undo once used. Nothing is sent (REQ-ZCHAT-004.A3).
pub fn offer_bar(app: &mut App, ui: &mut Ui, chat_id: &str) {
    let palette = app.palette;
    if app
        .znga
        .composer_undo
        .as_ref()
        .is_some_and(|(chat, _)| chat != chat_id)
    {
        app.znga.composer_undo = None;
    }
    let offer = app
        .znga
        .offer
        .as_ref()
        .filter(|o| o.chat == chat_id)
        .map(|o| (o.text.clone(), o.rewrite));
    if let Some((text, rewrite)) = offer {
        bar(ui, &palette, |ui| {
            ui.label(RichText::new("✨").color(palette.accent));
            let label = if rewrite {
                "Znga's version: "
            } else {
                "Znga suggests: "
            };
            let shown: String = text.chars().take(160).collect();
            ui.label(
                RichText::new(format!("{label}{shown}"))
                    .font(theme::regular(13.0))
                    .color(palette.text),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.small_button("Dismiss").clicked() {
                    app.znga.offer = None;
                }
                if ui.small_button("Use").clicked() {
                    let before = std::mem::replace(&mut app.composer, text.clone());
                    app.znga.composer_undo = Some((chat_id.to_owned(), before));
                    app.znga.offer = None;
                    app.focus_composer = true;
                }
            });
        });
        return;
    }
    if let Some((_, before)) = app.znga.composer_undo.clone() {
        bar(ui, &palette, |ui| {
            ui.label(
                RichText::new("Znga changed your message. Nothing was sent.")
                    .font(theme::regular(13.0))
                    .color(palette.secondary),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.small_button("×").clicked() {
                    app.znga.composer_undo = None;
                }
                if ui.small_button("Undo").clicked() {
                    app.composer = before;
                    app.znga.composer_undo = None;
                    app.focus_composer = true;
                }
            });
        });
    }
}

fn bar(ui: &mut Ui, palette: &theme::Palette, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(palette.surface)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(add);
        });
    ui.add_space(4.0);
}

/// The rail button's target: the Znga chat with Znga on, else Settings at
/// the Znga section.
pub fn launch(app: &mut App) {
    if chat::enabled(app) {
        chat::open_thread(app);
    } else {
        // Opening Settings clears its search, so the search comes after.
        app.actions
            .push(crate::model::Action::Open(crate::model::Page::Settings));
        app.actions
            .push(crate::model::Action::SearchSettings("Znga".into()));
    }
}

/// Unread Znga messages on this account, for the rail's badge.
pub fn unread(app: &App) -> u32 {
    app.znga
        .threads
        .get(app.account().id.as_str())
        .map_or(0, |t| t.unread)
}
