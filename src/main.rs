use eframe::egui::{self, Color32, RichText, Vec2};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

const GOLD: Color32 = Color32::from_rgb(222, 167, 94);
const MUTED: Color32 = Color32::from_rgb(160, 154, 148);

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
enum Game {
    #[default]
    Diablo,
    Hellfire,
}

impl Game {
    fn flag(self) -> &'static str {
        match self {
            Self::Diablo => "--diablo",
            Self::Hellfire => "--hellfire",
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    executable: String,
    data: String,
    game: Game,
    skip_intro: bool,
}

fn home() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn data_home() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/share"))
}

fn log_path() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("devilution-launcher/game.log")
}

impl Settings {
    fn detected() -> Self {
        // O instalador deste repositório usa ~/.local/share mesmo com XDG personalizado.
        let local = home().join(".local/share/devilutionx/devilutionx");
        let executable = [local, data_home().join("devilutionx/devilutionx")]
            .into_iter()
            .chain(
                env::split_paths(&env::var_os("PATH").unwrap_or_default())
                    .map(|p| p.join("devilutionx")),
            )
            .find(|p| p.is_file())
            .unwrap_or_default();
        let legacy = home().join(".local/share/diasurgical/devilution");
        let data = if legacy.is_dir() {
            legacy
        } else {
            data_home().join("diasurgical/devilution")
        };
        Self {
            executable: executable.to_string_lossy().into_owned(),
            data: data.to_string_lossy().into_owned(),
            skip_intro: true,
            ..Self::default()
        }
    }

    fn validate(&self) -> Result<(), String> {
        if !Path::new(&self.executable).is_file() {
            return Err("Selecione o executável do DevilutionX em Configuração.".into());
        }
        let data = Path::new(&self.data);
        if !data.is_dir() {
            return Err("A pasta dos dados não existe. Confira Configuração.".into());
        }
        let mut required = vec!["diabdat.mpq"];
        if self.game == Game::Hellfire {
            required.extend(["hellfire.mpq", "hfmonk.mpq", "hfmusic.mpq", "hfvoice.mpq"]);
        }
        let missing: Vec<_> = required
            .into_iter()
            .filter(|name| !has_file(data, name))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Faltam arquivos na pasta dos dados: {}",
                missing.join(", ")
            ))
        }
    }

    fn command(&self) -> Result<Command, String> {
        self.validate()?;
        let executable = fs::canonicalize(&self.executable).map_err(|e| e.to_string())?;
        let data = fs::canonicalize(&self.data).map_err(|e| e.to_string())?;
        let directory = executable
            .parent()
            .ok_or("Executável sem pasta de origem")?;
        let mut command = Command::new(&executable);
        command
            .current_dir(directory)
            .arg("--data-dir")
            .arg(&data)
            .arg("--save-dir")
            .arg(&data)
            .arg("--config-dir")
            .arg(&data)
            .arg(self.game.flag());
        if self.skip_intro {
            command.arg("-n");
        }
        let mut libraries = vec![directory.to_path_buf()];
        if let Some(existing) = env::var_os("LD_LIBRARY_PATH") {
            libraries.extend(env::split_paths(&existing));
        }
        command.env(
            "LD_LIBRARY_PATH",
            env::join_paths(libraries).map_err(|e| e.to_string())?,
        );
        Ok(command)
    }
}

fn has_file(directory: &Path, name: &str) -> bool {
    fs::read_dir(directory).is_ok_and(|entries| {
        entries.filter_map(Result::ok).any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(name)
                && entry.path().is_file()
                && entry.metadata().is_ok_and(|m| m.len() > 0)
        })
    })
}

struct Launcher {
    settings: Settings,
    child: Option<Child>,
    message: String,
    error: bool,
    logs: String,
}

impl Launcher {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(22, 20, 21);
        visuals.window_fill = Color32::from_rgb(30, 27, 28);
        visuals.selection.bg_fill = Color32::from_rgb(110, 65, 35);
        cc.egui_ctx.set_visuals(visuals);
        cc.egui_ctx.style_mut(|s| {
            s.spacing.item_spacing = Vec2::new(12.0, 10.0);
            s.spacing.button_padding = Vec2::new(16.0, 10.0);
        });
        let settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, "settings"))
            .unwrap_or_else(Settings::detected);
        Self {
            settings,
            child: None,
            message: String::new(),
            error: false,
            logs: String::new(),
        }
    }

    fn report(&mut self, result: Result<(), String>, success: &str) {
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

    fn launch(&mut self) -> Result<(), String> {
        let mut command = self.settings.command()?;
        let data = Path::new(&self.settings.data);
        let executable = fs::canonicalize(&self.settings.executable).map_err(|e| e.to_string())?;
        // --data-dir também altera onde o motor procura seus recursos adicionais.
        for name in ["devilutionx.mpq", "fonts.mpq"] {
            let source = executable.parent().unwrap().join(name);
            if source.is_file() && !has_file(data, name) {
                fs::copy(&source, data.join(name))
                    .map_err(|e| format!("Não foi possível copiar {name}: {e}"))?;
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
                .map_err(|e| format!("Não foi possível iniciar: {e}"))?,
        );
        self.logs.clear();
        Ok(())
    }

    fn open(&mut self, path: &Path) {
        if !path.exists() {
            self.report(Err(format!("Ainda não existe: {}", path.display())), "");
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
            .map_err(|e| format!("Não foi possível abrir {}: {e}", path.display()));
        self.report(result, "Abrindo no aplicativo padrão…");
    }

    fn poll(&mut self, ctx: &egui::Context) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        match child.try_wait() {
            Ok(Some(status)) => {
                self.child = None;
                self.logs = fs::read_to_string(log_path()).unwrap_or_default();
                self.error = !status.success();
                self.message = if status.success() {
                    "Jogo encerrado.".into()
                } else {
                    format!("O jogo encerrou com {status}. Confira o registro abaixo.")
                };
            }
            Ok(None) => ctx.request_repaint_after(Duration::from_millis(500)),
            Err(error) => {
                self.message = format!("Não foi possível consultar o jogo: {error}");
                self.error = true;
                ctx.request_repaint_after(Duration::from_secs(1));
            }
        }
    }
}

impl eframe::App for Launcher {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "settings", &self.settings);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.poll(ctx);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(22, 20, 21))
                    .inner_margin(28.0),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(
                        RichText::new("SANCTUARY  /  DEVILUTIONX")
                            .size(12.0)
                            .color(GOLD),
                    );
                    ui.add_space(10.0);
                    ui.heading(RichText::new("Retorne a Tristram.").size(32.0));
                    ui.label(
                        RichText::new("Seu portal para Diablo I e Hellfire no Linux.").color(MUTED),
                    );
                    ui.add_space(22.0);
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.settings.game, Game::Diablo, "DIABLO I");
                        ui.selectable_value(&mut self.settings.game, Game::Hellfire, "HELLFIRE");
                    });
                    ui.add_space(8.0);
                    let ready = self.settings.validate();
                    let running = self.child.is_some();
                    egui::Frame::group(ui.style())
                        .inner_margin(18.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new(if self.settings.game == Game::Diablo {
                                    "A escuridão espera sob a catedral."
                                } else {
                                    "Uma nova ameaça emerge das profundezas."
                                })
                                .size(18.0),
                            );
                            ui.add_space(8.0);
                            let (status, color) = if running {
                                ("Jogo em execução".into(), GOLD)
                            } else {
                                match &ready {
                                    Ok(()) => (
                                        "Pronto para jogar".into(),
                                        Color32::from_rgb(139, 192, 145),
                                    ),
                                    Err(e) => (e.clone(), GOLD),
                                }
                            };
                            ui.label(RichText::new(status).color(color));
                            ui.add_space(10.0);
                            if ui
                                .add_enabled(
                                    !running && ready.is_ok(),
                                    egui::Button::new(
                                        RichText::new(if running { "EM JOGO" } else { "JOGAR" })
                                            .size(20.0)
                                            .color(Color32::from_rgb(22, 20, 21)),
                                    )
                                    .fill(GOLD)
                                    .min_size(Vec2::new(ui.available_width(), 52.0)),
                                )
                                .clicked()
                            {
                                let result = self.launch();
                                self.report(result, "DevilutionX iniciado.");
                                ctx.request_repaint_after(Duration::from_millis(500));
                            }
                            ui.checkbox(&mut self.settings.skip_intro, "Pular vídeos de abertura");
                        });
                    ui.add_space(12.0);
                    egui::CollapsingHeader::new("Configuração")
                        .default_open(ready.is_err())
                        .show(ui, |ui| {
                            ui.add_enabled_ui(!running, |ui| {
                                ui.label("Executável do DevilutionX");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.settings.executable)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("/caminho/para/devilutionx"),
                                );
                                ui.label("Pasta com DIABDAT.MPQ e saves");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.settings.data)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("/caminho/para/dados"),
                                );
                                ui.horizontal_wrapped(|ui| {
                                    if ui.button("Detectar instalação").clicked() {
                                        let detected = Settings::detected();
                                        self.settings.executable = detected.executable;
                                        self.settings.data = detected.data;
                                    }
                                    if ui.button("Salvar configuração").clicked() {
                                        if let Some(storage) = frame.storage_mut() {
                                            eframe::set_value(storage, "settings", &self.settings);
                                            storage.flush();
                                            self.report(Ok(()), "Configuração salva.");
                                        } else {
                                            self.report(
                                                Err("Armazenamento indisponível nesta sessão."
                                                    .into()),
                                                "",
                                            );
                                        }
                                    }
                                });
                            });
                            ui.small("Os caminhos também são salvos ao fechar o launcher.");
                        });
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Pasta dos dados").clicked() {
                            self.open(&PathBuf::from(&self.settings.data));
                        }
                        if ui.button("diablo.ini").clicked() {
                            self.open(&Path::new(&self.settings.data).join("diablo.ini"));
                        }
                        if ui.button("Registro do jogo").clicked() {
                            self.open(&log_path());
                        }
                    });
                    if !self.message.is_empty() {
                        ui.add_space(8.0);
                        ui.colored_label(
                            if self.error {
                                Color32::from_rgb(245, 130, 120)
                            } else {
                                GOLD
                            },
                            &self.message,
                        );
                    }
                    if !self.logs.is_empty() {
                        egui::CollapsingHeader::new("Última execução")
                            .default_open(self.error)
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(160.0)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::Label::new(RichText::new(&self.logs).monospace())
                                                .selectable(true),
                                        );
                                    });
                            });
                    }
                    ui.add_space(20.0);
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        ui.small("Devilution Launcher · Rust + egui");
                        ui.hyperlink_to("Obter os arquivos do jogo", "https://devilutionx.com/mpq");
                    });
                });
            });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Devilution Launcher",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([700.0, 640.0])
                .with_min_inner_size([540.0, 500.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Launcher::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = env::temp_dir().join(format!(
                "devilution-test-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(p.join("dados com espaços")).unwrap();
            fs::write(p.join("motor"), "fixture").unwrap();
            Self(p)
        }
        fn settings(&self) -> Settings {
            Settings {
                executable: self.0.join("motor").display().to_string(),
                data: self.0.join("dados com espaços").display().to_string(),
                skip_intro: true,
                ..Settings::default()
            }
        }
        fn file(&self, name: &str) {
            fs::write(self.0.join("dados com espaços").join(name), b"MPQ").unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn validates_diablo_and_hellfire_archives() {
        let fixture = Fixture::new();
        let mut settings = fixture.settings();
        assert!(settings.validate().unwrap_err().contains("diabdat.mpq"));
        fixture.file("DIABDAT.MPQ");
        assert!(settings.validate().is_ok());
        settings.game = Game::Hellfire;
        assert!(settings.validate().unwrap_err().contains("hellfire.mpq"));
        for name in ["HELLFIRE.MPQ", "hfmonk.mpq", "hfmusic.mpq", "hfvoice.mpq"] {
            fixture.file(name);
        }
        assert!(settings.validate().is_ok());
        assert!(
            settings
                .command()
                .unwrap()
                .get_args()
                .any(|a| a == "--hellfire")
        );
        fs::write(Path::new(&settings.data).join("hfvoice.mpq"), []).unwrap();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn command_preserves_paths_and_selects_game() {
        let fixture = Fixture::new();
        fixture.file("DIABDAT.MPQ");
        let mut settings = fixture.settings();
        let command = settings.command().unwrap();
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            [
                "--data-dir",
                &settings.data,
                "--save-dir",
                &settings.data,
                "--config-dir",
                &settings.data,
                "--diablo",
                "-n"
            ]
        );
        assert_eq!(command.get_current_dir(), Some(fixture.0.as_path()));
        settings.skip_intro = false;
        assert!(!settings.command().unwrap().get_args().any(|a| a == "-n"));
    }

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
            data: "/dados com espaços".into(),
            game: Game::Hellfire,
            ..Settings::default()
        };
        eframe::set_value(&mut storage, "settings", &settings);
        let restored: Settings = eframe::get_value(&storage, "settings").unwrap();
        assert_eq!(restored.data, settings.data);
        assert_eq!(restored.game, Game::Hellfire);
    }
}
