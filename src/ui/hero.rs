use super::theme::{BACKGROUND, MUTED, load_texture};
use crate::game::Game;
use eframe::egui::{self, Color32, Vec2};

pub(crate) struct Hero {
    diablo: egui::TextureHandle,
    hellfire: egui::TextureHandle,
    diablo2: egui::TextureHandle,
}

impl Hero {
    pub(crate) fn new(ctx: &egui::Context) -> Self {
        Self {
            diablo2: load_texture(
                ctx,
                "diablo2-hero",
                include_bytes!("../../assets/diablo2-hero.jpg"),
            ),
            diablo: load_texture(
                ctx,
                "diablo-hero",
                include_bytes!("../../assets/diablo-hero.jpg"),
            ),
            hellfire: load_texture(
                ctx,
                "hellfire-hero",
                include_bytes!("../../assets/hellfire-hero.png"),
            ),
        }
    }

    pub(crate) fn show(&self, ui: &mut egui::Ui, game: Game) {
        let footer = if ui.available_width() < 640.0 {
            140.0
        } else {
            110.0
        };
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(
                ui.available_width(),
                (ui.available_height() - footer).max(220.0),
            ),
            egui::Sense::hover(),
        );
        let mut weights = [0.0_f32; 3];
        for (i, edition) in Game::ALL.iter().enumerate() {
            let linear = ui.ctx().animate_value_with_time(
                ui.make_persistent_id(("hero-edition", i)),
                if game == *edition { 1.0 } else { 0.0 },
                0.45,
            );
            weights[i] = linear * linear * (3.0 - 2.0 * linear);
        }
        let sum = weights.iter().sum::<f32>().max(f32::EPSILON);
        for weight in &mut weights {
            *weight /= sum;
        }
        let painter = ui.painter();
        painter.rect_filled(rect, 0, BACKGROUND);
        // Fade every layer so transparent artwork reveals the background.
        for (i, texture) in [&self.diablo, &self.hellfire, &self.diablo2]
            .into_iter()
            .enumerate()
        {
            if weights[i] > 0.0 {
                painter.image(
                    texture.id(),
                    rect,
                    cover_uv(
                        texture.size_vec2(),
                        rect.size(),
                        if i == 2 { 0.85 } else { 0.5 },
                    ),
                    Color32::from_white_alpha((weights[i] * 255.0).round() as u8),
                );
            }
        }
        gradient(
            painter,
            rect,
            Color32::from_black_alpha(10),
            Color32::from_black_alpha(110),
        );
        gradient(
            painter,
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.center().y),
                rect.right_bottom(),
            ),
            Color32::TRANSPARENT,
            BACKGROUND,
        );
        gradient(
            painter,
            egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.right(), rect.top() + (rect.height() * 0.3).min(120.0)),
            ),
            BACKGROUND,
            Color32::TRANSPARENT,
        );
        // Fade the old copy out before showing the new one, keeping the titles legible.
        for (title, tagline, alpha) in [
            (
                "DIABLO",
                "A classic. A new adventure.",
                (2.0 * weights[0] - 1.0).clamp(0.0, 1.0),
            ),
            (
                "HELLFIRE",
                "The darkness goes deeper.",
                (2.0 * weights[1] - 1.0).clamp(0.0, 1.0),
            ),
            (
                "DIABLO II",
                "Follow the Dark Wanderer.",
                (2.0 * weights[2] - 1.0).clamp(0.0, 1.0),
            ),
        ] {
            if alpha > 0.0 {
                let offset = (1.0 - alpha) * 6.0;
                painter.text(
                    egui::pos2(rect.left() + 32.0, rect.bottom() - 90.0 + offset),
                    egui::Align2::LEFT_BOTTOM,
                    title,
                    egui::FontId::proportional(64.0),
                    Color32::from_rgb(245, 235, 214).gamma_multiply(alpha),
                );
                painter.text(
                    egui::pos2(rect.left() + 35.0, rect.bottom() - 64.0 + offset),
                    egui::Align2::LEFT_BOTTOM,
                    tagline,
                    egui::FontId::proportional(15.0),
                    MUTED.gamma_multiply(alpha),
                );
            }
        }
    }
}

fn cover_uv(image: Vec2, viewport: Vec2, anchor: f32) -> egui::Rect {
    let image_aspect = image.x / image.y;
    let viewport_aspect = viewport.x / viewport.y;
    if image_aspect > viewport_aspect {
        let width = viewport_aspect / image_aspect;
        egui::Rect::from_min_max(
            egui::pos2((1.0 - width) * anchor, 0.0),
            egui::pos2((1.0 - width) * anchor + width, 1.0),
        )
    } else {
        let height = image_aspect / viewport_aspect;
        egui::Rect::from_min_max(
            egui::pos2(0.0, (1.0 - height) / 2.0),
            egui::pos2(1.0, (1.0 + height) / 2.0),
        )
    }
}

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
