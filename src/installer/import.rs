use super::{Progress, check_cancel, data::existing_archive};
use crate::diablo2::ARCHIVES;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    sync::atomic::AtomicBool,
};

pub(super) fn archives(
    source: Option<&Path>,
    target: &Path,
    work: &Path,
    cancel: &AtomicBool,
    progress: Progress<'_>,
) -> Result<(), String> {
    let mut missing = Vec::new();
    for &name in ARCHIVES {
        if !existing_archive(target, name)? {
            missing.push(name);
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    let source = source.filter(|p| p.is_dir()).ok_or(
        "Choose a folder containing your Diablo II + Lord of Destruction MPQs in Install.",
    )?;
    let mut files = Vec::new();
    for name in missing {
        let path = fs::read_dir(source).map_err(|e| e.to_string())?.filter_map(Result::ok)
            .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("{name} is missing from the import folder. A full Diablo II + Lord of Destruction 1.14d installation is required."))?.path();
        if !existing_archive(source, name)? {
            return Err(format!("{name} is not a valid MPQ."));
        }
        files.push((name, path));
    }
    for (name, path) in files {
        check_cancel(cancel)?;
        let mut input = fs::File::open(&path).map_err(|e| e.to_string())?;
        let size = input.metadata().map_err(|e| e.to_string())?.len();
        let part = work.join(format!("{name}.importing"));
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&part)
            .map_err(|e| e.to_string())?;
        let mut buffer = vec![0; 1024 * 1024];
        let mut copied = 0;
        loop {
            check_cancel(cancel)?;
            let n = input.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            output.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
            copied += n as u64;
            progress(format!("Importing {name}"), copied, Some(size));
        }
        if copied != size || fs::metadata(&path).map_err(|e| e.to_string())?.len() != size {
            return Err(format!("{name} changed during import. Please try again."));
        }
        output.sync_all().map_err(|e| e.to_string())?;
        drop(output);
        check_cancel(cancel)?;
        fs::hard_link(&part, target.join(name))
            .map_err(|e| format!("Could not import {name}: {e}"))?;
        fs::remove_file(part).map_err(|e| e.to_string())?;
    }
    Ok(())
}
