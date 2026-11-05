mod data;
mod download;
mod engine;
mod job;
#[cfg(test)]
mod tests;
use crate::game::Game;
pub(crate) use data::{DIABLO_URL, HELLFIRE_URL};
pub(crate) use engine::ENGINE_URL;
pub(crate) use job::Job;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
pub struct Plan {
    pub game: Game,
    pub data: PathBuf,
    pub executable: PathBuf,
    pub managed_engine: PathBuf,
}
pub struct Installed {
    pub executable: PathBuf,
    pub game: Game,
}
pub enum Event {
    Progress {
        message: String,
        bytes: u64,
        total: Option<u64>,
    },
    Finished(Result<Installed, String>),
}
fn check_cancel(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Installation canceled. Completed files were kept.".into())
    } else {
        Ok(())
    }
}
struct WorkDir(PathBuf);
impl WorkDir {
    fn new(parent: &Path) -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = parent.join(format!(
            ".devilution-install-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)
            .map_err(|e| format!("Could not create the installation folder: {e}"))?;
        Ok(Self(path))
    }
}
impl Drop for WorkDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
type Progress<'a> = &'a dyn Fn(String, u64, Option<u64>);
fn command_output(command: &mut Command) -> Result<std::process::Output, String> {
    command.output().map_err(|e| {
        format!(
            "Could not run {}: {e}. Install the packages listed in the README.",
            command.get_program().to_string_lossy()
        )
    })
}

fn run(plan: &Plan, cancel: &AtomicBool, progress: Progress<'_>) -> Result<Installed, String> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err("Automatic installation currently supports Linux x86_64 only.".into());
    }
    check_cancel(cancel)?;
    for program in ["curl", "python3", "sha256sum"] {
        let output = command_output(Command::new(program).arg("--version"))?;
        if !output.status.success() {
            return Err(format!(
                "{program} is unavailable. Check the README requirements."
            ));
        }
    }
    fs::create_dir_all(&plan.data).map_err(|e| format!("Could not create the data folder: {e}"))?;
    let work = WorkDir::new(&plan.data)?;
    let executable = engine::prepare(plan, &work.0, cancel, progress)?;
    data::install_archive(&data::DIABLO, &plan.data, &work.0, cancel, progress)?;
    if plan.game == Game::Hellfire {
        for file in data::HELLFIRE {
            data::install_archive(file, &plan.data, &work.0, cancel, progress)?;
        }
    }
    engine::install_resources(&executable, &plan.data)?;
    Ok(Installed {
        executable,
        game: plan.game,
    })
}
