use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq)]
pub struct GameOptions {
    pub width: i32,
    pub height: i32,
    pub fullscreen: bool,
    pub upscale: bool,
    pub frame_control: i32,
    pub music: i32,
    pub sound: i32,
    pub language: String,
    pub run_in_town: bool,
    pub auto_gold: bool,
    pub quick_cast: bool,
    pub show_fps: bool,
}

impl GameOptions {
    fn from_ini(text: &str) -> Self {
        let int = |section, key, default| {
            value(text, section, key)
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(default)
        };
        Self {
            width: int("Graphics", "Width", 640),
            height: int("Graphics", "Height", 480),
            fullscreen: int("Graphics", "Fullscreen", 1) != 0,
            upscale: int("Graphics", "Upscale", 1) != 0,
            frame_control: int("Graphics", "Frame Rate Control", 1),
            music: ((int("Audio", "Music Volume", 0).clamp(-1600, 0) + 1600) * 100 + 800) / 1600,
            sound: ((int("Audio", "Sound Volume", 0).clamp(-1600, 0) + 1600) * 100 + 800) / 1600,
            language: value(text, "Language", "Code").unwrap_or("en").into(),
            run_in_town: int("Game", "Run in Town", 0) != 0,
            auto_gold: int("Game", "Auto Gold Pickup", 0) != 0,
            quick_cast: int("Game", "Quick Cast", 0) != 0,
            show_fps: int("Graphics", "Show FPS", 0) != 0,
        }
    }

    fn entries(&self) -> Vec<(&'static str, &'static str, String)> {
        vec![
            ("Graphics", "Width", self.width.to_string()),
            ("Graphics", "Height", self.height.to_string()),
            (
                "Graphics",
                "Fullscreen",
                i32::from(self.fullscreen).to_string(),
            ),
            ("Graphics", "Upscale", i32::from(self.upscale).to_string()),
            (
                "Graphics",
                "Frame Rate Control",
                self.frame_control.to_string(),
            ),
            ("Graphics", "Show FPS", i32::from(self.show_fps).to_string()),
            (
                "Audio",
                "Music Volume",
                ((self.music.clamp(0, 100) - 100) * 16).to_string(),
            ),
            (
                "Audio",
                "Sound Volume",
                ((self.sound.clamp(0, 100) - 100) * 16).to_string(),
            ),
            ("Language", "Code", self.language.clone()),
            (
                "Game",
                "Run in Town",
                i32::from(self.run_in_town).to_string(),
            ),
            (
                "Game",
                "Auto Gold Pickup",
                i32::from(self.auto_gold).to_string(),
            ),
            ("Game", "Quick Cast", i32::from(self.quick_cast).to_string()),
        ]
    }
}

fn value<'a>(text: &'a str, section: &str, key: &str) -> Option<&'a str> {
    let mut active = false;
    let mut result = None;
    for line in text.lines().map(str::trim) {
        if line.starts_with([';', '#']) {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            active = name.trim().eq_ignore_ascii_case(section);
        } else if active
            && let Some((name, val)) = line.split_once('=')
            && name.trim().eq_ignore_ascii_case(key)
        {
            result = Some(val.trim());
        }
    }
    result
}

// Só substitui a chave editada; comentários e opções desconhecidas permanecem.
fn set_value(text: &str, section: &str, key: &str, val: &str) -> String {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let mut active = false;
    let mut insertion = None;
    let mut found_section = false;
    let mut replaced = false;
    for (index, line) in lines.iter_mut().enumerate() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            if active {
                insertion = Some(index);
            }
            active = name.trim().eq_ignore_ascii_case(section);
            if active {
                found_section = true;
                insertion = Some(index + 1);
            }
        } else if active {
            insertion = Some(index + 1);
            if let Some((name, _)) = trimmed.split_once('=')
                && name.trim().eq_ignore_ascii_case(key)
            {
                *line = format!("{key}={val}");
                replaced = true;
            }
        }
    }
    if !replaced {
        if found_section {
            lines.insert(insertion.unwrap(), format!("{key}={val}"));
        } else {
            if !lines.is_empty() && !lines.last().unwrap().is_empty() {
                lines.push(String::new());
            }
            lines.extend([format!("[{section}]"), format!("{key}={val}")]);
        }
    }
    let mut result = lines.join(newline);
    if text.ends_with('\n') || text.is_empty() {
        result.push_str(newline);
    }
    result
}

fn read(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Não foi possível ler {}: {e}", path.display())),
    }
}

pub const DEFAULTS: &str = include_str!("../config/defaults-1.5.5.ini");
pub fn default_value(section: &str, key: &str) -> Option<&'static str> {
    value(DEFAULTS, section, key)
}

#[derive(Clone, Debug)]
pub struct ExtraOption {
    pub section: String,
    pub key: String,
    pub value: String,
    original: String,
}

impl ExtraOption {
    pub fn changed(&self) -> bool {
        self.value != self.original
    }

    pub fn hellfire_only(&self) -> bool {
        matches!(
            self.key.as_str(),
            "Hellfire Intro"
                | "Alternate nest art"
                | "Theo Quest"
                | "Cow Quest"
                | "Test Bard"
                | "Test Barbarian"
                | "Auto Oil Pickup"
        )
    }
}

fn ini_entries(text: &str) -> Vec<(String, String, String)> {
    let mut section = String::new();
    let mut entries: Vec<(String, String, String)> = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with([';', '#']) {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = name.trim().into();
        } else if !section.is_empty()
            && let Some((key, val)) = line.split_once('=')
        {
            let key = key.trim();
            if let Some(existing) = entries
                .iter_mut()
                .find(|(s, k, _)| s.eq_ignore_ascii_case(&section) && k.eq_ignore_ascii_case(key))
            {
                existing.2 = val.trim().into();
            } else {
                entries.push((section.clone(), key.into(), val.trim().into()));
            }
        }
    }
    entries
}

fn extra_options(text: &str, primary: &GameOptions) -> Vec<ExtraOption> {
    let covered = primary.entries();
    let mut entries = ini_entries(DEFAULTS);
    for (section, key, val) in ini_entries(text) {
        if let Some(entry) = entries
            .iter_mut()
            .find(|(s, k, _)| s.eq_ignore_ascii_case(&section) && k.eq_ignore_ascii_case(&key))
        {
            entry.2 = val;
        } else {
            entries.push((section, key, val));
        }
    }
    entries
        .into_iter()
        .filter(|(section, key, _)| {
            !matches!(section.as_str(), "Diablo" | "Hellfire")
                && !(section == "GameMode" && key == "Game")
                && !covered
                    .iter()
                    .any(|(s, k, _)| section.eq_ignore_ascii_case(s) && key.eq_ignore_ascii_case(k))
        })
        .map(|(section, key, value)| ExtraOption {
            section,
            key,
            original: value.clone(),
            value,
        })
        .collect()
}

pub struct GameConfig {
    pub path: PathBuf,
    pub options: GameOptions,
    pub extra: Vec<ExtraOption>,
    original: Option<String>,
    loaded: GameOptions,
}

impl GameConfig {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let original = read(&path)?;
        let options = GameOptions::from_ini(original.as_deref().unwrap_or(""));
        let extra = extra_options(original.as_deref().unwrap_or(""), &options);
        Ok(Self {
            path,
            extra,
            loaded: options.clone(),
            options,
            original,
        })
    }

    pub fn dirty(&self) -> bool {
        self.options != self.loaded || self.extra.iter().any(ExtraOption::changed)
    }

    pub fn save(&mut self) -> Result<(), String> {
        if !self.dirty() {
            return Ok(());
        }
        if !(640..=16384).contains(&self.options.width)
            || !(480..=16384).contains(&self.options.height)
        {
            return Err("Resolução fora do intervalo permitido (640×480 a 16384×16384).".into());
        }
        if read(&self.path)? != self.original {
            return Err(
                "O diablo.ini foi alterado fora do launcher. Clique em Recarregar antes de salvar."
                    .into(),
            );
        }
        let mut text = self.original.clone().unwrap_or_default();
        for ((section, key, val), (_, _, old)) in self
            .options
            .entries()
            .into_iter()
            .zip(self.loaded.entries())
        {
            if val != old {
                text = set_value(&text, section, key, &val);
            }
        }
        for option in self.extra.iter().filter(|option| option.changed()) {
            if option.value.contains(['\n', '\r', '\0']) {
                return Err(format!(
                    "{}: o valor deve ocupar uma única linha.",
                    option.key
                ));
            }
            crate::option_catalog::validate(option)?;
            text = set_value(&text, &option.section, &option.key, &option.value);
        }
        let temp = self
            .path
            .with_file_name(format!(".diablo.ini.launcher-{}.tmp", std::process::id()));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("Não foi possível preparar a gravação: {e}"))?;
        let result = (|| {
            if self.original.is_some() {
                fs::copy(&self.path, self.path.with_extension("ini.launcher.bak"))?;
                file.set_permissions(fs::metadata(&self.path)?.permissions())?;
            }
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            fs::rename(&temp, &self.path)
        })();
        if let Err(e) = result {
            let _ = fs::remove_file(&temp);
            return Err(format!("Não foi possível salvar as configurações: {e}"));
        }
        self.original = Some(text);
        self.loaded = self.options.clone();
        for option in &mut self.extra {
            option.original = option.value.clone();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn patches_only_selected_keys_and_preserves_crlf() {
        let ini =
            "; comentário\r\n[Graphics]\r\nWidth=640\r\nCustom=keep\r\n[Other]\r\nWidth=5\r\n";
        assert_eq!(
            set_value(ini, "Graphics", "Width", "1280"),
            ini.replace("Width=640", "Width=1280")
        );
        let added = set_value(ini, "Audio", "Music Volume", "-1600");
        assert!(added.starts_with(ini));
        assert_eq!(value(&added, "Audio", "Music Volume"), Some("-1600"));
        assert_eq!(
            value(
                &set_value(ini, "Graphics", "Height", "720"),
                "Graphics",
                "Height"
            ),
            Some("720")
        );
        assert_eq!(value(&added, "Other", "Width"), Some("5"));
    }

    #[test]
    fn save_preserves_unknown_values_backs_up_and_rejects_conflicts() {
        let dir = std::env::temp_dir().join(format!(
            "devilution-ini-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("diablo.ini");
        let original =
            "[Audio]\nMusic Volume=-950\n[Graphics]\nFullscreen=1\n[Keymapping]\nInventory=I\n";
        fs::write(&path, original).unwrap();
        let mut config = GameConfig::load(path.clone()).unwrap();
        config.options.fullscreen = false;
        config.save().unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            original.replace("Fullscreen=1", "Fullscreen=0")
        );
        assert_eq!(
            fs::read_to_string(path.with_extension("ini.launcher.bak")).unwrap(),
            original
        );
        assert!(!config.dirty());
        config.options.music = 0;
        config.save().unwrap();
        assert_eq!(
            value(&fs::read_to_string(&path).unwrap(), "Audio", "Music Volume"),
            Some("-1600")
        );
        config.options.sound = 50;
        fs::write(&path, "external edit").unwrap();
        assert!(config.save().unwrap_err().contains("alterado fora"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "external edit");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn catalog_covers_all_linux_options_and_defaults_are_valid() {
        let options = GameOptions::from_ini(DEFAULTS);
        let extras = extra_options(DEFAULTS, &options);
        let primary = options.entries();
        let keys = ini_entries(DEFAULTS);
        assert_eq!(keys.len(), 190);
        assert_eq!(primary.len() + extras.len(), keys.len());
        for (section, key, _) in keys {
            assert!(
                primary.iter().any(|(s, k, _)| *s == section && *k == key)
                    || extras.iter().any(|o| o.section == section && o.key == key),
                "{section}.{key} ausente"
            );
        }
        for option in &extras {
            crate::option_catalog::validate(option).unwrap();
        }
        assert!(extras.iter().filter(|o| o.section == "Keymapping").count() > 40);
        assert!(extras.iter().filter(|o| o.section == "Padmapping").count() > 50);
    }

    #[test]
    fn edits_bindings_and_new_options_without_touching_unrelated_keys() {
        let dir = std::env::temp_dir().join(format!(
            "devilution-all-options-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("diablo.ini");
        let original = "; keep\n[Keymapping]\nInventory=I\n[Controller]\ndeadzone=0.070000\n[Mod]\nFuture=old\n";
        fs::write(&path, original).unwrap();
        let mut config = GameConfig::load(path.clone()).unwrap();
        for (section, key, val) in [
            ("Keymapping", "Inventory", "TAB"),
            ("Controller", "deadzone", "0.15"),
            ("Mod", "Future", "new"),
        ] {
            config
                .extra
                .iter_mut()
                .find(|o| o.section == section && o.key == key)
                .unwrap()
                .value = val.into();
        }
        assert!(config.dirty());
        config.save().unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            original
                .replace("Inventory=I", "Inventory=TAB")
                .replace("deadzone=0.070000", "deadzone=0.15")
                .replace("Future=old", "Future=new")
        );
        assert!(!config.dirty());
        config
            .extra
            .iter_mut()
            .find(|o| o.key == "Port")
            .unwrap()
            .value = "70000".into();
        assert!(config.save().unwrap_err().contains("inválido"));
        config
            .extra
            .iter_mut()
            .find(|o| o.key == "Port")
            .unwrap()
            .value = "6112".into();
        config
            .extra
            .iter_mut()
            .find(|o| o.key == "Future")
            .unwrap()
            .value = "x\nInjected=yes".into();
        assert!(config.save().unwrap_err().contains("única linha"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn volume_endpoints_and_new_config_defaults() {
        let options = GameOptions::from_ini("[Audio]\nMusic Volume=-1600\nSound Volume=0\n");
        assert_eq!((options.music, options.sound), (0, 100));
        assert_eq!(options.language, "en");
        assert_eq!(
            value(
                &set_value("", "Language", "Code", "pt_BR"),
                "Language",
                "Code"
            ),
            Some("pt_BR")
        );
    }
}
