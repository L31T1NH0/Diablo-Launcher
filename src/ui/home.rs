use super::{
    Tab,
    theme::{BACKGROUND, GOLD, MUTED},
    widgets::action_button,
};
use crate::{app::Launcher, config::GameConfig, game::Game};
use eframe::egui::{self, Color32, Vec2};
use std::time::Duration;
fn gradient(painter: &egui::Painter, rect: egui::Rect, top: Color32, bottom: Color32) {
    let mut mesh = egui::Mesh::default();
    for (pos, color) in [
        (rect.left_top(), top),
        (rect.right_top(), top),
        (rect.right_bottom(), bottom),
        (rect.left_bottom(), bottom),
    ] {
        mesh.colored_vertex(pos, color);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
}

pub(crate) fn show(app: &mut Launcher, ui: &mut egui::Ui) {
    let height = (ui.available_height()
        - if ui.available_width() < 640.0 {
            140.0
        } else {
            110.0
        })
    .max(220.0);
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let image_aspect = app.hero.size_vec2().x / app.hero.size_vec2().y;
    let rect_aspect = rect.width() / rect.height();
    let uv = if image_aspect > rect_aspect {
        let width = rect_aspect / image_aspect;
        egui::Rect::from_min_max(
            egui::pos2((1.0 - width) / 2.0, 0.0),
            egui::pos2((1.0 + width) / 2.0, 1.0),
        )
    } else {
        let height = image_aspect / rect_aspect;
        egui::Rect::from_min_max(
            egui::pos2(0.0, (1.0 - height) / 2.0),
            egui::pos2(1.0, (1.0 + height) / 2.0),
        )
    };
    ui.painter().image(app.hero.id(), rect, uv, Color32::WHITE);
    gradient(
        ui.painter(),
        rect,
        Color32::from_black_alpha(10),
        Color32::from_black_alpha(110),
    );
    gradient(
        ui.painter(),
        egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.center().y),
            rect.right_bottom(),
        ),
        Color32::TRANSPARENT,
        BACKGROUND,
    );
    gradient(
        ui.painter(),
        egui::Rect::from_min_max(
            rect.left_top(),
            egui::pos2(rect.right(), rect.top() + (rect.height() * 0.3).min(120.0)),
        ),
        BACKGROUND,
        Color32::TRANSPARENT,
    );
    let left = rect.left() + 32.0;
    ui.painter().text(
        egui::pos2(left, rect.bottom() - 90.0),
        egui::Align2::LEFT_BOTTOM,
        "DIABLO",
        egui::FontId::proportional(64.0),
        Color32::from_rgb(245, 235, 214),
    );
    ui.painter().text(
        egui::pos2(left + 3.0, rect.bottom() - 64.0),
        egui::Align2::LEFT_BOTTOM,
        "A classic. A new adventure.",
        egui::FontId::proportional(15.0),
        MUTED,
    );

    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(32, 14))
        .show(ui, |ui| {
            let ready = app.settings.validate();
            let dirty = app.config.as_ref().is_ok_and(GameConfig::dirty);
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
                    if action_button(
                        ui,
                        label,
                        !running && !installing && !dirty,
                        Vec2::new(148.0, 48.0),
                    )
                    .clicked()
                    {
                        if ready.is_ok() {
                            let result = app.launch();
                            app.report(result, "");
                        } else {
                            app.tab = Tab::Install;
                        }
                        ui.ctx().request_repaint_after(Duration::from_millis(500));
                    }
                    ui.add_enabled_ui(!running && !installing, |ui| {
                        super::widgets::dropdown(ui, app.settings.game.title(), |ui| {
                            for game in [Game::Diablo, Game::Hellfire] {
                                if app.settings.has_data(game) {
                                    ui.selectable_value(&mut app.settings.game, game, game.title());
                                }
                            }
                            if !app.settings.has_data(Game::Diablo) {
                                ui.label("No game installed yet");
                            }
                        });
                    });
                });
            });
            if !app.message.is_empty() && app.error {
                ui.colored_label(Color32::from_rgb(245, 139, 128), &app.message);
            }
        });
}
