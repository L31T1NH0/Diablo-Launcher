use super::{Progress, check_cancel, command_output};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
pub(super) struct Download<'a> {
    pub(super) name: &'a str,
    pub(super) url: &'a str,
    pub(super) size: u64,
    pub(super) hash: &'a str,
}
pub(super) fn hash(path: &Path) -> Result<String, String> {
    let output = command_output(Command::new("sha256sum").arg(path))?;
    if !output.status.success() {
        return Err(format!("Could not verify {}.", path.display()));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_start_matches('\\')
        .into())
}
pub(super) fn download(
    file: &Download<'_>,
    target: &Path,
    cancel: &AtomicBool,
    progress: Progress<'_>,
) -> Result<(), String> {
    check_cancel(cancel)?;
    let log = target.with_extension("download.log");
    let errors = fs::File::create(&log).map_err(|e| e.to_string())?;
    let mut child = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--retry",
            "2",
            "--connect-timeout",
            "20",
            "--speed-limit",
            "1024",
            "--speed-time",
            "60",
            "--silent",
            "--show-error",
            "--proto",
            "=http,https",
            "--proto-redir",
            "=http,https",
            "--output",
        ])
        .arg(target)
        .arg(file.url)
        .stdout(Stdio::null())
        .stderr(Stdio::from(errors))
        .spawn()
        .map_err(|e| {
            format!("Could not start curl: {e}. Install the packages listed in the README.")
        })?;
    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return check_cancel(cancel);
        }
        progress(
            format!("Downloading {}", file.name),
            fs::metadata(target).map(|m| m.len()).unwrap_or(0),
            (file.size > 0).then_some(file.size),
        );
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(200)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
    };
    if !status.success() {
        let text = fs::read_to_string(log).unwrap_or_default();
        let tail: String = text
            .chars()
            .rev()
            .take(1500)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        return Err(format!("Download failed for {}. {tail}", file.name));
    }
    check_cancel(cancel)?;
    progress(format!("Checking {}", file.name), 0, None);
    if file.size > 0 && fs::metadata(target).map_err(|e| e.to_string())?.len() != file.size {
        return Err(format!(
            "{} has an unexpected size. It was not installed.",
            file.name
        ));
    }
    if !file.hash.is_empty() && hash(target)? != file.hash {
        return Err(format!(
            "{} failed its checksum check. It was not installed.",
            file.name
        ));
    }
    Ok(())
}
