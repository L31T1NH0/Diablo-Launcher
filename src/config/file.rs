use std::{fs, io::Write, path::Path};

pub(super) fn read(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Could not read {}: {e}", path.display())),
    }
}

pub(super) fn save(path: &Path, original: &Option<String>, text: &str) -> Result<(), String> {
    if read(path)? != *original {
        return Err(format!(
            "Another app changed {}. Click Reload before saving.",
            path.display()
        ));
    }
    let parent = path
        .parent()
        .ok_or("The settings file has no parent folder")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let name = path
        .file_name()
        .ok_or("Invalid settings filename")?
        .to_string_lossy();
    let temp = parent.join(format!(".{name}.launcher-{}.tmp", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| format!("Could not prepare the settings file: {e}"))?;
    let result = (|| {
        if original.is_some() {
            fs::copy(path, parent.join(format!("{name}.launcher.bak")))?;
            file.set_permissions(fs::metadata(path)?.permissions())?;
        }
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temp, path)
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(temp);
        return Err(format!("Could not save settings: {e}"));
    }
    Ok(())
}
