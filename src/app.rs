use crate::{
    config::{GameConfig, catalog::Category},
    game::{Game, Settings, has_file},
    installer,
    paths::{data_home, log_path},
    ui::{self, Tab, theme::load_texture},
};
use eframe::egui;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};
pub(crate) struct Launcher {
    pub(crate) settings: Settings,
    pub(crate) child: Option<Child>,
    pub(crate) message: String,
    pub(crate) error: bool,
    pub(crate) logs: String,
    pub(crate) tab: Tab,
    pub(crate) config: Result<GameConfig, String>,
    pub(crate) hero: ui::hero::Hero,
    pub(crate) logo: egui::TextureHandle,
    pub(crate) wordmark: egui::TextureHandle,
    pub(crate) category: Category,
    pub(crate) install: Option<installer::Job>,
    pub(crate) install_game: Game,
    pub(crate) install_status: String,
    pub(crate) install_bytes: u64,
    pub(crate) install_total: Option<u64>,
}
impl Launcher {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::theme::configure(&cc.egui_ctx);
        let settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, "settings"))
            .unwrap_or_else(Settings::detected);
        let config = GameConfig::load(Path::new(&settings.data).join("diablo.ini"));
        Self {
            hero: ui::hero::Hero::new(&cc.egui_ctx),
            logo: load_texture(
                &cc.egui_ctx,
                "devilution-logo",
                include_bytes!("../assets/devilutionx-logo.png"),
            ),
            wordmark: load_texture(
                &cc.egui_ctx,
                "devilution-wordmark",
                include_bytes!("../assets/devilutionx-wordmark.png"),
            ),
            category: Category::Video,
            install: None,
            install_game: Game::Diablo,
            install_status: String::new(),
            install_bytes: 0,
            install_total: None,
            tab: Tab::Play,
            config,
            settings,
            child: None,
            message: String::new(),
            error: false,
            logs: String::new(),
        }
    }

    pub(crate) fn start_install(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if self.child.is_some() || self.install.is_some() {
            return Err("Close the game and wait for the current installation to finish.".into());
        }
        if self.config.as_ref().is_ok_and(GameConfig::dirty) {
            return Err("Save or reload your game settings before installing.".into());
        }
        if self.settings.data.trim().is_empty() {
            return Err("Choose a game data folder in Settings first.".into());
        }
        let plan = installer::Plan {
            game: self.install_game,
            data: PathBuf::from(&self.settings.data),
            executable: PathBuf::from(&self.settings.executable),
            managed_engine: data_home().join("devilutionx"),
        };
        self.install_status = "Preparing installation…".into();
        self.install_bytes = 0;
        self.install_total = None;
        self.message.clear();
        self.error = false;
        let ctx = ctx.clone();
        self.install = Some(installer::Job::start(plan, move || ctx.request_repaint()));
        Ok(())
    }

    pub(crate) fn save_settings(&mut self, frame: &mut eframe::Frame) -> Result<(), String> {
        let expected = Path::new(&self.settings.data).join("diablo.ini");
        match &mut self.config {
            Ok(config) if config.path == expected => config.save()?,
            Ok(_) => return Err("The data folder changed. Click Reload before saving.".into()),
            Err(error) => return Err(error.clone()),
        }
        if let Some(storage) = frame.storage_mut() {
            eframe::set_value(storage, "settings", &self.settings);
            storage.flush();
        }
        Ok(())
    }

    pub(crate) fn report(&mut self, result: Result<(), String>, success: &str) {
        match result {
            Ok(()) => {
                self.message = success.into();
                self.error = false;
            }
            Err(error) => {
                self.message = error;
                self.error = true;
            }
        }
    }

    pub(crate) fn launch(&mut self) -> Result<(), String> {
        if self.config.as_ref().is_ok_and(GameConfig::dirty) {
            return Err("Save or reload your settings before playing.".into());
        }
        let mut command = self.settings.command()?;
        let data = Path::new(&self.settings.data);
        let executable = fs::canonicalize(&self.settings.executable).map_err(|e| e.to_string())?;
        // --data-dir also changes where the engine looks for its own resources.
        for name in ["devilutionx.mpq", "fonts.mpq"] {
            let source = executable.parent().unwrap().join(name);
            if source.is_file() && !has_file(data, name) {
                fs::copy(&source, data.join(name))
                    .map_err(|e| format!("Could not copy {name}: {e}"))?;
            }
        }
        let path = log_path();
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let log = fs::File::create(&path).map_err(|e| e.to_string())?;
        command.stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?));
        command.stderr(Stdio::from(log));
        self.child = Some(
            command
                .spawn()
                .map_err(|e| format!("Could not start the game: {e}"))?,
        );
        self.logs.clear();
        Ok(())
    }

    pub(crate) fn open(&mut self, path: &Path) {
        if !path.exists() {
            self.report(
                Err(format!("File or folder not found: {}", path.display())),
                "",
            );
            return;
        }
        let result = Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|mut child| {
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
            })
            .map_err(|e| format!("Could not open {}: {e}", path.display()));
        self.report(result, "Opening…");
    }

    pub(crate) fn poll(&mut self, ctx: &egui::Context) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        match child.try_wait() {
            Ok(Some(status)) => {
                self.child = None;
                self.reload_config();
                self.logs = fs::read_to_string(log_path()).unwrap_or_default();
                self.error = !status.success();
                self.message = if status.success() {
                    "Game closed.".into()
                } else {
                    format!("The game stopped with {status}. Check the log for details.")
                };
            }
            Ok(None) => ctx.request_repaint_after(Duration::from_millis(500)),
            Err(error) => {
                self.message = format!("Could not check the game status: {error}");
                self.error = true;
                ctx.request_repaint_after(Duration::from_secs(1));
            }
        }
    }

    pub(crate) fn reload_config(&mut self) {
        self.config = GameConfig::load(Path::new(&self.settings.data).join("diablo.ini"));
    }

    pub(crate) fn poll_install(&mut self, ctx: &egui::Context) {
        while let Some(event) = self
            .install
            .as_ref()
            .and_then(|job| job.events.try_recv().ok())
        {
            match event {
                installer::Event::Progress {
                    message,
                    bytes,
                    total,
                } => {
                    self.install_status = message;
                    self.install_bytes = bytes;
                    self.install_total = total;
                }
                installer::Event::Finished(result) => {
                    self.install.take();
                    match result {
                        Ok(installed) => {
                            self.settings.executable =
                                installed.executable.to_string_lossy().into_owned();
                            self.settings.game = installed.game;
                            self.install_status = "Installation complete. You can now play.".into();
                            self.report(Ok(()), "Installation complete.");
                        }
                        Err(error) => {
                            self.install_status = "Installation stopped.".into();
                            self.report(Err(error), "");
                        }
                    }
                    self.reload_config();
                    ctx.request_repaint();
                }
            }
        }
        if self.install.is_some() {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }
}

impl eframe::App for Launcher {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "settings", &self.settings);
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.poll(ctx);
        self.poll_install(ctx);
        if self.install.is_none()
            && self.child.is_none()
            && !self.settings.has_data(self.settings.game)
        {
            self.settings.game = Game::Diablo;
        }
        ui::show(self, ctx, frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_round_trip() {
        struct Memory(std::collections::HashMap<String, String>);
        impl eframe::Storage for Memory {
            fn get_string(&self, key: &str) -> Option<String> {
                self.0.get(key).cloned()
            }
            fn set_string(&mut self, key: &str, value: String) {
                self.0.insert(key.into(), value);
            }
            fn flush(&mut self) {}
        }
        let mut storage = Memory(Default::default());
        let settings = Settings {
            data: "/data with spaces".into(),
            game: Game::Hellfire,
            ..Settings::default()
        };
        eframe::set_value(&mut storage, "settings", &settings);
        let restored: Settings = eframe::get_value(&storage, "settings").unwrap();
        assert_eq!(restored.data, settings.data);
        assert_eq!(restored.game, Game::Hellfire);
    }
}
