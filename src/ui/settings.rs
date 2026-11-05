use super::{theme::GOLD, widgets::action_button};
use crate::app::Launcher;
use crate::{config::catalog::Category, game::Settings, paths::log_path};
use eframe::egui::{self, Color32, RichText, Vec2};
use std::path::{Path, PathBuf};
pub(crate) fn show(app: &mut Launcher, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    let running = app.child.is_some() || app.install.is_some();
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(28, 16))
        .show(ui, |ui| {
            ui.heading("Settings");
            if running {
                ui.colored_label(
                    GOLD,
                    if app.install.is_some() {
                        "Wait for installation to finish before changing settings."
                    } else {
                        "Close the game to change settings."
                    },
                );
            }
            ui.horizontal_wrapped(|ui| {
                for category in Category::ALL {
                    if category == Category::Other
                        && !app.config.as_ref().is_ok_and(|c| {
                            c.extra.iter().any(|o| {
                                crate::config::catalog::category(o) == Category::Other
                                    && o.section != "GameMode"
                            })
                        })
                    {
                        continue;
                    }
                    if ui
                        .selectable_label(app.category == category, category.title())
                        .clicked()
                    {
                        if app.category != category {
                            egui::Popup::close_all(ui.ctx());
                        }
                        app.category = category;
                    }
                }
            });
            ui.add_space(8.0);
            let mut reload = false;
            egui::ScrollArea::vertical()
                .id_salt(app.category)
                .auto_shrink([false, false])
                .max_height((ui.available_height() - 82.0).max(100.0))
                .show(ui, |ui| {
                    ui.add_enabled_ui(!running, |ui| {
                        match &mut app.config {
                            Ok(config) => {
                                egui::Frame::new()
                                    .fill(Color32::from_rgb(22, 23, 28))
                                    .corner_radius(10)
                                    .inner_margin(18.0)
                                    .show(ui, |ui| {
                                        ui.set_min_width((ui.available_width() - 2.0).max(0.0));
                                        super::options::game_options(
                                            ui,
                                            config,
                                            app.category,
                                            &mut app.settings.skip_intro,
                                        );
                                    });
                            }
                            Err(error) => {
                                ui.colored_label(GOLD, error);
                            }
                        }
                        ui.add_space(16.0);
                        egui::CollapsingHeader::new("Advanced").show(ui, |ui| {
                            ui.label("DevilutionX executable");
                            ui.add(
                                egui::TextEdit::singleline(&mut app.settings.executable)
                                    .desired_width(f32::INFINITY),
                            );
                            ui.label("Game data and saves folder");
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut app.settings.data)
                                        .desired_width(f32::INFINITY),
                                )
                                .lost_focus()
                                && app.config.as_ref().is_ok_and(|c| {
                                    c.path != Path::new(&app.settings.data).join("diablo.ini")
                                })
                            {
                                reload = true;
                            }
                            if ui.button("Find installation").clicked() {
                                let detected = Settings::detected();
                                app.settings.executable = detected.executable;
                                app.settings.data = detected.data;
                                reload = true;
                            }
                            if let Ok(config) = &mut app.config {
                                egui::CollapsingHeader::new("Expansion and demo options").show(
                                    ui,
                                    |ui| {
                                        super::options::expansion_options(
                                            ui,
                                            config,
                                            app.settings.skip_intro,
                                        );
                                    },
                                );
                            }
                            if let Err(error) = app.settings.validate() {
                                ui.colored_label(GOLD, error);
                            }
                        });
                    });
                    egui::CollapsingHeader::new("Files and troubleshooting").show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Data and saves").clicked() {
                                app.open(&PathBuf::from(&app.settings.data));
                            }
                            if ui.button("diablo.ini").clicked() {
                                app.open(&Path::new(&app.settings.data).join("diablo.ini"));
                            }
                            if ui.button("Game log").clicked() {
                                app.open(&log_path());
                            }
                        });
                        ui.hyperlink_to("Get the game files", "https://devilutionx.com/mpq");
                        if !app.logs.is_empty() {
                            ui.add(
                                egui::Label::new(RichText::new(&app.logs).monospace())
                                    .selectable(true),
                            );
                        }
                    });
                });
            ui.add_space(10.0);
            let mut save = false;
            ui.horizontal(|ui| {
                save =
                    action_button(ui, "Save changes", !running, Vec2::new(162.0, 36.0)).clicked();
                if ui
                    .add_enabled(!running, egui::Button::new("Reload"))
                    .clicked()
                {
                    reload = true;
                }
            });
            if reload {
                egui::Popup::close_all(ui.ctx());
                app.reload_config();
                app.report(Ok(()), "Settings reloaded.");
            }
            if save {
                let result = app.save_settings(frame);
                app.report(result, "Settings saved.");
            }
            if !app.message.is_empty() {
                ui.colored_label(
                    if app.error {
                        Color32::from_rgb(245, 139, 128)
                    } else {
                        GOLD
                    },
                    &app.message,
                );
            }
        });
}
