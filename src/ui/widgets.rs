use super::theme::{PRIMARY, PRIMARY_ACTIVE, PRIMARY_HOVER};
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
        widgets.hovered.weak_bg_fill = PRIMARY_HOVER;
        widgets.hovered.bg_fill = PRIMARY_HOVER;
        widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(216, 95, 102));
        widgets.active.weak_bg_fill = PRIMARY_ACTIVE;
        widgets.active.bg_fill = PRIMARY_ACTIVE;
        ui.add_enabled(
            enabled,
            egui::Button::new(RichText::new(label).strong().color(Color32::WHITE))
                .corner_radius(7)
                .min_size(size),
        )
    })
    .inner
}

/// One control with independent launch and edition-menu hit areas.
pub(crate) fn split_action_button(
    ui: &mut egui::Ui,
    label: &str,
    edition: &str,
    enabled: bool,
    menu_enabled: bool,
    contents: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let (rect, outer) = ui.allocate_exact_size(Vec2::new(224.0, 54.0), egui::Sense::hover());
    let menu_rect = egui::Rect::from_min_max(
        egui::pos2(rect.right() - 44.0, rect.top()),
        rect.right_bottom(),
    );
    let action_rect = egui::Rect::from_min_max(rect.left_top(), menu_rect.left_bottom());
    let enabled = enabled && ui.is_enabled();
    let menu_enabled = menu_enabled && ui.is_enabled();
    let action = ui.interact(
        action_rect,
        outer.id.with("action"),
        if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    let menu = ui.interact(
        menu_rect,
        outer.id.with("edition"),
        if menu_enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    action.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            enabled,
            format!("{label} {edition}"),
        )
    });
    menu.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, menu_enabled, "Choose game")
    });
    let disabled = ui.visuals().widgets.noninteractive.bg_fill;
    ui.painter()
        .rect_filled(rect, 7, if enabled { PRIMARY } else { disabled });
    let action_fill = if enabled && action.is_pointer_button_down_on() {
        Some(PRIMARY_ACTIVE)
    } else if enabled && (action.hovered() || action.has_focus()) {
        Some(PRIMARY_HOVER)
    } else {
        None
    };
    if let Some(fill) = action_fill {
        ui.painter().rect_filled(
            action_rect,
            egui::CornerRadius {
                nw: 7,
                sw: 7,
                ne: 0,
                se: 0,
            },
            fill,
        );
    }
    let menu_fill = if !menu_enabled {
        if enabled { PRIMARY } else { disabled }
    } else if menu.is_pointer_button_down_on() {
        PRIMARY_ACTIVE
    } else if menu.hovered() || menu.has_focus() {
        PRIMARY_HOVER
    } else {
        PRIMARY
    };
    ui.painter().rect_filled(
        menu_rect,
        egui::CornerRadius {
            nw: 0,
            sw: 0,
            ne: 7,
            se: 7,
        },
        menu_fill,
    );
    let text = if enabled {
        Color32::WHITE
    } else {
        Color32::from_gray(135)
    };
    ui.painter().text(
        action_rect.center() + egui::vec2(0.0, -8.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(17.0),
        text,
    );
    ui.painter().text(
        action_rect.center() + egui::vec2(0.0, 13.0),
        egui::Align2::CENTER_CENTER,
        edition,
        egui::FontId::proportional(12.0),
        if enabled {
            Color32::from_white_alpha(205)
        } else {
            text
        },
    );
    ui.painter().line_segment(
        [
            menu_rect.left_top() + egui::vec2(0.0, 12.0),
            menu_rect.left_bottom() - egui::vec2(0.0, 12.0),
        ],
        egui::Stroke::new(1.0_f32, Color32::from_white_alpha(35)),
    );
    let center = menu_rect.center();
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            center + egui::vec2(-4.0, -2.0),
            center + egui::vec2(4.0, -2.0),
            center + egui::vec2(0.0, 3.0),
        ],
        if menu_enabled {
            Color32::WHITE
        } else {
            Color32::from_gray(115)
        },
        egui::Stroke::NONE,
    ));
    let popup = action.id.with("edition-menu");
    if !menu_enabled || !ui.clip_rect().intersects(rect) {
        egui::Popup::close_id(ui.ctx(), popup);
    } else {
        egui::Popup::menu(&menu)
            .id(popup)
            .anchor(rect)
            .align(egui::RectAlign::TOP_END)
            .width(rect.width())
            .show(|ui| {
                ui.set_min_width(ui.available_width());
                contents(ui);
            });
    }
    if menu_enabled {
        menu.on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("Choose game");
    }
    if enabled {
        action.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        action
    }
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
    fn split_button_keeps_launch_and_menu_clicks_independent() {
        let ctx = egui::Context::default();
        let render = |events: Vec<egui::Event>, enabled| {
            let mut response = None;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(800.0, 600.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        response = Some(split_action_button(
                            ui,
                            "PLAY",
                            "Hellfire",
                            enabled,
                            enabled,
                            |ui| {
                                ui.label("Diablo");
                            },
                        ));
                    });
                },
            );
            response.unwrap()
        };
        let initial = render(vec![], true);
        let click = |pos, enabled| {
            render(
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: Default::default(),
                    },
                ],
                enabled,
            );
            render(
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                }],
                enabled,
            )
        };
        let menu_click = click(initial.rect.right_center() + egui::vec2(22.0, 0.0), true);
        assert!(!menu_click.clicked(), "The arrow must only open the menu");
        let popup = initial.id.with("edition-menu");
        assert!(egui::Popup::is_id_open(&ctx, popup));
        egui::Popup::close_id(&ctx, popup);
        assert!(click(initial.rect.center(), true).clicked());
        assert!(!click(initial.rect.center(), false).clicked());
    }

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
