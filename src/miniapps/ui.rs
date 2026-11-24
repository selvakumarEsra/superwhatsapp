//! The Mini-apps window and its rail button (REQ-MINIAPP-002, 005). Its
//! own layer, like Znga's: nothing is drawn inside upstream panels.

use std::collections::BTreeMap;

use egui::{Color32, Rect, RichText, Sense, Stroke, Vec2, vec2};

use super::hooks::{self, Sheet};
use super::{Card, ChatKind, Control};
use crate::app::App;
use crate::ui::rail::{Destination, destination_id};

const RAIL_CELL: f32 = 48.0;
const RAIL_PLATE: f32 = 36.0;
/// Rail cells Znga's and the extras' buttons take before ours.
const BEFORE: f32 = 1.0 + if cfg!(feature = "extras") { 2.0 } else { 0.0 };

fn slot(ctx: &egui::Context, list_hidden: bool) -> Option<Rect> {
    let calls = ctx.read_response(destination_id(Destination::Calls))?.rect;
    let starred = ctx
        .read_response(destination_id(Destination::Starred))?
        .rect;
    let step = RAIL_CELL * (1.0 + BEFORE);
    let center = if list_hidden {
        starred.center() - vec2(0.0, step)
    } else {
        calls.center() + vec2(0.0, step)
    };
    let rect = Rect::from_center_size(center, Vec2::splat(RAIL_PLATE));
    let clear = if list_hidden {
        rect.top() > calls.bottom() + RAIL_CELL
    } else {
        rect.bottom() + 4.0 < starred.top()
    };
    clear.then_some(rect)
}

fn paint_target(painter: &egui::Painter, center: egui::Pos2, color: Color32) {
    painter.circle_stroke(center, 9.0, Stroke::new(1.6, color));
    painter.circle_stroke(center, 5.0, Stroke::new(1.6, color));
    painter.circle_filled(center, 1.8, color);
}

fn launcher(app: &mut App, ctx: &egui::Context) {
    let Some(rect) = slot(ctx, !app.sidebar_visible) else {
        return;
    };
    let palette = app.palette;
    let lit = app.miniapps.open;
    egui::Area::new(egui::Id::new("miniapps-launcher"))
        .fixed_pos(rect.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(rect.size(), Sense::click());
            let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    true,
                    lit,
                    "Mini-apps",
                )
            });
            if ui.is_rect_visible(rect) {
                let hovered = response.hovered() || response.has_focus();
                if lit {
                    crate::ui::widgets::highlight_card(ui, &palette, rect, palette.surface_active);
                } else if hovered {
                    crate::ui::widgets::highlight_card(ui, &palette, rect, palette.surface_hover);
                }
                let tint = if lit || hovered {
                    palette.text
                } else {
                    palette.secondary
                };
                paint_target(ui.painter(), rect.center(), tint);
            }
            if response.on_hover_text("Mini-apps").clicked() {
                app.miniapps.open = !app.miniapps.open;
            }
        });
}

pub fn show(app: &mut App, ctx: &egui::Context) {
    if app.app_lock.is_locked() || !app.is_linked() {
        return;
    }
    launcher(app, ctx);
    if !app.miniapps.open {
        return;
    }
    let mut open = true;
    let screen = ctx.content_rect();
    let width = 380.0_f32.min(screen.width() - 32.0).max(280.0);
    egui::Window::new(RichText::new("Mini-apps").strong())
        .id(egui::Id::new("miniapps-panel"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(width)
        .default_height((screen.height() - 140.0).clamp(320.0, 720.0))
        .default_pos(screen.right_top() + vec2(-width - 48.0, 72.0))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| panel(app, ui));
        });
    if !open {
        app.miniapps.open = false;
    }
}

/// Chats on the active account an app could be added to: the own chat and
/// the open chat.
fn candidates(app: &App) -> Vec<(String, String, ChatKind)> {
    let account = app.account();
    let id = account.id.as_str();
    let own = account
        .me
        .as_deref()
        .and_then(crate::znga::proactive::own_chat);
    let mut out = Vec::new();
    if let Some(own) = &own
        && hooks::usable(app, id, own)
    {
        out.push((own.clone(), "your own chat".to_owned(), ChatKind::Own));
    }
    if let Some(open) = app.open_chat.as_deref()
        && Some(open) != own.as_deref()
        && hooks::usable(app, id, open)
        && let Some(chat) = app.chat(open)
    {
        let kind = if chat.is_group() {
            ChatKind::Group
        } else {
            ChatKind::Direct
        };
        out.push((open.to_owned(), app.chat_title(chat), kind));
    }
    out
}

fn panel(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let mut enabled = app.miniapps.host.saved.enabled;
    if ui
        .checkbox(&mut enabled, "Mini-apps on")
        .on_hover_text("Off: no app hears anything or acts. Installs are kept.")
        .changed()
    {
        app.miniapps.host.set_enabled(enabled);
    }
    if let Some(notice) = app.miniapps.notice.clone() {
        ui.colored_label(palette.danger, notice);
    }
    if let Some(sheet) = app.miniapps.sheet.clone() {
        ui.separator();
        install_sheet(app, ui, &sheet);
        return;
    }
    ui.separator();
    let account = app.account().id.as_str().to_owned();
    let installs: Vec<_> = app.miniapps.host.installs_on(&account).cloned().collect();
    if installs.is_empty() {
        ui.label(RichText::new("No mini-apps yet.").color(palette.secondary));
    }
    for install in installs {
        let key = install.key();
        let Some(manifest) = app
            .miniapps
            .host
            .apps()
            .find(|manifest| manifest.id == install.app)
        else {
            continue;
        };
        let chat_title = if install.kind == ChatKind::Own {
            "your own chat".to_owned()
        } else {
            app.chat_title_of(&install.chat)
        };
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(manifest.marker()).strong());
            ui.label(RichText::new(format!("· {chat_title}")).color(palette.secondary));
        });
        if let Some(error) = &install.off {
            ui.colored_label(palette.danger, format!("Turned off: {error}"));
            if ui.button("Turn back on").clicked() {
                hooks::restart(app, &key);
            }
        } else {
            let cards = app.miniapps.host.cards(&key).to_vec();
            if cards.is_empty() {
                ui.label(RichText::new("Loading…").color(palette.secondary));
            }
            for shown in &cards {
                if let Some((control, fields)) = card(app, ui, &key, shown) {
                    hooks::click(app, &key, &shown.id, control, fields);
                }
            }
        }
        if ui
            .small_button("Remove and delete its data")
            .on_hover_text("Stops the app in this chat and deletes what it kept.")
            .clicked()
        {
            hooks::uninstall(app, &key);
        }
        ui.separator();
    }
    ui.add_space(6.0);
    ui.label(RichText::new("Add a mini-app").strong());
    let manifests: Vec<_> = app.miniapps.host.apps().collect();
    let mut offered = false;
    for (chat, chat_title, kind) in candidates(app) {
        for manifest in &manifests {
            if !manifest.kinds.contains(&kind)
                || app.miniapps.host.installed(&account, &chat, manifest.id)
            {
                continue;
            }
            offered = true;
            if ui
                .button(format!("{} to {chat_title}", manifest.marker()))
                .on_hover_text(manifest.description)
                .clicked()
            {
                app.miniapps.notice = None;
                app.miniapps.sheet = Some(Sheet {
                    account: account.clone(),
                    chat: chat.clone(),
                    chat_title: chat_title.clone(),
                    kind,
                    app: manifest.id.to_owned(),
                });
            }
        }
    }
    if !offered {
        ui.label(
            RichText::new("Nothing to add here. Open a chat to see apps for it.")
                .color(palette.secondary),
        );
    }
}

/// The permission screen shown before an install (REQ-MINIAPP-002.A2).
pub fn install_sheet(app: &mut App, ui: &mut egui::Ui, sheet: &Sheet) {
    let Some(manifest) = app
        .miniapps
        .host
        .apps()
        .find(|manifest| manifest.id == sheet.app)
    else {
        app.miniapps.sheet = None;
        return;
    };
    ui.label(
        RichText::new(format!(
            "Add {} to {}?",
            manifest.marker(),
            sheet.chat_title
        ))
        .strong(),
    );
    ui.label(manifest.description);
    ui.add_space(4.0);
    ui.label("It will be able to:");
    for permission in manifest.permissions {
        ui.label(format!("• {}", permission.describe()));
    }
    ui.label(
        RichText::new("Every action passes Znga's Sentinel and is logged.")
            .color(app.palette.secondary),
    );
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui.button("Add").clicked() {
            app.miniapps.sheet = None;
            hooks::install(app, sheet.clone());
        }
        if ui.button("Cancel").clicked() {
            app.miniapps.sheet = None;
        }
    });
}

/// Draws one card. Returns the control clicked and the card's field
/// values (REQ-MINIAPP-005.A2). Text is plain, never markup (A3).
pub fn card(
    app: &mut App,
    ui: &mut egui::Ui,
    key: &str,
    card: &Card,
) -> Option<(Control, BTreeMap<String, String>)> {
    let palette = app.palette;
    let mut clicked = None;
    egui::Frame::group(ui.style())
        .fill(palette.surface_hover)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if !card.title.is_empty() {
                ui.label(RichText::new(&card.title).strong());
            }
            for line in &card.lines {
                ui.add(egui::Label::new(line.as_str()).wrap());
            }
            if let Some(progress) = card.progress {
                ui.add(egui::ProgressBar::new(progress.clamp(0.0, 1.0)).show_percentage());
            }
            for (index, (step, done)) in card.steps.iter().enumerate() {
                let mut on = *done;
                if ui.checkbox(&mut on, step.as_str()).changed() {
                    clicked = Some(Control::Step(index, on));
                }
            }
            for field in &card.fields {
                let slot = (key.to_owned(), card.id.clone(), field.id.clone());
                let value = app
                    .miniapps
                    .fields
                    .entry(slot)
                    .or_insert_with(|| field.value.clone());
                ui.horizontal(|ui| {
                    ui.label(&field.label);
                    ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
                });
            }
            ui.horizontal_wrapped(|ui| {
                for button in card.shown_buttons() {
                    if ui.button(&button.label).clicked() {
                        clicked = Some(Control::Button(button.id.clone()));
                    }
                }
            });
        });
    let control = clicked?;
    let mut fields = BTreeMap::new();
    for field in &card.fields {
        let slot = (key.to_owned(), card.id.clone(), field.id.clone());
        let value = app.miniapps.fields.remove(&slot).unwrap_or_default();
        fields.insert(field.id.clone(), value);
    }
    Some((control, fields))
}
