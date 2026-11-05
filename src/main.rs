mod game_config;

use eframe::egui::{self, Color32, RichText, Vec2};
use game_config::GameConfig;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

const GOLD: Color32 = Color32::from_rgb(225, 166, 92);
const BACKGROUND: Color32 = Color32::from_rgb(14, 15, 18);

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Play,
    Settings,
}
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
        // O instalador Mint separado usa ~/.local/share mesmo com XDG personalizado.
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
    tab: Tab,
    config: Result<GameConfig, String>,
    hero: egui::TextureHandle,
    logo: egui::TextureHandle,
}

impl Launcher {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BACKGROUND;
        visuals.override_text_color = Some(Color32::from_rgb(230, 229, 226));
        visuals.window_fill = Color32::from_rgb(22, 23, 27);
        visuals.selection.bg_fill = Color32::from_rgb(75, 53, 35);
        visuals.widgets.noninteractive.bg_stroke.color = Color32::from_rgb(45, 45, 49);
        cc.egui_ctx.set_visuals(visuals);
        cc.egui_ctx.style_mut(|s| {
            s.spacing.item_spacing = Vec2::new(12.0, 10.0);
            s.spacing.button_padding = Vec2::new(16.0, 10.0);
        });
        let settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, "settings"))
            .unwrap_or_else(Settings::detected);
        let config = GameConfig::load(Path::new(&settings.data).join("diablo.ini"));
        Self {
            hero: load_texture(
                &cc.egui_ctx,
                "diablo-art",
                include_bytes!("../assets/diablo-hero.jpg"),
            ),
            logo: load_texture(
                &cc.egui_ctx,
                "devilution-logo",
                include_bytes!("../assets/devilutionx-logo.png"),
            ),
            tab: Tab::Play,
            config,
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
        if self.config.as_ref().is_ok_and(GameConfig::dirty) {
            return Err("Salve ou recarregue as configurações antes de jogar.".into());
        }
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
                self.reload_config();
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

fn load_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let image = image::load_from_memory(bytes)
        .expect("Imagem incorporada inválida")
        .to_rgba8();
    ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.as_raw(),
        ),
        egui::TextureOptions::LINEAR,
    )
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

impl Launcher {
    fn reload_config(&mut self) {
        self.config = GameConfig::load(Path::new(&self.settings.data).join("diablo.ini"));
    }

    fn play_ui(&mut self, ui: &mut egui::Ui) {
        let height = (ui.available_height() - 110.0).max(220.0);
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );
        let image_aspect = self.hero.size_vec2().x / self.hero.size_vec2().y;
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
        ui.painter().image(self.hero.id(), rect, uv, Color32::WHITE);
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
            "O clássico. Uma nova jornada.",
            egui::FontId::proportional(15.0),
            MUTED,
        );

        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(32, 14))
            .show(ui, |ui| {
                let ready = self.settings.validate();
                let dirty = self.config.as_ref().is_ok_and(GameConfig::dirty);
                let running = self.child.is_some();
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(8.0);
                        let (status, color) = if running {
                            ("Em jogo", GOLD)
                        } else if dirty {
                            ("Configurações não salvas", GOLD)
                        } else if ready.is_ok() {
                            ("Pronto para jogar", Color32::from_rgb(133, 183, 150))
                        } else {
                            ("Instalação incompleta", GOLD)
                        };
                        ui.label(RichText::new(status).size(13.0).color(color));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !running && ready.is_ok() && !dirty,
                                egui::Button::new(
                                    RichText::new(if running { "EM JOGO" } else { "JOGAR" })
                                        .strong()
                                        .size(18.0)
                                        .color(BACKGROUND),
                                )
                                .fill(GOLD)
                                .corner_radius(8)
                                .min_size(Vec2::new(148.0, 48.0)),
                            )
                            .clicked()
                        {
                            let result = self.launch();
                            self.report(result, "");
                            ui.ctx().request_repaint_after(Duration::from_millis(500));
                        }
                    });
                });
                if !self.message.is_empty() && self.error {
                    ui.colored_label(Color32::from_rgb(245, 139, 128), &self.message);
                }
            });
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let running = self.child.is_some();
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(32, 20))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Configurações");
                    ui.label(
                        RichText::new(if running {
                            "Feche o jogo para editar as opções."
                        } else {
                            "Ajuste o jogo ao seu jeito."
                        })
                        .color(MUTED),
                    );
                    ui.add_space(14.0);
                    let mut save = false;
                    let mut reload = false;
                    ui.add_enabled_ui(!running, |ui| {
                        if let Ok(config) = &mut self.config {
                            let options = &mut config.options;
                            section(ui, "Vídeo", |ui| {
                                egui::Grid::new("video-options")
                                    .num_columns(2)
                                    .spacing([24.0, 14.0])
                                    .show(ui, |ui| {
                                        ui.label("Resolução");
                                        egui::ComboBox::from_id_salt("resolution")
                                            .selected_text(format!(
                                                "{} × {}",
                                                options.width, options.height
                                            ))
                                            .show_ui(ui, |ui| {
                                                for (width, height) in [
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
                                                            (options.width, options.height)
                                                                == (width, height),
                                                            format!("{width} × {height}"),
                                                        )
                                                        .clicked()
                                                    {
                                                        options.width = width;
                                                        options.height = height;
                                                    }
                                                }
                                            });
                                        ui.end_row();
                                        ui.label("Exibição");
                                        ui.checkbox(&mut options.fullscreen, "Tela cheia");
                                        ui.end_row();
                                        ui.label("Escala");
                                        ui.checkbox(
                                            &mut options.upscale,
                                            "Ajustar imagem à janela",
                                        );
                                        ui.end_row();
                                        ui.label("Quadros por segundo");
                                        egui::ComboBox::from_id_salt("frames")
                                            .selected_text(match options.frame_control {
                                                0 => "Sem limite",
                                                1 => "Sincronização vertical",
                                                2 => "Limitar FPS",
                                                _ => "Personalizado",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut options.frame_control,
                                                    0,
                                                    "Sem limite",
                                                );
                                                ui.selectable_value(
                                                    &mut options.frame_control,
                                                    1,
                                                    "Sincronização vertical",
                                                );
                                                ui.selectable_value(
                                                    &mut options.frame_control,
                                                    2,
                                                    "Limitar FPS",
                                                );
                                            });
                                        ui.end_row();
                                    });
                            });
                            section(ui, "Áudio", |ui| {
                                egui::Grid::new("audio-options").spacing([24.0, 14.0]).show(
                                    ui,
                                    |ui| {
                                        ui.label("Música");
                                        ui.add(
                                            egui::Slider::new(&mut options.music, 0..=100)
                                                .suffix(" %"),
                                        );
                                        ui.end_row();
                                        ui.label("Efeitos sonoros");
                                        ui.add(
                                            egui::Slider::new(&mut options.sound, 0..=100)
                                                .suffix(" %"),
                                        );
                                        ui.end_row();
                                    },
                                );
                            });
                            section(ui, "Jogo", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Idioma");
                                    egui::ComboBox::from_id_salt("language")
                                        .selected_text(match options.language.as_str() {
                                            "pt_BR" => "Português (Brasil)",
                                            "en" => "English",
                                            "es" => "Español",
                                            other => other,
                                        })
                                        .show_ui(ui, |ui| {
                                            for (code, label) in [
                                                ("pt_BR", "Português (Brasil)"),
                                                ("en", "English"),
                                                ("es", "Español"),
                                            ] {
                                                ui.selectable_value(
                                                    &mut options.language,
                                                    code.into(),
                                                    label,
                                                );
                                            }
                                        });
                                });
                                ui.checkbox(
                                    &mut self.settings.skip_intro,
                                    "Pular vídeos de abertura",
                                );
                                ui.checkbox(&mut options.run_in_town, "Correr na cidade");
                                ui.checkbox(&mut options.auto_gold, "Coletar ouro automaticamente");
                                ui.checkbox(&mut options.quick_cast, "Lançar magias pelos atalhos");
                            });
                        } else if let Err(error) = &self.config {
                            ui.colored_label(GOLD, error);
                        }
                        egui::CollapsingHeader::new("Avançado").show(ui, |ui| {
                            ui.label("Executável do DevilutionX");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.settings.executable)
                                    .desired_width(f32::INFINITY),
                            );
                            ui.label("Pasta dos dados e saves");
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut self.settings.data)
                                        .desired_width(f32::INFINITY),
                                )
                                .lost_focus()
                                && self.config.as_ref().is_ok_and(|c| {
                                    c.path != Path::new(&self.settings.data).join("diablo.ini")
                                })
                            {
                                reload = true;
                            }
                            if ui.button("Detectar instalação").clicked() {
                                let detected = Settings::detected();
                                self.settings.executable = detected.executable;
                                self.settings.data = detected.data;
                                reload = true;
                            }
                            if let Ok(config) = &mut self.config {
                                ui.checkbox(&mut config.options.show_fps, "Mostrar FPS no jogo");
                            }
                            let mut hellfire = self.settings.game == Game::Hellfire;
                            if ui
                                .checkbox(&mut hellfire, "Usar expansão Hellfire")
                                .changed()
                            {
                                self.settings.game = if hellfire {
                                    Game::Hellfire
                                } else {
                                    Game::Diablo
                                };
                            }
                            if hellfire {
                                ui.small(
                                    "Requer os quatro arquivos MPQ da expansão na pasta dos dados.",
                                );
                            }
                            if let Err(error) = self.settings.validate() {
                                ui.colored_label(GOLD, error);
                            }
                        });
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            save = ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Salvar alterações").color(BACKGROUND),
                                    )
                                    .fill(GOLD)
                                    .corner_radius(6),
                                )
                                .clicked();
                            reload |= ui.button("Recarregar").clicked();
                        });
                    });
                    if reload {
                        self.reload_config();
                        self.report(Ok(()), "Configurações recarregadas.");
                    }
                    if save {
                        let expected = Path::new(&self.settings.data).join("diablo.ini");
                        let result = match &mut self.config {
                            Ok(config) if config.path == expected => config.save(),
                            Ok(_) => Err(
                                "A pasta dos dados mudou. Clique em Recarregar antes de salvar."
                                    .into(),
                            ),
                            Err(error) => Err(error.clone()),
                        };
                        if result.is_ok()
                            && let Some(storage) = frame.storage_mut()
                        {
                            eframe::set_value(storage, "settings", &self.settings);
                            storage.flush();
                        }
                        self.report(result, "Configurações salvas.");
                    }
                    if !self.message.is_empty() {
                        ui.add_space(8.0);
                        ui.colored_label(
                            if self.error {
                                Color32::from_rgb(245, 139, 128)
                            } else {
                                GOLD
                            },
                            &self.message,
                        );
                    }
                    ui.add_space(14.0);
                    egui::CollapsingHeader::new("Arquivos e diagnóstico").show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Dados e saves").clicked() {
                                self.open(&PathBuf::from(&self.settings.data));
                            }
                            if ui.button("diablo.ini").clicked() {
                                self.open(&Path::new(&self.settings.data).join("diablo.ini"));
                            }
                            if ui.button("Registro").clicked() {
                                self.open(&log_path());
                            }
                        });
                        ui.hyperlink_to(
                            "Como obter os arquivos do jogo",
                            "https://devilutionx.com/mpq",
                        );
                        if !self.logs.is_empty() {
                            egui::ScrollArea::vertical()
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::Label::new(RichText::new(&self.logs).monospace())
                                            .selectable(true),
                                    );
                                });
                        }
                    });
                });
            });
    }
}

fn section(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::from_rgb(22, 23, 27))
        .corner_radius(10)
        .inner_margin(18.0)
        .show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 2.0).max(0.0));
            ui.label(RichText::new(title).strong().size(15.0));
            ui.add_space(6.0);
            body(ui);
        });
    ui.add_space(8.0);
}

impl eframe::App for Launcher {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "settings", &self.settings);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.poll(ctx);
        egui::TopBottomPanel::top("navigation")
            .frame(
                egui::Frame::new()
                    .fill(BACKGROUND)
                    .inner_margin(egui::Margin::symmetric(24, 16)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.image((self.logo.id(), Vec2::splat(28.0)));
                    if ui.available_width() > 480.0 {
                        ui.label(RichText::new("DEVILUTIONX").strong().size(15.0));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .selectable_label(self.tab == Tab::Settings, "Configurações")
                            .clicked()
                        {
                            if self.tab != Tab::Settings
                                && !self.config.as_ref().is_ok_and(GameConfig::dirty)
                            {
                                self.reload_config();
                            }
                            self.tab = Tab::Settings;
                        }
                        if ui
                            .selectable_label(self.tab == Tab::Play, "Jogar")
                            .clicked()
                        {
                            self.tab = Tab::Play;
                        }
                    });
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND))
            .show(ctx, |ui| match self.tab {
                Tab::Play => self.play_ui(ui),
                Tab::Settings => self.settings_ui(ui, frame),
            });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Devilution Launcher",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_icon(
                    eframe::icon_data::from_png_bytes(include_bytes!(
                        "../assets/devilutionx-logo.png"
                    ))
                    .expect("Ícone incorporado inválido"),
                )
                .with_inner_size([1000.0, 680.0])
                .with_min_inner_size([620.0, 540.0]),
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
