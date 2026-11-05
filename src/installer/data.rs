use super::{
    Progress, check_cancel,
    download::{Download, download},
};
use std::{fs, io::Read, path::Path, sync::atomic::AtomicBool};
pub const DIABLO_URL: &str = "http://cdn.pvpgn.pro/diablo1/DIABDAT.MPQ";
pub const HELLFIRE_URL: &str = "https://hoffmeister.li/downloads/games/diablo_1/";

pub(super) const DIABLO: Download<'_> = Download {
    name: "DIABDAT.MPQ",
    url: DIABLO_URL,
    size: 517501242,
    hash: "6bc120a603fdb383ff91f374e7db599765d6230d81e5a4c1b33f8254911774e7",
};
pub(super) const HELLFIRE: &[Download<'_>] = &[
    Download {
        name: "hellfire.mpq",
        url: "https://hoffmeister.li/downloads/games/diablo_1/hellfire.mpq",
        size: 65502336,
        hash: "bbbd3e4b09db3c16d72e3f8503058dd09a8aaecccf486c3b6a0fca270cef9ba8",
    },
    Download {
        name: "hfmonk.mpq",
        url: "https://hoffmeister.li/downloads/games/diablo_1/hfmonk.mpq",
        size: 37658368,
        hash: "5358fa2aa6c7bfdd3bfdd07b0457807d3220a60cfe8b1967a0938b433a9b33c5",
    },
    Download {
        name: "hfmusic.mpq",
        url: "https://hoffmeister.li/downloads/games/diablo_1/hfmusic.mpq",
        size: 34379360,
        hash: "2d719ef13502bf3c68eb4771411391bd37bb420d753eb6239becb063c6918de1",
    },
    Download {
        name: "hfvoice.mpq",
        url: "https://hoffmeister.li/downloads/games/diablo_1/hfvoice.mpq",
        size: 37743520,
        hash: "243fec5f04710ba316b9f61065a257e47c1404b13f63c52541ed4eb714267b72",
    },
];

pub(super) fn existing_archive(data: &Path, name: &str) -> Result<bool, String> {
    let entries = match fs::read_dir(data) {
        Ok(entries) => entries,
        Err(e) => return Err(e.to_string()),
    };
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(name)
        {
            let mut header = [0; 32];
            let valid = fs::File::open(entry.path())
                .and_then(|mut f| f.read_exact(&mut header))
                .is_ok()
                && &header[..4] == b"MPQ\x1a";
            if !valid {
                return Err(format!(
                    "{name} already exists but is not a valid MPQ. Move it aside or choose another data folder; it was not overwritten."
                ));
            }
            return Ok(true);
        }
    }
    Ok(false)
}
pub(super) fn install_archive(
    file: &Download<'_>,
    data: &Path,
    work: &Path,
    cancel: &AtomicBool,
    progress: Progress<'_>,
) -> Result<(), String> {
    if existing_archive(data, file.name)? {
        return Ok(());
    }
    let part = work.join(format!("{}.part", file.name));
    download(file, &part, cancel, progress)?;
    check_cancel(cancel)?;
    let mut header = [0; 4];
    fs::File::open(&part)
        .and_then(|mut f| f.read_exact(&mut header))
        .map_err(|e| e.to_string())?;
    if &header != b"MPQ\x1a" {
        return Err(format!("{} is not an MPQ archive.", file.name));
    }
    // The staging folder shares the filesystem; linking publishes without overwriting a concurrent import.
    fs::hard_link(&part, data.join(file.name))
        .map_err(|e| format!("Could not install {}: {e}", file.name))?;
    fs::remove_file(part).map_err(|e| e.to_string())?;
    Ok(())
}
