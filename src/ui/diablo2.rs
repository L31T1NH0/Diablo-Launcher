use super::{
    theme::GOLD,
    widgets::{action_button, dropdown},
};
use crate::{app::Launcher, config::diablo2::D2Config, diablo2};
use eframe::egui::{self, Vec2};
use std::path::PathBuf;

fn checkbox(ui: &mut egui::Ui, cfg: &mut D2Config, key: &str, label: &str, default: bool) {
    let mut value = cfg.get(key, if default { "1" } else { "0" }) == "1";
    if ui.checkbox(&mut value, label).changed() {
        cfg.set(key, i32::from(value).to_string());
    }
}

pub(crate) fn settings(app: &mut Launcher, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    let busy = app.child.is_some() || app.install.is_some();
    egui::Frame::new().inner_margin(28.0).show(ui, |ui| {
        ui.heading("Diablo II settings");
        if busy {ui.colored_label(GOLD,"Close the game and wait for installation before changing settings.");}
        egui::ScrollArea::vertical().max_height((ui.available_height()-85.0).max(100.0)).show(ui, |ui| {
            ui.add_enabled_ui(!busy, |ui| {
                if let Ok(cfg) = &mut app.d2_config {
                    ui.heading("Display");
                    checkbox(ui,cfg,"fullscreen","Fullscreen",false);
                    checkbox(ui,cfg,"widescreen","Widescreen",false);
                    checkbox(ui,cfg,"video","Intro movies",true);
                    ui.horizontal(|ui| {
                        ui.label("Window scale");
                        let mut scale: i32 = cfg.get("scale","1").parse().unwrap_or(1);
                        let old = scale;
                        dropdown(ui,&format!("{scale}×"),|ui| {
                            for n in 1..=8 {ui.selectable_value(&mut scale,n,format!("{n}×"));}
                        });
                        if scale != old {cfg.set("scale",scale.to_string());}
                    });
                    ui.horizontal(|ui| {
                        ui.label("Image filter");
                        let mut mode = cfg.get("scalemode","pixelart").to_owned();
                        let old = mode.clone();
                        dropdown(ui,&old,|ui| {
                            for value in ["pixelart","nearest","linear"] {ui.selectable_value(&mut mode,value.into(),value);}
                        });
                        if mode != old {cfg.set("scalemode",mode);}
                    });
                    ui.add_space(12.0);
                    ui.heading("Sound");
                    for (key,label,default) in [("master_volume","Master volume","100"),("music_volume","Music volume","50")] {
                        let mut value: i32 = cfg.get(key,default).parse().unwrap_or(50);
                        if ui.add(egui::Slider::new(&mut value,0..=100).text(label)).changed() {cfg.set(key,value.to_string());}
                    }
                } else if let Err(error) = &app.d2_config {ui.colored_label(GOLD,error);}
                ui.add_space(12.0);
                egui::CollapsingHeader::new("Gameplay and controls").show(ui,|ui| {
                    for &(key,label) in diablo2::TOGGLES {
                        let value = app.settings.diablo2.toggles.entry(key.into()).or_insert(true);
                        ui.checkbox(value,label);
                    }
                    ui.small("Keyboard and gamepad bindings are available in the game's Configure Controls menu.");
                });
                egui::CollapsingHeader::new("Advanced").show(ui,|ui| {
                    ui.label("D2Decomp executable");
                    ui.add(egui::TextEdit::singleline(&mut app.settings.diablo2.executable).desired_width(f32::INFINITY));
                    ui.label("Diablo II MPQ folder");
                    ui.add(egui::TextEdit::singleline(&mut app.settings.diablo2.data).desired_width(f32::INFINITY));
                    if let Err(error) = app.settings.diablo2.validate() {ui.colored_label(GOLD,error);}
                });
            });
            egui::CollapsingHeader::new("Files and troubleshooting").show(ui,|ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Game data").clicked() {app.open(&PathBuf::from(&app.settings.diablo2.data));}
                    if ui.button("Saves").clicked() {app.open(&diablo2::config_path().with_file_name("save"));}
                    if ui.button("d2d.cfg").clicked() {app.open(&diablo2::config_path());}
                    if ui.button("Game log").clicked() {app.open(&crate::paths::game_log(crate::game::Game::DiabloII));}
                });
                ui.hyperlink_to("D2Decomp project", "https://github.com/psi29a/d2decomped");
                if !app.logs.is_empty() {ui.monospace(&app.logs);}
            });
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if action_button(ui,"Save changes",!busy,Vec2::new(162.0,36.0)).clicked() {
                let result = app.save_settings(frame); app.report(result,"Settings saved.");
            }
            if ui.add_enabled(!busy,egui::Button::new("Reload")).clicked() {
                egui::Popup::close_all(ui.ctx());app.reload_config();app.report(Ok(()),"Settings reloaded.");
            }
        });
        if !app.message.is_empty() {ui.colored_label(if app.error {egui::Color32::from_rgb(245,139,128)} else {GOLD},&app.message);}
    });
}
