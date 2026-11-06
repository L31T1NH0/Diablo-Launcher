mod app;
mod config;
mod diablo2;
mod game;
mod installer;
mod paths;
mod ui;
use eframe::egui;

fn main() -> eframe::Result {
    eframe::run_native(
        "Diablo Launcher",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                // Keep the existing storage id when renaming the app.
                .with_app_id("Devilution Launcher")
                .with_icon(
                    eframe::icon_data::from_png_bytes(include_bytes!(
                        "../assets/devilutionx-logo.png"
                    ))
                    .expect("Invalid bundled icon"),
                )
                .with_inner_size([1000.0, 680.0])
                .with_min_inner_size([620.0, 540.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(app::Launcher::new(cc)))),
    )
}
