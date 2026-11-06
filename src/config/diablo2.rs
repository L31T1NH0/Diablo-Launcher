use super::file;
use std::{collections::BTreeMap, path::PathBuf};

pub(crate) struct D2Config {
    pub(crate) path: PathBuf,
    original: Option<String>,
    values: BTreeMap<String, String>,
    changes: BTreeMap<String, String>,
}

fn entry(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.starts_with(['#', ';']) {
        return None;
    }
    line.split_once('=').map(|(k, v)| (k.trim(), v.trim()))
}

impl D2Config {
    pub(crate) fn load(path: PathBuf) -> Result<Self, String> {
        let original = file::read(&path)?;
        let values = original
            .as_deref()
            .unwrap_or_default()
            .lines()
            .filter_map(entry)
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Ok(Self {
            path,
            original,
            values,
            changes: BTreeMap::new(),
        })
    }
    pub(crate) fn get<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.changes
            .get(key)
            .or_else(|| self.values.get(key))
            .map(String::as_str)
            .unwrap_or(default)
    }
    pub(crate) fn set(&mut self, key: &str, value: String) {
        if self.values.get(key) == Some(&value) {
            self.changes.remove(key);
        } else {
            self.changes.insert(key.into(), value);
        }
    }
    pub(crate) fn dirty(&self) -> bool {
        !self.changes.is_empty()
    }
    pub(crate) fn save(&mut self) -> Result<(), String> {
        if !self.dirty() {
            return Ok(());
        }
        for (key, value) in &self.changes {
            if value.contains(['\n', '\r', '\0']) || key.contains(['=', '\n', '\r', '\0']) {
                return Err("Settings must be single-line values.".into());
            }
            let valid = match key.as_str() {
                "fullscreen" | "widescreen" | "video" => matches!(value.as_str(), "0" | "1"),
                "scale" => value.parse::<u8>().is_ok_and(|n| (1..=8).contains(&n)),
                "master_volume" | "music_volume" => value.parse::<u8>().is_ok_and(|n| n <= 100),
                "scalemode" => matches!(value.as_str(), "pixelart" | "nearest" | "linear"),
                _ => true,
            };
            if !valid {
                return Err(format!("Invalid value for {key}."));
            }
        }
        let original = self.original.as_deref().unwrap_or_default();
        let newline = if original.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let mut pending = self.changes.clone();
        let mut text = String::new();
        for line in original.split_inclusive('\n') {
            if let Some((key, _)) = entry(line)
                && let Some(value) = self.changes.get(key)
            {
                text.push_str(&format!("{key} = {value}"));
                if line.ends_with('\n') {
                    text.push_str(newline);
                }
                pending.remove(key);
            } else {
                text.push_str(line);
            }
        }
        for (key, value) in pending {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push_str(newline);
            }
            text.push_str(&format!("{key} = {value}{newline}"));
        }
        file::save(&self.path, &self.original, &text)?;
        self.original = Some(text);
        self.values.append(&mut self.changes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };
    #[test]
    fn preserves_native_options_and_saves_rejects_conflicts_and_invalid_values() {
        let root = std::env::temp_dir().join(format!(
            "d2-config-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = root.join("d2d.cfg");
        fs::create_dir(&root).unwrap();
        let original =
            "# keep\r\nfullscreen = 0\r\npad_deadzone = 0.2\r\nlast_tcp_ip = 10.0.0.5\r\n";
        fs::write(&path, original).unwrap();
        fs::write(root.join("hero.d2s"), b"save").unwrap();
        let mut cfg = D2Config::load(path.clone()).unwrap();
        cfg.set("fullscreen", "1".into());
        cfg.save().unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            original.replace("fullscreen = 0", "fullscreen = 1")
        );
        assert_eq!(
            fs::read_to_string(root.join("d2d.cfg.launcher.bak")).unwrap(),
            original
        );
        assert_eq!(fs::read(root.join("hero.d2s")).unwrap(), b"save");
        cfg.set("scale", "9".into());
        assert!(cfg.save().unwrap_err().contains("Invalid"));
        cfg.set("scale", "2".into());
        fs::write(&path, "external edit").unwrap();
        assert!(cfg.save().unwrap_err().contains("Another app changed"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "external edit");
        cfg.set("scale", "2\nvideo=0".into());
        assert!(cfg.save().unwrap_err().contains("single-line"));
        fs::remove_dir_all(root).unwrap();
    }
}
