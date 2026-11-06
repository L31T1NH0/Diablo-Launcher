use crate::{
    game::has_file,
    paths::{config_home, data_home, home},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) const ARCHIVES: &[&str] = &[
    "d2data.mpq",
    "d2char.mpq",
    "d2sfx.mpq",
    "d2speech.mpq",
    "d2music.mpq",
    "d2video.mpq",
    "d2exp.mpq",
    "d2xtalk.mpq",
    "d2xmusic.mpq",
    "d2xvideo.mpq",
    "patch_d2.mpq",
];
pub(crate) const VERSION: &str = "0.9.0";
pub(crate) const TOGGLES: &[(&str, &str)] = &[
    ("autoloot", "Pick up gold automatically"),
    ("trans_roof", "See through nearby roofs"),
    ("autoparty", "Automatically join parties"),
    ("pad_controls", "Gamepad controls menu"),
    ("click_keys", "Extra mouse button bindings"),
    ("coop_lobby", "Local co-op character selection"),
    ("light_grid", "Lighting across the full view"),
    ("splash_centre", "Centered rain splashes"),
];

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Installation {
    pub(crate) executable: String,
    pub(crate) data: String,
    pub(crate) source: String,
    pub(crate) toggles: std::collections::BTreeMap<String, bool>,
}

impl Default for Installation {
    fn default() -> Self {
        let source = home().join("Games/Diablo-II/data");
        Self {
            executable: engine_dir().join("d2d").display().to_string(),
            data: data_home()
                .join("diablo-launcher/diablo2/data")
                .display()
                .to_string(),
            source: if source.is_dir() {
                source.display().to_string()
            } else {
                String::new()
            },
            toggles: Default::default(),
        }
    }
}

pub(crate) fn engine_dir() -> PathBuf {
    data_home().join("d2decomp").join(VERSION)
}

pub(crate) fn config_path() -> PathBuf {
    config_home().join("d2d/d2d.cfg")
}

pub(crate) fn has_data(data: &Path) -> bool {
    ARCHIVES.iter().all(|name| has_file(data, name))
}

impl Installation {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !Path::new(&self.executable).is_file() {
            return Err("Install D2Decomp, or choose its executable in Settings.".into());
        }
        let missing: Vec<_> = ARCHIVES
            .iter()
            .filter(|name| !has_file(Path::new(&self.data), name))
            .copied()
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "Import Diablo II + Lord of Destruction in Install. Missing: {}",
                missing.join(", ")
            ));
        }
        Ok(())
    }

    pub(crate) fn command(&self) -> Result<Command, String> {
        self.validate()?;
        let exe = fs::canonicalize(&self.executable).map_err(|e| e.to_string())?;
        let data = fs::canonicalize(&self.data).map_err(|e| e.to_string())?;
        let mut command = Command::new(&exe);
        command
            .current_dir(exe.parent().ok_or("D2Decomp has no parent folder")?)
            .arg("--data")
            .arg(data);
        let toggles: Vec<_> = TOGGLES
            .iter()
            .map(|(key, _)| {
                format!(
                    "{key}={}",
                    if self.toggles.get(*key).copied().unwrap_or(true) {
                        "on"
                    } else {
                        "off"
                    }
                )
            })
            .collect();
        command.arg("--toggle").arg(toggles.join(","));
        Ok(command)
    }
}
