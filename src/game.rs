use crate::paths::{data_home, home};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path, process::Command};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) enum Game {
    #[default]
    Diablo,
    Hellfire,
    DiabloII,
}

impl Game {
    pub(crate) const ALL: [Self; 3] = [Self::Diablo, Self::Hellfire, Self::DiabloII];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Diablo => "Diablo",
            Self::Hellfire => "Hellfire",
            Self::DiabloII => "Diablo II",
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub(crate) executable: String,
    pub(crate) data: String,
    pub(crate) game: Game,
    pub(crate) skip_intro: bool,
    pub(crate) diablo2: crate::diablo2::Installation,
}

impl Settings {
    pub(crate) fn detected() -> Self {
        // The separate Mint installer uses ~/.local/share even with a custom XDG path.
        let local = home().join(".local/share/devilutionx/devilutionx");
        let executable = [local, data_home().join("devilutionx/devilutionx")]
            .into_iter()
            .chain(
                env::split_paths(&env::var_os("PATH").unwrap_or_default())
                    .map(|p| p.join("devilutionx")),
            )
            .find(|p| p.is_file())
            .unwrap_or_else(|| data_home().join("devilutionx/devilutionx"));
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

    pub(crate) fn has_data(&self, game: Game) -> bool {
        if game == Game::DiabloII {
            return crate::diablo2::has_data(Path::new(&self.diablo2.data));
        }
        let data = Path::new(&self.data);
        has_file(data, "diabdat.mpq")
            && (game == Game::Diablo
                || ["hellfire.mpq", "hfmonk.mpq", "hfmusic.mpq", "hfvoice.mpq"]
                    .iter()
                    .all(|name| has_file(data, name)))
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.game == Game::DiabloII {
            return self.diablo2.validate();
        }
        if !Path::new(&self.executable).is_file() {
            return Err("Choose your DevilutionX executable in Settings.".into());
        }
        let data = Path::new(&self.data);
        if !data.is_dir() {
            return Err("The game data folder was not found. Check Settings.".into());
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
                "Missing files in the game data folder: {}",
                missing.join(", ")
            ))
        }
    }

    pub(crate) fn command(&self) -> Result<Command, String> {
        if self.game == Game::DiabloII {
            return self.diablo2.command();
        }
        self.validate()?;
        let executable = fs::canonicalize(&self.executable).map_err(|e| e.to_string())?;
        let data = fs::canonicalize(&self.data).map_err(|e| e.to_string())?;
        let directory = executable
            .parent()
            .ok_or("The executable has no parent folder")?;
        let mut command = Command::new(&executable);
        command
            .current_dir(directory)
            .arg("--data-dir")
            .arg(&data)
            .arg("--save-dir")
            .arg(&data)
            .arg("--config-dir")
            .arg(&data)
            .arg(if self.game == Game::Hellfire {
                "--hellfire"
            } else {
                "--diablo"
            });
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

    pub(crate) fn data_for(&self, game: Game) -> &str {
        if game == Game::DiabloII {
            &self.diablo2.data
        } else {
            &self.data
        }
    }

    pub(crate) fn executable_for(&self, game: Game) -> &str {
        if game == Game::DiabloII {
            &self.diablo2.executable
        } else {
            &self.executable
        }
    }
}

pub(crate) fn has_file(directory: &Path, name: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
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
            fs::create_dir_all(p.join("data with spaces")).unwrap();
            fs::write(p.join("motor"), "fixture").unwrap();
            Self(p)
        }
        fn settings(&self) -> Settings {
            Settings {
                executable: self.0.join("motor").display().to_string(),
                data: self.0.join("data with spaces").display().to_string(),
                skip_intro: true,
                ..Settings::default()
            }
        }
        fn file(&self, name: &str) {
            fs::write(self.0.join("data with spaces").join(name), b"MPQ").unwrap();
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
        assert!(settings.has_data(Game::Diablo));
        assert!(!settings.has_data(Game::Hellfire));
        assert!(settings.validate().is_ok());
        settings.game = Game::Hellfire;
        assert!(settings.validate().unwrap_err().contains("hellfire.mpq"));
        for name in ["HELLFIRE.MPQ", "hfmonk.mpq", "hfmusic.mpq", "hfvoice.mpq"] {
            fixture.file(name);
        }
        assert!(settings.validate().is_ok());
        assert!(settings.has_data(Game::Hellfire));
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
}

#[cfg(test)]
mod diablo2_tests {
    use super::*;
    #[test]
    fn diablo2_uses_its_own_data_flags_and_toggles() {
        let root = env::temp_dir().join(format!(
            "d2-command-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let data = root.join("D2 data with spaces");
        fs::create_dir_all(&data).unwrap();
        fs::write(root.join("d2d"), b"engine").unwrap();
        let mut settings = Settings {
            game: Game::DiabloII,
            ..Settings::default()
        };
        settings.diablo2.data = data.display().to_string();
        settings.diablo2.executable = root.join("d2d").display().to_string();
        assert!(settings.validate().unwrap_err().contains("d2data.mpq"));
        for name in crate::diablo2::ARCHIVES {
            fs::write(data.join(name.to_uppercase()), b"MPQ\x1a").unwrap();
        }
        assert!(settings.has_data(Game::DiabloII));
        assert!(!settings.has_data(Game::Diablo));
        assert!(!settings.has_data(Game::Hellfire));
        settings.diablo2.toggles.insert("autoloot".into(), false);
        let command = settings.command().unwrap();
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args[0], "--data");
        assert_eq!(args[1], settings.diablo2.data);
        assert_eq!(args[2], "--toggle");
        assert!(args[3].contains("autoloot=off"));
        assert!(
            !args
                .iter()
                .any(|a| matches!(a.as_str(), "--data-dir" | "--diablo" | "--hellfire"))
        );
        assert_eq!(command.get_current_dir(), Some(root.as_path()));
        fs::remove_dir_all(root).unwrap();
    }
}
