use crate::{
    game_config::{ExtraOption, GameConfig},
    option_catalog::{self, Category, Kind},
};
use eframe::egui::{self, RichText, Vec2};

// Only show the menu while its button intersects the visible scroll area.
pub fn dropdown(
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

pub fn game_options(
    ui: &mut egui::Ui,
    config: &mut GameConfig,
    selected: Category,
    skip_intro: &mut bool,
) {
    let options = &mut config.options;
    match selected {
        Category::Video => {
            row(ui, "Resolution", |ui| {
                dropdown(
                    ui,
                    &format!("{} × {}", options.width, options.height),
                    |ui| {
                        for (w, h) in [
                            (640, 480),
                            (960, 540),
                            (1280, 720),
                            (1600, 900),
                            (1920, 1080),
                            (2560, 1440),
                            (3840, 2160),
                        ] {
                            if ui
                                .selectable_label(
                                    (options.width, options.height) == (w, h),
                                    format!("{w} × {h}"),
                                )
                                .clicked()
                            {
                                options.width = w;
                                options.height = h;
                            }
                        }
                    },
                );
            });
            row(ui, "Custom resolution", |ui| {
                ui.add(
                    egui::DragValue::new(&mut options.width)
                        .range(640..=16384)
                        .suffix(" px"),
                );
                ui.label("×");
                ui.add(
                    egui::DragValue::new(&mut options.height)
                        .range(480..=16384)
                        .suffix(" px"),
                );
            });
            ui.checkbox(&mut options.fullscreen, "Fullscreen");
            ui.checkbox(&mut options.upscale, "Scale image to window");
            row(ui, "Frame rate", |ui| {
                dropdown(
                    ui,
                    match options.frame_control {
                        0 => "Unlimited",
                        1 => "Vertical sync",
                        2 => "Limit FPS",
                        _ => "Custom",
                    },
                    |ui| {
                        ui.selectable_value(&mut options.frame_control, 0, "Unlimited");
                        ui.selectable_value(&mut options.frame_control, 1, "Vertical sync");
                        ui.selectable_value(&mut options.frame_control, 2, "Limit FPS");
                    },
                );
            });
        }
        Category::Audio => {
            row(ui, "Music", |ui| {
                ui.add(egui::Slider::new(&mut options.music, 0..=100).suffix(" %"));
            });
            row(ui, "Sound effects", |ui| {
                ui.add(egui::Slider::new(&mut options.sound, 0..=100).suffix(" %"));
            });
        }
        Category::Game => {
            row(ui, "Language", |ui| {
                let current = LANGUAGES
                    .iter()
                    .find(|(code, _)| *code == options.language)
                    .map(|(_, name)| *name)
                    .unwrap_or(&options.language)
                    .to_owned();
                dropdown(ui, &current, |ui| {
                    for (code, name) in LANGUAGES {
                        ui.selectable_value(&mut options.language, (*code).into(), *name);
                    }
                });
            });
            ui.checkbox(&mut options.run_in_town, "Run in town");
            ui.checkbox(&mut options.auto_gold, "Pick up gold automatically");
            ui.checkbox(&mut options.quick_cast, "Cast spells with hotkeys");
        }
        Category::Interface => {
            ui.checkbox(&mut options.show_fps, "Show FPS");
        }
        Category::Startup => {
            ui.checkbox(skip_intro, "Skip intro videos when starting the game");
        }
        Category::Keyboard => {
            ui.label(RichText::new("Use game key names, such as F5, LALT or RCONTROL. Leave a field empty to clear the shortcut.").color(crate::MUTED));
        }
        Category::Controller => {
            ui.label(
                RichText::new("Use + for button combinations, such as Select+LT. Leave a field empty to clear the shortcut.")
                    .color(crate::MUTED),
            );
        }
        _ => {}
    }
    ui.add_space(8.0);
    for option in config.extra.iter_mut().filter(|o| {
        !o.hellfire_only() && o.section != "GameMode" && option_catalog::category(o) == selected
    }) {
        let enabled =
            !(selected == Category::Startup && *skip_intro && option.key.ends_with("Intro"));
        ui.add_enabled_ui(enabled, |ui| option_ui(ui, option));
    }
}

pub fn expansion_options(ui: &mut egui::Ui, config: &mut GameConfig, skip_intro: bool) {
    for option in config
        .extra
        .iter_mut()
        .filter(|o| o.hellfire_only() || o.section == "GameMode")
    {
        ui.add_enabled_ui(!(skip_intro && option.key.ends_with("Intro")), |ui| {
            option_ui(ui, option)
        });
    }
}

fn row(ui: &mut egui::Ui, label: &str, contents: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal_wrapped(|ui| {
        ui.add_sized(
            [220.0, 26.0],
            egui::Label::new(label).halign(egui::Align::LEFT).wrap(),
        );
        contents(ui);
    });
}

fn option_ui(ui: &mut egui::Ui, option: &mut ExtraOption) {
    ui.push_id((&option.section.clone(), &option.key.clone()), |ui| {
        let label = option_catalog::label(option);
        match option_catalog::kind(option) {
            Kind::Toggle => {
                let mut checked = option.value == "1";
                if ui
                    .checkbox(&mut checked, &label)
                    .on_hover_text(format!("[{}] {}", option.section, option.key))
                    .changed()
                {
                    option.value = if checked { "1" } else { "0" }.into();
                }
            }
            Kind::Choices(choices) => row(ui, &label, |ui| {
                let current = choices
                    .iter()
                    .find(|(v, _)| *v == option.value)
                    .map(|(_, label)| *label)
                    .unwrap_or(&option.value)
                    .to_owned();
                dropdown(ui, &current, |ui| {
                    for (val, label) in choices {
                        ui.selectable_value(&mut option.value, (*val).into(), *label);
                    }
                });
            }),
            Kind::Integer(min, max) => row(ui, &label, |ui| {
                let mut val = option.value.parse::<i32>().unwrap_or(min);
                if ui
                    .add(egui::DragValue::new(&mut val).range(min..=max))
                    .changed()
                {
                    option.value = val.to_string();
                }
            }),
            Kind::Decimal(min, max) => row(ui, &label, |ui| {
                let mut val = option.value.parse::<f64>().unwrap_or(min);
                if ui
                    .add(egui::Slider::new(&mut val, min..=max).max_decimals(3))
                    .changed()
                {
                    option.value = val.to_string();
                }
            }),
            Kind::Text => row(ui, &label, |ui| {
                ui.add(egui::TextEdit::singleline(&mut option.value).desired_width(260.0))
                    .on_hover_text(format!("[{}] {}", option.section, option.key));
            }),
        }
    });
}

const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("pt_BR", "Portuguese (Brazil)"),
    ("es", "Spanish"),
    ("de", "German"),
    ("fr", "French"),
    ("it", "Italian"),
    ("pl", "Polish"),
    ("ru", "Russian"),
    ("uk", "Ukrainian"),
    ("be", "Belarusian"),
    ("bg", "Bulgarian"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("el", "Greek"),
    ("et", "Estonian"),
    ("fi", "Finnish"),
    ("he", "Hebrew"),
    ("hr", "Croatian"),
    ("hu", "Hungarian"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("ro", "Romanian"),
    ("sv", "Swedish"),
    ("tr", "Turkish"),
    ("tt", "Tatar"),
    ("zh_CN", "Chinese (Simplified)"),
    ("zh_TW", "Chinese (Traditional)"),
];

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
