use super::theme::PRIMARY;
use eframe::egui::{self, Color32, RichText, Vec2};

pub(crate) fn action_button(
    ui: &mut egui::Ui,
    label: &str,
    enabled: bool,
    size: Vec2,
) -> egui::Response {
    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.weak_bg_fill = PRIMARY;
        widgets.inactive.bg_fill = PRIMARY;
        widgets.inactive.bg_stroke = egui::Stroke::NONE;
        widgets.hovered.weak_bg_fill = Color32::from_rgb(183, 48, 57);
        widgets.hovered.bg_fill = Color32::from_rgb(183, 48, 57);
        widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(216, 95, 102));
        widgets.active.weak_bg_fill = Color32::from_rgb(119, 27, 36);
        widgets.active.bg_fill = Color32::from_rgb(119, 27, 36);
        ui.add_enabled(
            enabled,
            egui::Button::new(RichText::new(label).strong().color(Color32::WHITE))
                .corner_radius(7)
                .min_size(size),
        )
    })
    .inner
}

pub(crate) fn dropdown(
    ui: &mut egui::Ui,
    selected: &str,
    contents: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let response = ui.add(
        egui::Button::new(selected)
            .right_text("   ")
            .min_size(Vec2::new(180.0, 32.0)),
    );
    let center = egui::pos2(response.rect.right() - 14.0, response.rect.center().y);
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            center + egui::vec2(-4.0, -2.0),
            center + egui::vec2(4.0, -2.0),
            center + egui::vec2(0.0, 3.0),
        ],
        ui.style().interact(&response).fg_stroke.color,
        egui::Stroke::NONE,
    ));
    let popup = response.id.with("clipped-menu");
    if !ui.clip_rect().intersects(response.rect) {
        if egui::Popup::is_id_open(ui.ctx(), popup) {
            egui::Popup::close_id(ui.ctx(), popup);
            ui.ctx().request_repaint();
        }
    } else {
        egui::Popup::menu(&response)
            .id(popup)
            .width(response.rect.width())
            .show(|ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                egui::ScrollArea::vertical()
                    .id_salt(popup)
                    .max_height(240.0)
                    .show(ui, contents);
            });
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn popup_closes_when_its_anchor_scrolls_out_of_view() {
        let ctx = egui::Context::default();
        let mut id = None;
        let mut render = |offset: f32| {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(800.0, 600.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical()
                            .vertical_scroll_offset(offset)
                            .show(ui, |ui| {
                                id = Some(
                                    dropdown(ui, "Resolution", |ui| {
                                        ui.label("1280 × 720");
                                    })
                                    .id
                                    .with("clipped-menu"),
                                );
                                ui.add_space(2000.0);
                            });
                    });
                },
            );
            id.unwrap()
        };
        let id = render(0.0);
        egui::Popup::open_id(&ctx, id);
        assert!(egui::Popup::is_id_open(&ctx, id));
        render(0.0);
        assert!(
            egui::Popup::is_id_open(&ctx, id),
            "The menu should stay open while its button is visible"
        );
        render(500.0);
        assert!(!egui::Popup::is_id_open(&ctx, id));
    }
}
