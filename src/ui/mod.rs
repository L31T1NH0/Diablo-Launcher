mod home;
mod install;
pub(crate) mod options;
mod settings;
pub(crate) mod theme;
mod widgets;
use crate::{app::Launcher, config::GameConfig, game::Game};
use eframe::egui::{self, Color32, Vec2};
use theme::BACKGROUND;
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Tab {
    Play,
    Settings,
    Install,
}
pub(crate) fn show(app: &mut Launcher, ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("navigation")
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(BACKGROUND)
                .inner_margin(egui::Margin::symmetric(24, 16)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (icon_slot, _) =
                    ui.allocate_exact_size(Vec2::new(28.0, 40.0), egui::Sense::hover());
                ui.painter().image(
                    app.logo.id(),
                    egui::Rect::from_center_size(icon_slot.center(), Vec2::splat(28.0)),
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                let height = if ui.available_width() > 480.0 {
                    40.0
                } else {
                    28.0
                };
                ui.image((
                    app.wordmark.id(),
                    Vec2::new(
                        height * app.wordmark.size_vec2().x / app.wordmark.size_vec2().y,
                        height,
                    ),
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(app.tab == Tab::Settings, "Settings")
                        .clicked()
                    {
                        if app.tab != Tab::Settings
                            && !app.config.as_ref().is_ok_and(GameConfig::dirty)
                        {
                            app.reload_config();
                        }
                        egui::Popup::close_all(ui.ctx());
                        app.tab = Tab::Settings;
                    }
                    if ui
                        .selectable_label(app.tab == Tab::Install, "Install")
                        .clicked()
                    {
                        egui::Popup::close_all(ui.ctx());
                        app.tab = Tab::Install;
                        if app.settings.has_data(Game::Diablo)
                            && !app.settings.has_data(Game::Hellfire)
                        {
                            app.install_game = Game::Hellfire;
                        }
                    }
                    if ui.selectable_label(app.tab == Tab::Play, "Play").clicked() {
                        egui::Popup::close_all(ui.ctx());
                        app.tab = Tab::Play;
                    }
                });
            });
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(BACKGROUND))
        .show(ctx, |ui| match app.tab {
            Tab::Play => home::show(app, ui),
            Tab::Settings => settings::show(app, ui, frame),
            Tab::Install => install::show(app, ui),
        });
}
