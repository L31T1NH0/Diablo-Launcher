use super::{
    Plan, Progress, check_cancel, command_output,
    download::{Download, download},
};
use crate::diablo2::VERSION;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};

pub(crate) const URL: &str =
    "https://github.com/psi29a/d2decomped/releases/download/v0.9.0/d2decomped-x86_64.flatpak";
const COMMIT: &str = "d2d9739b675accf9e7ff96a9dd0ce0e0d204e7ad64dad5c19627558b266991d1";
const PACKAGE: Download<'_> = Download {
    name: "D2Decomp 0.9.0",
    url: URL,
    size: 4790904,
    hash: "2c1243070337915943c0007b0eee8a16f9780dfdac189058ea74d77ddbf7a64a",
};

fn ostree(repo: &Path, args: &[&std::ffi::OsStr]) -> Result<(), String> {
    let output = command_output(
        Command::new("ostree")
            .arg(format!("--repo={}", repo.display()))
            .args(args),
    )?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Could not unpack D2Decomp: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub(super) fn prepare(
    plan: &Plan,
    work: &Path,
    cancel: &AtomicBool,
    progress: Progress<'_>,
) -> Result<PathBuf, String> {
    let managed = plan.managed_engine.join(VERSION);
    let executable = if plan.executable.is_file() {
        plan.executable.clone()
    } else {
        managed.join("d2d")
    };
    if !executable.is_file() {
        let output = command_output(Command::new("ostree").arg("--version"))?;
        if !output.status.success() {
            return Err("Install ostree before installing D2Decomp. See the README.".into());
        }
        let bundle = work.join("d2decomp.flatpak");
        download(&PACKAGE, &bundle, cancel, progress)?;
        let repo = work.join("repo");
        let checkout = work.join("checkout");
        progress("Unpacking D2Decomp".into(), 0, None);
        // The official Flatpak engine targets baseline x86-64, including older CPUs without AVX.
        ostree(&repo, &["init".as_ref(), "--mode=bare-user".as_ref()])?;
        check_cancel(cancel)?;
        ostree(
            &repo,
            &[
                "static-delta".as_ref(),
                "apply-offline".as_ref(),
                bundle.as_os_str(),
            ],
        )?;
        check_cancel(cancel)?;
        ostree(
            &repo,
            &[
                "checkout".as_ref(),
                "--user-mode".as_ref(),
                COMMIT.as_ref(),
                checkout.as_os_str(),
            ],
        )?;
        check_cancel(cancel)?;
        let source = checkout.join("files/bin/d2d");
        if !source.is_file() {
            return Err("The D2Decomp package layout changed. Try a manual installation.".into());
        }
        fs::create_dir_all(&managed).map_err(|e| e.to_string())?;
        let staged = managed.join(".d2d.installing");
        fs::copy(source, &staged).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))
                .map_err(|e| e.to_string())?;
        }
        fs::rename(staged, &executable).map_err(|e| e.to_string())?;
    }
    check_cancel(cancel)?;
    let output = command_output(Command::new(&executable).arg("--help"))?;
    if !output.status.success() {
        return Err(format!(
            "D2Decomp could not start. Check the Diablo II requirements in the README. {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    fs::canonicalize(executable).map_err(|e| e.to_string())
}
