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

pub struct GameConfig {
    pub path: PathBuf,
    pub options: GameOptions,
    original: Option<String>,
    loaded: GameOptions,
}

impl GameConfig {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let original = read(&path)?;
        let options = GameOptions::from_ini(original.as_deref().unwrap_or(""));
        Ok(Self {
            path,
            loaded: options.clone(),
            options,
            original,
        })
    }

    pub fn dirty(&self) -> bool {
        self.options != self.loaded
    }

    pub fn save(&mut self) -> Result<(), String> {
        if !self.dirty() {
            return Ok(());
        }
        if !(640..=7680).contains(&self.options.width)
            || !(480..=4320).contains(&self.options.height)
        {
            return Err("Resolução fora do intervalo permitido (640×480 a 7680×4320).".into());
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
