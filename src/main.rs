mod game_config;
mod option_catalog;
mod options_ui;

use eframe::egui::{self, Color32, RichText, Vec2};
use game_config::GameConfig;
use option_catalog::Category;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

const GOLD: Color32 = Color32::from_rgb(225, 166, 92);
const PRIMARY: Color32 = Color32::from_rgb(151, 39, 48);
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
    wordmark: egui::TextureHandle,
    category: Category,
}

impl Launcher {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BACKGROUND;
        visuals.override_text_color = Some(Color32::from_rgb(230, 229, 226));
        visuals.window_fill = Color32::from_rgb(22, 23, 27);
        visuals.selection.bg_fill = Color32::from_rgb(76, 36, 43);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(225, 122, 126));
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.corner_radius = egui::CornerRadius::same(6);
            widget.fg_stroke.color = Color32::from_rgb(232, 230, 226);
        }
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(31, 33, 40);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(31, 33, 40);
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0_f32, Color32::from_rgb(65, 65, 74));
        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(47, 48, 57);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(47, 48, 57);
        visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.0_f32, Color32::from_rgb(150, 76, 83));
        visuals.widgets.active.weak_bg_fill = Color32::from_rgb(69, 37, 44);
        visuals.widgets.active.bg_fill = Color32::from_rgb(69, 37, 44);
        visuals.widgets.active.bg_stroke =
            egui::Stroke::new(1.0_f32, Color32::from_rgb(203, 100, 105));
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
            wordmark: load_texture(
                &cc.egui_ctx,
                "devilution-wordmark",
                include_bytes!("../assets/devilutionx-wordmark.png"),
            ),
            category: Category::Video,
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
        gradient(
            ui.painter(),
            egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.right(), rect.top() + (rect.height() * 0.3).min(120.0)),
            ),
            BACKGROUND,
            Color32::TRANSPARENT,
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
                        if action_button(
                            ui,
                            if running { "EM JOGO" } else { "JOGAR" },
                            !running && ready.is_ok() && !dirty,
                            Vec2::new(148.0, 48.0),
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
        egui::Frame::new().inner_margin(egui::Margin::symmetric(28,16)).show(ui,|ui| {
            ui.heading("Configurações");
            if running { ui.colored_label(GOLD,"Feche o jogo para editar as opções."); }
            ui.horizontal_wrapped(|ui| {
                for category in Category::ALL {
                    if category==Category::Other && !self.config.as_ref().is_ok_and(|c|
                        c.extra.iter().any(|o|option_catalog::category(o)==Category::Other && o.section!="GameMode")) { continue; }
                    if ui.selectable_label(self.category==category,category.title()).clicked() {
                        if self.category!=category { egui::Popup::close_all(ui.ctx()); }
                        self.category=category;
                    }
                }
            });
            ui.add_space(8.0);
            let mut reload=false;
            egui::ScrollArea::vertical().id_salt(self.category).auto_shrink([false,false])
                .max_height((ui.available_height()-82.0).max(100.0)).show(ui,|ui| {
                ui.add_enabled_ui(!running,|ui| {
                    match &mut self.config {
                        Ok(config)=>{
                            egui::Frame::new().fill(Color32::from_rgb(22,23,28)).corner_radius(10)
                                .inner_margin(18.0).show(ui,|ui| {
                                ui.set_min_width((ui.available_width()-2.0).max(0.0));
                                options_ui::game_options(ui,config,self.category,&mut self.settings.skip_intro);
                            });
                        }
                        Err(error)=>{ui.colored_label(GOLD,error);}
                    }
                    ui.add_space(16.0);
                    egui::CollapsingHeader::new("Avançado").show(ui,|ui| {
                        ui.label("Executável do DevilutionX");
                        ui.add(egui::TextEdit::singleline(&mut self.settings.executable).desired_width(f32::INFINITY));
                        ui.label("Pasta dos dados e saves");
                        if ui.add(egui::TextEdit::singleline(&mut self.settings.data).desired_width(f32::INFINITY)).lost_focus()
                            && self.config.as_ref().is_ok_and(|c|c.path!=Path::new(&self.settings.data).join("diablo.ini")) { reload=true; }
                        if ui.button("Detectar instalação").clicked() {
                            let detected=Settings::detected();
                            self.settings.executable=detected.executable;self.settings.data=detected.data;reload=true;
                        }
                        let mut hellfire=self.settings.game==Game::Hellfire;
                        if ui.checkbox(&mut hellfire,"Usar expansão Hellfire").changed() {
                            self.settings.game=if hellfire {Game::Hellfire} else {Game::Diablo};
                        }
                        if hellfire {ui.small("Requer os quatro arquivos MPQ da expansão na pasta dos dados.");}
                        if let Ok(config)=&mut self.config {
                            egui::CollapsingHeader::new("Opções da expansão e demonstração").show(ui,|ui| {
                                options_ui::expansion_options(ui, config, self.settings.skip_intro);
                            });
                        }
                        if let Err(error)=self.settings.validate() {ui.colored_label(GOLD,error);}
                    });
                });
                egui::CollapsingHeader::new("Arquivos e diagnóstico").show(ui,|ui| {
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Dados e saves").clicked() {self.open(&PathBuf::from(&self.settings.data));}
                        if ui.button("diablo.ini").clicked() {self.open(&Path::new(&self.settings.data).join("diablo.ini"));}
                        if ui.button("Registro").clicked() {self.open(&log_path());}
                    });
                    ui.hyperlink_to("Como obter os arquivos do jogo","https://devilutionx.com/mpq");
                    if !self.logs.is_empty() {
                        ui.add(egui::Label::new(RichText::new(&self.logs).monospace()).selectable(true));
                    }
                });
            });
            ui.add_space(10.0);
            let mut save=false;
            ui.horizontal(|ui| {
                save=action_button(ui,"Salvar alterações",!running,Vec2::new(162.0,36.0)).clicked();
                if ui.add_enabled(!running,egui::Button::new("Recarregar")).clicked() {reload=true;}
            });
            if reload {
                egui::Popup::close_all(ui.ctx());
                self.reload_config();self.report(Ok(()),"Configurações recarregadas.");
            }
            if save {
                let expected=Path::new(&self.settings.data).join("diablo.ini");
                let result=match &mut self.config {
                    Ok(config) if config.path==expected=>config.save(),
                    Ok(_)=>Err("A pasta dos dados mudou. Clique em Recarregar antes de salvar.".into()),
                    Err(error)=>Err(error.clone()),
                };
                if result.is_ok() && let Some(storage)=frame.storage_mut() {
                    eframe::set_value(storage,"settings",&self.settings);storage.flush();
                }
                self.report(result,"Configurações salvas.");
            }
            if !self.message.is_empty() {
                ui.colored_label(if self.error {Color32::from_rgb(245,139,128)} else {GOLD},&self.message);
            }
        });
    }
}

fn action_button(ui: &mut egui::Ui, label: &str, enabled: bool, size: Vec2) -> egui::Response {
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

impl eframe::App for Launcher {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "settings", &self.settings);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.poll(ctx);
        egui::TopBottomPanel::top("navigation")
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(BACKGROUND)
                    .inner_margin(egui::Margin::symmetric(24, 16)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (icon_slot, _) =
                        ui.allocate_exact_size(Vec2::new(28.0, 40.0), egui::Sense::hover());
                    ui.painter().image(
                        self.logo.id(),
                        egui::Rect::from_center_size(icon_slot.center(), Vec2::splat(28.0)),
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    let height = if ui.available_width() > 480.0 {
                        40.0
                    } else {
                        32.0
                    };
                    ui.image((
                        self.wordmark.id(),
                        Vec2::new(
                            height * self.wordmark.size_vec2().x / self.wordmark.size_vec2().y,
                            height,
                        ),
                    ));
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
                            egui::Popup::close_all(ui.ctx());
                            self.tab = Tab::Settings;
                        }
                        if ui
                            .selectable_label(self.tab == Tab::Play, "Jogar")
                            .clicked()
                        {
                            egui::Popup::close_all(ui.ctx());
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
