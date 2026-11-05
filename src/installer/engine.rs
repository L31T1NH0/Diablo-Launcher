use super::{
    Plan, Progress, check_cancel, command_output,
    download::{Download, download},
};
use crate::game::has_file;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};
pub const ENGINE_URL: &str = "https://github.com/diasurgical/DevilutionX/releases/latest/download/devilutionx-linux-x86_64.tar.xz";

const EXTRACT: &str = r#"
import pathlib, sys, tarfile
archive, target = map(pathlib.Path, sys.argv[1:])
root = target.resolve()
with tarfile.open(archive, 'r:xz') as tar:
    members = tar.getmembers()
    if sum(m.size for m in members) > 512 * 1024 * 1024:
        raise ValueError('Engine package is too large')
    for member in members:
        path = pathlib.PurePosixPath(member.name)
        if path.is_absolute() or '..' in path.parts or not (member.isfile() or member.isdir()):
            raise ValueError('Unsafe engine package entry: ' + member.name)
        if not root.joinpath(*path.parts).resolve().is_relative_to(root):
            raise ValueError('Engine package path leaves the installation folder')
    for member in members:
        path = root.joinpath(*pathlib.PurePosixPath(member.name).parts)
        if member.isdir():
            path.mkdir(parents=True, exist_ok=True)
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            with tar.extractfile(member) as source, path.open('wb') as destination:
                import shutil
                shutil.copyfileobj(source, destination)
"#;

pub(super) fn extract(archive: &Path, target: &Path) -> Result<(), String> {
    let output = command_output(
        Command::new("python3")
            .arg("-c")
            .arg(EXTRACT)
            .arg(archive)
            .arg(target),
    )?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Could not unpack DevilutionX: {}",
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
    let executable = if plan.executable.is_file() {
        fs::canonicalize(&plan.executable).map_err(|e| e.to_string())?
    } else {
        let executable = plan.managed_engine.join("devilutionx");
        if !executable.is_file() {
            let archive = work.join("engine.tar.xz");
            download(
                &Download {
                    name: "DevilutionX",
                    url: ENGINE_URL,
                    size: 0,
                    hash: "",
                },
                &archive,
                cancel,
                progress,
            )?;
            progress("Unpacking DevilutionX".into(), 0, None);
            let unpacked = work.join("engine");
            extract(&archive, &unpacked)?;
            check_cancel(cancel)?;
            if !unpacked.join("devilutionx").is_file()
                || !unpacked.join("devilutionx.mpq").is_file()
            {
                return Err("The engine package layout changed. Try a manual installation.".into());
            }
            fs::create_dir_all(&plan.managed_engine).map_err(|e| e.to_string())?;
            // Publish the executable last, so an interrupted install is not treated as complete.
            let mut files: Vec<_> = fs::read_dir(&unpacked)
                .map_err(|e| e.to_string())?
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?;
            files.sort_by_key(|entry| entry.file_name() == "devilutionx");
            for entry in files {
                if entry.path().is_dir() {
                    return Err(
                        "Unsupported engine package directory. Try a manual installation.".into(),
                    );
                }
                if matches!(
                    entry.path().extension().and_then(|e| e.to_str()),
                    Some("deb" | "rpm")
                ) {
                    continue;
                }
                let target = plan.managed_engine.join(entry.file_name());
                let staged = plan.managed_engine.join(format!(
                    ".{}.installing",
                    entry.file_name().to_string_lossy()
                ));
                fs::copy(entry.path(), &staged).map_err(|e| e.to_string())?;
                if entry.file_name() == "devilutionx" {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))
                            .map_err(|e| e.to_string())?;
                    }
                }
                fs::rename(staged, target).map_err(|e| e.to_string())?;
            }
        }
        executable
    };
    check_cancel(cancel)?;
    let directory = executable
        .parent()
        .ok_or("The engine path has no parent folder")?;
    let mut libraries = vec![directory.to_path_buf()];
    if let Some(path) = env::var_os("LD_LIBRARY_PATH") {
        libraries.extend(env::split_paths(&path));
    }
    let output = command_output(Command::new(&executable).arg("--version").env(
        "LD_LIBRARY_PATH",
        env::join_paths(libraries).map_err(|e| e.to_string())?,
    ))?;
    if !output.status.success() {
        return Err(format!(
            "DevilutionX could not start. Install your system's packages from the README. {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(executable)
}
pub(super) fn install_resources(executable: &Path, data: &Path) -> Result<(), String> {
    let directory = executable
        .parent()
        .ok_or("The engine path has no parent folder")?;
    for name in ["devilutionx.mpq", "fonts.mpq"] {
        let source = directory.join(name);
        if source.is_file() && !has_file(data, name) {
            let target = data.join(name);
            // create_new preserves an existing file even if another process imports it now.
            let mut input = fs::File::open(source).map_err(|e| e.to_string())?;
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(target)
                .map_err(|e| e.to_string())?;
            std::io::copy(&mut input, &mut output).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
