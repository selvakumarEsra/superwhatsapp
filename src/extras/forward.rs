//! A Favourites tab in the Forward dialog (REQ-EXTRAS-015): two chips under
//! the search field choose between every chat and the favourite chats. The
//! dialog itself is upstream's; it calls these from `extras hook` lines.

use crate::app::App;
use crate::model::Chat;
use crate::ui::widgets;

/// The chips' ids, for tests and the demo.
pub fn chip_id(favourites: bool) -> egui::Id {
    egui::Id::new(("forward-tab", favourites))
}

/// Height the chips take, with the space above them.
const CHIPS: f32 = 32.0;
/// Rows of chats that must still fit under the chips.
const ROWS: f32 = 2.0 * 52.0;

/// Draws the All / Favourites chips above a list that ends at `bottom`,
/// and returns whether only favourites are shown. In a window too short
/// for the chips and two rows, no chips are drawn and every chat shows.
/// The choice lasts until SuperWhatsapp quits (REQ-EXTRAS-015.A5).
pub fn tabs(app: &mut App, ui: &mut egui::Ui, bottom: f32) -> bool {
    if bottom - ui.cursor().top() < CHIPS + ROWS {
        return false;
    }
    let palette = app.palette;
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (favourites, label) in [(false, "All"), (true, "Favourites")] {
            let label = crate::i18n::gettext(app.locale, label);
            let selected = app.extras.forward_favourites == favourites;
            let chip = ui
                .push_id(chip_id(favourites), |ui| {
                    widgets::filter_chip(ui, &palette, &label, 0, selected)
                })
                .inner;
            ui.ctx()
                .data_mut(|data| data.insert_temp(chip_id(favourites), chip.rect));
            if chip.clicked() {
                app.extras.forward_favourites = favourites;
            }
        }
    });
    app.extras.forward_favourites
}

/// Keeps only the favourites, in the phone's order, when `favourites`.
/// `chats` already holds the chats that can receive a forward and match the
/// search (REQ-EXTRAS-015.A1, A2).
pub fn narrow(favourites: bool, chats: &mut Vec<Chat>) {
    if !favourites {
        return;
    }
    chats.retain(|chat| chat.favorite);
    chats.sort_by_key(|chat| chat.favorite_position);
}

/// What the list says when it is empty (REQ-EXTRAS-015.A4).
pub fn empty(app: &App, favourites: bool) -> &'static str {
    if !favourites {
        "No writable chats found"
    } else if app.forward_search.trim().is_empty() {
        "No favourite chats. Mark a chat as a favourite from its menu."
    } else {
        "No favourite chats match."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> (tempfile::TempDir, App) {
        let root = tempfile::tempdir().unwrap();
        let (mut app, _events) = App::headless(
            crate::paths::AppDirs::under(root.path()),
            crate::settings::Settings::default(),
        );
        crate::demo::populate(&mut app);
        crate::demo::apply_flags(&mut app, Some("forward"));
        (root, app)
    }

    fn frame(app: &mut App, ctx: &egui::Context, events: Vec<egui::Event>) {
        // Rows not drawn this frame must leave no rect behind.
        ctx.data_mut(|data| {
            for chat in &app.chats {
                data.remove::<egui::Rect>(egui::Id::new(("forward-recipient", &chat.id)));
            }
        });
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1180.0, 780.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                let ctx = ui.ctx().clone();
                app.background_frame(&ctx);
                app.frame_ui(ui);
            },
        );
        output.textures_delta.clear();
    }

    fn click(app: &mut App, ctx: &egui::Context, id: egui::Id) {
        let pos = ctx
            .data(|data| data.get_temp::<egui::Rect>(id))
            .expect("drawn")
            .center();
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        frame(app, ctx, Vec::new());
    }

    fn shown(app: &App, ctx: &egui::Context) -> Vec<String> {
        app.chats
            .iter()
            .filter(|chat| {
                ctx.data(|data| {
                    data.get_temp::<egui::Rect>(egui::Id::new(("forward-recipient", &chat.id)))
                })
                .is_some()
            })
            .map(|chat| chat.id.clone())
            .collect()
    }

    /// Marks the second and fourth writable chats as favourites, the fourth
    /// first in the phone's order.
    fn mark(app: &mut App) -> Vec<String> {
        let writable: Vec<String> = app
            .chats
            .iter()
            .filter(|chat| crate::ui::dialogs::forwardable(chat))
            .map(|chat| chat.id.clone())
            .collect();
        let picks = vec![writable[3].clone(), writable[1].clone()];
        for chat in &mut app.chats {
            chat.favorite = false;
            if let Some(position) = picks.iter().position(|id| *id == chat.id) {
                chat.favorite = true;
                chat.favorite_position = position as u32;
            }
        }
        picks
    }

    #[test]
    fn favourites_tab_lists_only_favourites_in_their_order() {
        let (_root, mut app) = app();
        let favourites = mark(&mut app);
        let ctx = egui::Context::default();
        app.attach(&ctx);
        frame(&mut app, &ctx, Vec::new());
        assert!(!app.extras.forward_favourites, "opens on All");
        assert!(shown(&app, &ctx).len() > favourites.len());

        click(&mut app, &ctx, chip_id(true));
        assert!(app.extras.forward_favourites);
        let mut chats: Vec<Chat> = app.chats.clone();
        narrow(app.extras.forward_favourites, &mut chats);
        let order: Vec<String> = chats.into_iter().map(|chat| chat.id).collect();
        assert_eq!(order, favourites, "the phone's order");
        let mut drawn = shown(&app, &ctx);
        drawn.sort();
        let mut expected = favourites.clone();
        expected.sort();
        assert_eq!(drawn, expected);

        // Closing and reopening keeps the tab (A5).
        app.dialog = None;
        frame(&mut app, &ctx, Vec::new());
        crate::demo::apply_flags(&mut app, Some("forward"));
        frame(&mut app, &ctx, Vec::new());
        assert!(app.extras.forward_favourites);
        click(&mut app, &ctx, chip_id(false));
        assert!(shown(&app, &ctx).len() > favourites.len());
    }

    #[test]
    fn search_and_picks_carry_across_tabs() {
        let (_root, mut app) = app();
        let favourites = mark(&mut app);
        let ctx = egui::Context::default();
        app.attach(&ctx);
        frame(&mut app, &ctx, Vec::new());
        // Pick a chat that is not a favourite on All.
        let other = shown(&app, &ctx)
            .into_iter()
            .find(|id| !favourites.contains(id))
            .unwrap();
        click(&mut app, &ctx, egui::Id::new(("forward-recipient", &other)));
        assert_eq!(app.forward_recipients, std::slice::from_ref(&other));

        click(&mut app, &ctx, chip_id(true));
        click(
            &mut app,
            &ctx,
            egui::Id::new(("forward-recipient", &favourites[0])),
        );
        assert_eq!(
            app.forward_recipients,
            [other.clone(), favourites[0].clone()]
        );

        // A search narrows the favourites and survives switching (A2).
        let title = app.chat_title(app.chat(&favourites[1]).unwrap());
        app.forward_search = title.clone();
        frame(&mut app, &ctx, Vec::new());
        assert_eq!(shown(&app, &ctx), [favourites[1].clone()]);
        click(&mut app, &ctx, chip_id(false));
        assert_eq!(app.forward_search, title);
        assert!(shown(&app, &ctx).contains(&favourites[1]));
        assert_eq!(app.forward_recipients, [other, favourites[0].clone()]);
    }

    #[test]
    fn an_empty_favourites_tab_says_how_to_add_one() {
        let (_root, mut app) = app();
        for chat in &mut app.chats {
            chat.favorite = false;
        }
        assert_eq!(empty(&app, false), "No writable chats found");
        app.extras.forward_favourites = true;
        let mut chats = app.chats.clone();
        narrow(app.extras.forward_favourites, &mut chats);
        assert!(chats.is_empty());
        assert_eq!(
            empty(&app, true),
            "No favourite chats. Mark a chat as a favourite from its menu."
        );
        app.forward_search = "zzz".into();
        assert_eq!(empty(&app, true), "No favourite chats match.");
        // Too short for the chips: none are drawn and every chat shows.
        let ctx = egui::Context::default();
        app.attach(&ctx);
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(360.0, 240.0),
                )),
                ..Default::default()
            },
            |ui| {
                let ctx = ui.ctx().clone();
                app.background_frame(&ctx);
                app.frame_ui(ui);
            },
        );
        output.textures_delta.clear();
        assert!(
            ctx.data(|data| data.get_temp::<egui::Rect>(chip_id(true)))
                .is_none()
        );
        // Reviewing hides the chips (A6).
        app.forward_reviewing = true;
        app.forward_recipients = vec![app.chats[0].id.clone()];
        ctx.data_mut(|data| data.remove::<egui::Rect>(chip_id(true)));
        frame(&mut app, &ctx, Vec::new());
        assert!(
            ctx.data(|data| data.get_temp::<egui::Rect>(chip_id(true)))
                .is_none()
        );
    }
}
