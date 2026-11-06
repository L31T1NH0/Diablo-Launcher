use super::{theme::GOLD, widgets::action_button};
use crate::{app::Launcher, game::Game, installer};
use eframe::egui::{self, Color32, Vec2};

pub(crate) fn show(app: &mut Launcher, ui: &mut egui::Ui) {
    egui::Frame::new().inner_margin(28.0).show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Install the game");
            ui.add_space(8.0);
            ui.colored_label(GOLD, "Experimental installer — this may not work on your system.");
            ui.label("Install your system packages first. The README has the commands.");
            ui.hyperlink_to("View package requirements", "https://github.com/L31T1NH0/devilution-launcher#requirements");
            ui.add_space(18.0);
            let busy = app.install.is_some();
            let running = app.child.is_some();
            ui.add_enabled_ui(!busy && !running, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut app.install_game, Game::Diablo, "Diablo");
                    ui.selectable_value(&mut app.install_game, Game::Hellfire, "Diablo + Hellfire");
                    ui.selectable_value(&mut app.install_game, Game::DiabloII, "Diablo II + Lord of Destruction");
                });
                if app.install_game == Game::DiabloII {
                    ui.label("Import your Diablo II + Lord of Destruction 1.14d MPQs. About 2 GiB of free space is needed for a full copy.");
                    ui.label("Folder containing the MPQs");
                    ui.add(egui::TextEdit::singleline(&mut app.settings.diablo2.source).desired_width(f32::INFINITY));
                    ui.small("The engine is downloaded separately (about 5 MB). SDL3, OpenAL and ostree are required; see the README.");
                } else {
                    ui.label(if app.install_game == Game::Diablo {"Up to 550 MB to download."}
                        else {"Up to 725 MB to download, including Diablo."});
                }
                ui.label("Existing game files and saves are kept. Source files are never moved.");
                ui.add_space(10.0);
                if action_button(ui, if app.install_game == Game::DiabloII {"Install and import"} else {"Install missing files"}, true, Vec2::new(195.0, 42.0)).clicked() {
                    let result=app.start_install(ui.ctx());
                    app.report(result, "");
                }
            });
            if running {ui.colored_label(GOLD, "Close the game before installing files.");}
            if busy {
                ui.add_space(16.0);
                ui.label(&app.install_status);
                if let Some(total)=app.install_total {
                    ui.add(egui::ProgressBar::new((app.install_bytes as f64 / total as f64).clamp(0.0, 1.0) as f32)
                        .text(format!("{:.1} / {:.1} MiB", app.install_bytes as f64/1048576.0, total as f64/1048576.0)));
                } else {
                    ui.horizontal(|ui| {ui.spinner();ui.label(format!("{:.1} MiB",app.install_bytes as f64/1048576.0));});
                }
                if ui.button("Cancel").clicked() && let Some(job)=&app.install {job.cancel();}
            } else if !app.install_status.is_empty() {ui.label(&app.install_status);}
            if !app.message.is_empty() {
                ui.add_space(8.0);
                ui.colored_label(if app.error {Color32::from_rgb(245, 139, 128)} else {GOLD}, &app.message);
            }
            ui.add_space(18.0);
            egui::CollapsingHeader::new("Download sources and install folder").show(ui, |ui| {
                ui.label(format!("Game folder: {}",app.settings.data_for(app.install_game)));
                if app.install_game == Game::DiabloII {
                    ui.hyperlink_to("D2Decomp: official release",installer::D2_ENGINE_URL);
                    ui.hyperlink_to("Diablo II and LoD: Blizzard installers", "https://us.support.blizzard.com/en/article/13867");
                    ui.small("Import the MPQs from your own Diablo II + Lord of Destruction installation. They are stored separately from Diablo I.");
                } else {
                    ui.hyperlink_to("DevilutionX: official release",installer::ENGINE_URL);
                    ui.hyperlink_to("Diablo MPQ: cdn.pvpgn.pro",installer::DIABLO_URL);
                    ui.hyperlink_to("Hellfire MPQs: hoffmeister.li",installer::HELLFIRE_URL);
                    ui.small("Game data comes from third-party mirrors. Downloads are checked before installation.");
                }
            });
        });
    });
}
