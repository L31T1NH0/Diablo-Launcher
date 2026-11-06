use super::{Tab, theme::GOLD, widgets::split_action_button};
use crate::{app::Launcher, game::Game};
use eframe::egui::{self, Color32};
use std::time::Duration;
pub(crate) fn show(app: &mut Launcher, ui: &mut egui::Ui) {
    app.hero.show(ui, app.settings.game);

    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(32, 14))
        .show(ui, |ui| {
            let ready = app.settings.validate();
            let dirty = app.settings_dirty();
            let running = app.child.is_some();
            let installing = app.install.is_some();
            let (status, color) = if installing {
                ("Installing…", GOLD)
            } else if running {
                ("Game running", GOLD)
            } else if dirty {
                ("Unsaved settings", GOLD)
            } else if ready.is_ok() {
                ("Ready to play", Color32::from_rgb(133, 183, 150))
            } else {
                ("Ready to install", GOLD)
            };
            let compact = ui.available_width() < 560.0;
            if compact {
                ui.colored_label(color, status);
            }
            ui.horizontal(|ui| {
                if !compact {
                    ui.colored_label(color, status);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = if running {
                        "RUNNING"
                    } else if installing {
                        "INSTALLING"
                    } else if ready.is_ok() {
                        "PLAY"
                    } else {
                        "INSTALL"
                    };
                    let selected = app.settings.game.title();
                    if split_action_button(
                        ui,
                        label,
                        selected,
                        !running && !installing && !dirty,
                        !running && !installing && !dirty,
                        |ui| {
                            for game in Game::ALL {
                                if game == Game::Hellfire && !app.settings.has_data(game) {
                                    continue;
                                }
                                let title = if app.settings.has_data(game) {
                                    game.title().to_owned()
                                } else {
                                    format!("{} · Install", game.title())
                                };
                                if ui
                                    .selectable_value(&mut app.settings.game, game, title)
                                    .changed()
                                {
                                    ui.ctx().request_repaint();
                                }
                            }
                        },
                    )
                    .clicked()
                    {
                        if ready.is_ok() {
                            let result = app.launch();
                            app.report(result, "");
                        } else {
                            app.install_game = app.settings.game;
                            app.tab = Tab::Install;
                        }
                        ui.ctx().request_repaint_after(Duration::from_millis(500));
                    }
                });
            });
            if !app.message.is_empty() && app.error {
                ui.colored_label(Color32::from_rgb(245, 139, 128), &app.message);
            }
        });
}
