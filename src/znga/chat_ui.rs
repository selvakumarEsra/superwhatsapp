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
        (
            t.unread,
            t.entries().last().map(|e| {
                // The preview reads as the answer does, without its markers.
                let text = if e.from_person {
                    e.text.clone()
                } else {
                    super::answer::plain(&e.text, &e.refs)
                };
                (e.at, text)
            }),
        )
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
    // A source in a chat Znga may no longer read is not shown
    // (REQ-ZANS-001.A5).
    let entries: Vec<Entry> = entries
        .into_iter()
        .map(|mut entry| {
            entry.refs.retain(|r| chat::may_link(app, &r.chat));
            entry
        })
        .collect();
    let writing = chat::writing(app);
    let mut clicked: Option<Click> = None;
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
    match clicked {
        Some(Click::Card(entry, index, action)) => chat::card_action(app, entry, index, action),
        Some(Click::Open { chat: id, message }) => chat::open_message(app, &id, &message),
        Some(Click::OpenChat(id)) => chat::open_cited_chat(app, &id),
        Some(Click::Reply(entry, text)) => {
            app.znga.reply_to = Some((entry, text));
            ui.memory_mut(|m| m.request_focus(egui::Id::new("znga-composer")));
        }
        Some(Click::Forward(text)) => {
            app.znga.forward_filter.clear();
            app.znga.forwarding = Some(text);
        }
        None => {}
    }
    if let Some(text) = ask {
        chat::send_message(app, text);
    }
    forward_picker(app, ui);
}

/// What a click in the Znga chat asks for.
enum Click {
    /// A card's button: (entry, card, action).
    Card(u64, usize, &'static str),
    /// A cited message (REQ-ZANS-001).
    Open { chat: String, message: String },
    /// A cited message's chat.
    OpenChat(String),
    /// Reply to this answer, quoting its text (REQ-ZANS-003).
    Reply(u64, String),
    /// Forward this text to a chat the person picks.
    Forward(String),
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

/// One message; returns what was clicked in it.
fn bubble(ui: &mut Ui, palette: &theme::Palette, entry: &Entry, width: f32) -> Option<Click> {
    let mut clicked = None;
    let max = (width * 0.75).clamp(240.0, 560.0);
    let layout = if entry.from_person {
        Layout::top_down(Align::Max)
    } else {
        Layout::top_down(Align::Min)
    };
    let shown = ui.with_layout(layout, |ui| {
        let fill = if entry.from_person {
            palette.bubble_out
        } else {
            palette.bubble_in
        };
        bubble_frame(fill)
            .show(ui, |ui| {
                ui.set_max_width(max);
                if entry.unasked {
                    ui.label(
                        RichText::new("✨ Znga noticed")
                            .font(theme::semibold(12.5))
                            .color(palette.accent),
                    );
                }
                if !entry.text.is_empty() {
                    if entry.from_person {
                        // People's words are shown as written (REQ-ZANS-002).
                        ui.label(
                            RichText::new(&entry.text)
                                .font(theme::regular(14.0))
                                .color(palette.text),
                        );
                    } else if let Some(link) = formatted(ui, palette, &entry.text, &entry.refs) {
                        clicked = Some(link);
                    }
                }
                if !entry.from_person
                    && let Some(link) = sources(ui, palette, &entry.text, &entry.refs)
                {
                    clicked = Some(link);
                }
                for (index, item) in entry.items.iter().enumerate() {
                    if let Some(action) = card(ui, palette, item) {
                        clicked = Some(Click::Card(entry.id, index, action));
                    }
                }
                ui.with_layout(Layout::top_down(Align::Max), |ui| {
                    ui.label(
                        RichText::new(clock(entry.at))
                            .font(theme::regular(11.0))
                            .color(palette.dim),
                    );
                });
            })
            .response
            .rect
    });
    if !entry.text.is_empty()
        && let Some(action) = hover_actions(ui, palette, entry, shown.inner)
    {
        clicked = Some(action);
    }
    clicked
}

/// Copy, Reply and Forward beside a message while the pointer is on it or
/// one of them has the keyboard (REQ-ZANS-003). The person's own messages
/// offer Copy only.
fn hover_actions(
    ui: &mut Ui,
    palette: &theme::Palette,
    entry: &Entry,
    rect: egui::Rect,
) -> Option<Click> {
    let focus_id = egui::Id::new(("znga-actions", entry.id));
    let focused = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(focus_id))
        .unwrap_or(false);
    // The row floats over the bubble on a layer of its own, so the pointer
    // is checked against the bubble without regard to layers; only the part
    // of the bubble in view counts.
    let visible = rect.intersect(ui.clip_rect());
    let near = ui
        .ctx()
        .pointer_latest_pos()
        .is_some_and(|pos| visible.contains(pos))
        && visible.height() > 24.0;
    if !near && !focused {
        ui.ctx().data_mut(|d| d.insert_temp(focus_id, false));
        return None;
    }
    let labels: &[&str] = if entry.from_person {
        &["Copy"]
    } else {
        &["Copy", "Reply", "Forward"]
    };
    let anchor = pos2(visible.right() - 6.0, visible.top() + 6.0);
    let mut picked = None;
    let mut any_focus = false;
    egui::Area::new(egui::Id::new(("znga-actions-row", entry.id)))
        .order(egui::Order::Foreground)
        .pivot(egui::Align2::RIGHT_TOP)
        .fixed_pos(anchor)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(palette.panel)
                .stroke(egui::Stroke::new(1.0, palette.outline))
                .corner_radius(7.0)
                .inner_margin(egui::Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;
                        for label in labels {
                            let button = egui::Button::new(
                                RichText::new(*label)
                                    .font(theme::medium(11.5))
                                    .color(palette.secondary),
                            )
                            .frame(false)
                            .small();
                            let response = ui.add(button);
                            any_focus |= response.has_focus();
                            if response.clicked() {
                                picked = Some(*label);
                            }
                        }
                    });
                });
        });
    ui.ctx().data_mut(|d| d.insert_temp(focus_id, any_focus));
    let plain = if entry.from_person {
        entry.text.clone()
    } else {
        super::answer::plain(&entry.text, &entry.refs)
    };
    match picked? {
        "Copy" => {
            ui.ctx().copy_text(plain);
            None
        }
        "Reply" => Some(Click::Reply(entry.id, plain)),
        _ => Some(Click::Forward(plain)),
    }
}

/// Znga's answer with its headings, lists and emphasis drawn, never their
/// markers, and its citations as links (REQ-ZANS-001, REQ-ZANS-002).
fn formatted(
    ui: &mut Ui,
    palette: &theme::Palette,
    text: &str,
    refs: &[super::answer::Ref],
) -> Option<Click> {
    use super::answer::Block;
    let mut clicked = None;
    let blocks = super::answer::blocks(text);
    ui.spacing_mut().item_spacing.y = 3.0;
    for (index, block) in blocks.iter().enumerate() {
        let found = match block {
            Block::Heading { level, spans } => {
                if index > 0 {
                    ui.add_space(6.0);
                }
                let size = match level {
                    1 => 17.0,
                    2 => 15.5,
                    _ => 14.5,
                };
                spans_row(ui, palette, spans, refs, theme::semibold(size))
            }
            Block::Paragraph(spans) => spans_row(ui, palette, spans, refs, theme::regular(14.0)),
            Block::Bullet { depth, spans } => list_item(ui, palette, *depth, "•", spans, refs),
            Block::Numbered {
                depth,
                number,
                spans,
            } => list_item(ui, palette, *depth, number, spans, refs),
            Block::Quote(spans) => {
                let mut link = None;
                egui::Frame::new()
                    .inner_margin(egui::Margin {
                        left: 8,
                        right: 0,
                        top: 1,
                        bottom: 1,
                    })
                    .show(ui, |ui| {
                        let rect = ui.max_rect();
                        ui.painter().vline(
                            rect.left() - 6.0,
                            rect.y_range(),
                            egui::Stroke::new(2.0, palette.outline),
                        );
                        link = spans_row(ui, palette, spans, refs, theme::regular(13.5));
                    });
                link
            }
            Block::Rule => {
                ui.separator();
                None
            }
            Block::Gap => {
                ui.add_space(5.0);
                None
            }
        };
        if found.is_some() {
            clicked = found;
        }
    }
    clicked
}

fn list_item(
    ui: &mut Ui,
    palette: &theme::Palette,
    depth: u8,
    marker: &str,
    spans: &[super::answer::Span],
    refs: &[super::answer::Ref],
) -> Option<Click> {
    let mut clicked = None;
    ui.horizontal_top(|ui| {
        ui.add_space(4.0 + f32::from(depth) * 16.0);
        let width = if marker.len() > 2 { 24.0 } else { 16.0 };
        // The marker sits in a wrapped row as the text does, so the two
        // share a baseline.
        ui.allocate_ui_with_layout(vec2(width, 0.0), Layout::top_down(Align::Min), |ui| {
            ui.set_width(width);
            ui.horizontal_wrapped(|ui| {
                let (font, color) = if marker == "•" {
                    (theme::semibold(14.0), palette.text)
                } else {
                    (theme::regular(14.0), palette.secondary)
                };
                ui.label(RichText::new(marker).font(font).color(color));
            });
        });
        ui.vertical(|ui| {
            clicked = spans_row(ui, palette, spans, refs, theme::regular(14.0));
        });
    });
    clicked
}

/// One line of an answer: text runs wrap together, citations are links
/// naming the chat and the date.
fn spans_row(
    ui: &mut Ui,
    palette: &theme::Palette,
    spans: &[super::answer::Span],
    refs: &[super::answer::Ref],
    font: egui::FontId,
) -> Option<Click> {
    use super::answer::Span;
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let mut last: Option<u32> = None;
        for span in spans {
            let text = match span {
                Span::Text(text) => RichText::new(text).font(font.clone()).color(palette.text),
                Span::Bold(text) => RichText::new(text)
                    .font(egui::FontId::new(
                        font.size,
                        theme::semibold(font.size).family,
                    ))
                    .color(palette.text),
                Span::Italic(text) => RichText::new(text)
                    .font(font.clone())
                    .italics()
                    .color(palette.text),
                Span::Code(text) => RichText::new(text)
                    .font(egui::FontId::monospace(font.size - 1.0))
                    .background_color(palette.surface)
                    .color(palette.text),
                Span::Cite(n) => {
                    if last == Some(*n) {
                        continue;
                    }
                    last = Some(*n);
                    match refs.iter().find(|r| r.n == *n) {
                        Some(found) => {
                            ui.add_space(3.0);
                            let link = ui
                                .link(
                                    RichText::new(format!(
                                        "{} · {}",
                                        found.chat_title,
                                        super::answer::day(found.at)
                                    ))
                                    .font(theme::medium(font.size.min(14.0) - 1.5))
                                    .color(palette.accent),
                                )
                                .on_hover_text(format!("{}: {}", found.sender, found.line));
                            if link.clicked() {
                                clicked = Some(Click::Open {
                                    chat: found.chat.clone(),
                                    message: found.message.clone(),
                                });
                            }
                            ui.add_space(3.0);
                        }
                        None => {
                            ui.label(
                                RichText::new(format!("[{n}]"))
                                    .font(font.clone())
                                    .color(palette.dim),
                            );
                        }
                    }
                    continue;
                }
            };
            last = None;
            ui.label(text);
        }
    });
    clicked
}

/// The messages an answer cites, each once: chat, sender, date and first
/// line, each opening its message; the chat name opens the chat
/// (REQ-ZANS-001.A3).
fn sources(
    ui: &mut Ui,
    palette: &theme::Palette,
    text: &str,
    refs: &[super::answer::Ref],
) -> Option<Click> {
    let cited = super::answer::cited(text, refs);
    if cited.is_empty() {
        return None;
    }
    let mut clicked = None;
    ui.add_space(8.0);
    ui.label(
        RichText::new("SOURCES")
            .font(theme::semibold(11.0))
            .color(palette.secondary),
    );
    for source in cited {
        let frame = egui::Frame::new()
            .fill(palette.surface)
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(8, 5))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    if ui
                        .link(
                            RichText::new(&source.chat_title)
                                .font(theme::semibold(12.5))
                                .color(palette.accent),
                        )
                        .on_hover_text("Open the chat")
                        .clicked()
                    {
                        clicked = Some(Click::OpenChat(source.chat.clone()));
                    }
                    // A one-to-one chat is named after its sender.
                    let who = if source.sender == source.chat_title {
                        String::new()
                    } else {
                        format!("· {} ", source.sender)
                    };
                    ui.label(
                        RichText::new(format!("{who}· {}", super::answer::day(source.at)))
                            .font(theme::regular(12.0))
                            .color(palette.secondary),
                    );
                });
                ui.label(
                    RichText::new(&source.line)
                        .font(theme::regular(13.0))
                        .color(palette.text),
                );
            });
        let response = ui
            .interact(
                frame.response.rect,
                egui::Id::new(("znga-source", source.chat.as_str(), source.message.as_str())),
                Sense::click(),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("Go to the message");
        if response.hovered() {
            ui.painter().rect_stroke(
                frame.response.rect,
                6.0,
                egui::Stroke::new(1.0, palette.accent),
                egui::StrokeKind::Inside,
            );
        }
        if response.clicked() && clicked.is_none() {
            clicked = Some(Click::Open {
                chat: source.chat.clone(),
                message: source.message.clone(),
            });
        }
        ui.add_space(3.0);
    }
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
                // Drawn as it will be once done (REQ-ZANS-002); the links
                // come with the finished answer.
                formatted(ui, palette, partial, &[]);
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
    reply_quote(app, ui);
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

/// The answer being replied to, above the composer, with × to drop it
/// (REQ-ZANS-003.A3).
fn reply_quote(app: &mut App, ui: &mut Ui) {
    let palette = app.palette;
    let Some((_, text)) = app.znga.reply_to.clone() else {
        return;
    };
    let mut close = false;
    egui::Frame::new()
        .fill(palette.surface)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let rect = ui.max_rect();
            ui.painter().vline(
                rect.left() - 5.0,
                rect.y_range(),
                egui::Stroke::new(3.0, palette.accent),
            );
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width((ui.available_width() - 30.0).max(0.0));
                    ui.label(
                        RichText::new("Replying to Znga")
                            .font(theme::semibold(12.0))
                            .color(palette.accent),
                    );
                    let first: String = text
                        .lines()
                        .find(|l| !l.trim().is_empty())
                        .unwrap_or("")
                        .chars()
                        .take(140)
                        .collect();
                    ui.label(
                        RichText::new(first)
                            .font(theme::regular(12.5))
                            .color(palette.secondary),
                    );
                });
                close = ui
                    .small_button("×")
                    .on_hover_text("Don't reply to this")
                    .clicked();
            });
        });
    ui.add_space(4.0);
    if close {
        app.znga.reply_to = None;
    }
}

/// The chats an answer can be forwarded to, while one is being forwarded:
/// picking one opens it with the text offered above its composer; nothing
/// is sent (REQ-ZANS-003.A4).
fn forward_picker(app: &mut App, ui: &mut Ui) {
    let palette = app.palette;
    let Some(text) = app.znga.forwarding.clone() else {
        return;
    };
    let filter = app.znga.forward_filter.trim().to_lowercase();
    let chats: Vec<(String, String)> = app
        .account()
        .chats
        .iter()
        .filter(|c| chat::tools_for(app, &c.id))
        .map(|c| (c.id.clone(), app.chat_title(c)))
        .filter(|(_, title)| filter.is_empty() || title.to_lowercase().contains(&filter))
        .take(200)
        .collect();
    let mut open = true;
    let mut picked: Option<String> = None;
    egui::Window::new("Forward Znga's answer")
        .id(egui::Id::new("znga-forward"))
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .default_width(320.0)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .frame(egui::Frame::popup(ui.style()).fill(palette.panel))
        .show(ui.ctx(), |ui| {
            let preview: String = text.chars().take(120).collect();
            ui.label(
                RichText::new(preview)
                    .font(theme::regular(12.5))
                    .color(palette.secondary),
            );
            ui.add_space(6.0);
            let search = ui.add(
                egui::TextEdit::singleline(&mut app.znga.forward_filter)
                    .hint_text("Search chats")
                    .desired_width(f32::INFINITY),
            );
            if search.gained_focus() || !search.has_focus() && ui.memory(|m| m.focused().is_none())
            {
                search.request_focus();
            }
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    if chats.is_empty() {
                        ui.label(RichText::new("No chats match.").color(palette.dim));
                    }
                    for (id, title) in &chats {
                        let row = ui.add(
                            egui::Button::new(
                                RichText::new(title)
                                    .font(theme::regular(13.5))
                                    .color(palette.text),
                            )
                            .frame(false)
                            .min_size(vec2(ui.available_width(), 26.0)),
                        );
                        if row.clicked() {
                            picked = Some(id.clone());
                        }
                    }
                });
            ui.add_space(4.0);
            ui.label(
                RichText::new(
                    "The chat opens with the text ready. Nothing is sent until you send it.",
                )
                .font(theme::regular(11.5))
                .color(palette.dim),
            );
        });
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        open = false;
    }
    if let Some(chat_id) = picked {
        app.znga.forwarding = None;
        chat::forward(app, &chat_id, &text);
    } else if !open {
        app.znga.forwarding = None;
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
    translate_bar(app, ui, chat_id);
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

// ---------------------------------------------------------------------
// Translating others' messages (REQ-ZANS-004).

/// The toggle's label for a message: "Translate to English", "Show
/// original", or that it is on its way.
fn toggle_label(toggle: &super::translate::inline::Toggle) -> String {
    use super::translate::inline::Toggle;
    match toggle {
        Toggle::Translate(language) => format!("Translate to {language}"),
        Toggle::Translating => "Translating…".to_owned(),
        Toggle::Original => "Show original".to_owned(),
    }
}

/// "Translate with Znga", or "Show original" once translated, in the menu
/// of a message in another language, in a chat Znga may read
/// (REQ-ZANS-004).
pub fn message_menu(
    ui: &mut Ui,
    palette: &theme::Palette,
    chat_id: &str,
    message: &crate::model::Message,
) {
    use super::translate::inline::{self, Toggle};
    let Some(text) = super::hooks::readable_text(message) else {
        return;
    };
    let Some(toggle) = inline::toggle(chat_id, &message.id, text) else {
        return;
    };
    let label = match &toggle {
        Toggle::Translate(_) => "Translate with Znga".to_owned(),
        other => toggle_label(other),
    };
    let enabled = toggle != Toggle::Translating;
    if widgets::menu_item_enabled(ui, palette, None, &format!("🌐 {label}"), enabled) {
        inline::click(chat_id, &message.id);
        ui.ctx().request_repaint();
        ui.close();
    }
}

/// The translate toggle beside a bubble's React, Reply and Forward
/// buttons (REQ-ZANS-004.A1, A3): a pill reading "Translate to English",
/// then "Show original". `slot` is the next free button place, `outward`
/// the direction away from the bubble; `revealed` whether the bubble's
/// buttons show.
#[allow(clippy::too_many_arguments)]
pub fn translate_control(
    ui: &mut Ui,
    palette: &theme::Palette,
    chat_id: &str,
    message: &crate::model::Message,
    slot: egui::Rect,
    outward: f32,
    bounds: egui::Rect,
    revealed: bool,
) {
    use super::translate::inline::{self, Toggle};
    let Some(text) = super::hooks::readable_text(message) else {
        return;
    };
    let Some(toggle) = inline::toggle(chat_id, &message.id, text) else {
        return;
    };
    let label = toggle_label(&toggle);
    let font = theme::medium(12.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.clone(), font.clone(), palette.text);
    let height = slot.height();
    let width = galley.size().x + height + 10.0;
    let pill = if outward >= 0.0 {
        egui::Rect::from_min_size(slot.min, vec2(width, height))
    } else {
        egui::Rect::from_min_size(pos2(slot.max.x - width, slot.min.y), vec2(width, height))
    };
    // Where the words do not fit, the globe alone, with them as its tooltip.
    let compact = !bounds.contains_rect(pill);
    let rect = if compact { slot } else { pill };
    if !bounds.contains_rect(rect) {
        return;
    }
    let id = egui::Id::new(("znga-translate-toggle", chat_id, message.id.as_str()));
    // The pointer crossing from the bubble to the pill keeps it shown.
    let held = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let pointer = ui.input(|i| i.pointer.interact_pos());
    let over = pointer.is_some_and(|p| rect.expand(4.0).contains(p));
    let show = revealed || (held && over);
    ui.ctx().data_mut(|d| d.insert_temp(id, show));
    if !show {
        return;
    }
    let response = ui.interact(rect, id, Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        palette.surface_hover
    } else {
        palette.surface
    };
    ui.painter().rect_filled(rect, height / 2.0, fill);
    ui.painter().rect_stroke(
        rect,
        height / 2.0,
        egui::Stroke::new(1.0, palette.outline),
        egui::StrokeKind::Inside,
    );
    let color = if hovered {
        palette.text
    } else {
        palette.secondary
    };
    let icon = egui::Rect::from_min_size(rect.min, vec2(height, height));
    ui.painter().text(
        icon.center(),
        egui::Align2::CENTER_CENTER,
        "🌐",
        theme::regular(13.0),
        color,
    );
    if !compact {
        ui.painter().galley(
            pos2(icon.right() - 2.0, rect.center().y - galley.size().y / 2.0),
            galley,
            color,
        );
    }
    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(if compact {
            label.as_str()
        } else {
            "Znga translates it for you; nothing is sent to the chat"
        });
    if response.clicked() && toggle != Toggle::Translating {
        inline::click(chat_id, &message.id);
        ui.ctx().request_repaint();
    }
    if toggle == Toggle::Translating {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(200));
    }
}

/// Above the composer: an offer to translate the chat's recent messages
/// in another language, in place (REQ-ZANS-004).
fn translate_bar(app: &mut App, ui: &mut Ui, chat_id: &str) {
    use super::translate::inline;
    let palette = app.palette;
    if !chat::tools_for(app, chat_id) {
        return;
    }
    let found = super::hooks::translatable(app, chat_id);
    let waiting: Vec<(String, &'static str)> = found
        .into_iter()
        .filter(|(id, _)| !inline::is_on(chat_id, id))
        .collect();
    if waiting.is_empty()
        || app.znga.translations.declined.contains(chat_id)
        || app.znga.offer.as_ref().is_some_and(|o| o.chat == chat_id)
    {
        return;
    }
    let language = waiting
        .last()
        .map(|(_, l)| *l)
        .unwrap_or("another language");
    let mut translate = false;
    let mut close = false;
    bar(ui, &palette, |ui| {
        ui.label(RichText::new("✨").color(palette.accent));
        let said = if waiting.len() == 1 {
            format!("A message here looks like {language}.")
        } else {
            format!("{} messages here look like {language}.", waiting.len())
        };
        ui.label(
            RichText::new(said)
                .font(theme::regular(13.0))
                .color(palette.text),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            close = ui.small_button("×").on_hover_text("Not now").clicked();
            translate = ui
                .small_button("Translate")
                .on_hover_text(
                    "Znga translates them in place; only these messages are sent to its engine",
                )
                .clicked();
        });
    });
    if close {
        app.znga.translations.declined.insert(chat_id.to_owned());
    }
    if translate {
        let skip = waiting.len().saturating_sub(10);
        for (id, _) in &waiting[skip..] {
            inline::turn_on(chat_id, id);
        }
        ui.ctx().request_repaint();
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
