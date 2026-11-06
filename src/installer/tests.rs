use super::*;
use super::{
    data::{DIABLO, HELLFIRE, existing_archive, install_archive},
    download::{Download, download, hash},
    engine::extract,
};
use std::{
    env,
    thread::{self, JoinHandle},
    time::Duration,
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    time::{SystemTime, UNIX_EPOCH},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = env::temp_dir().join(format!(
            "devilution-installer-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn mpq() -> Vec<u8> {
    let mut bytes = vec![0; 32];
    bytes[..4].copy_from_slice(b"MPQ\x1a");
    bytes
}
fn serve(body: Vec<u8>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/fixture", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut connection, _) = listener.accept().unwrap();
        connection
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = [0; 4096];
        let _ = connection.read(&mut request).unwrap();
        write!(
            connection,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        connection.write_all(&body).unwrap();
    });
    (url, server)
}

#[test]
fn downloads_publish_only_verified_mpqs_and_preserve_existing_files() {
    let fixture = Fixture::new();
    let data = fixture.0.join("data with spaces");
    fs::create_dir(&data).unwrap();
    let work = WorkDir::new(&data).unwrap();
    let cancel = AtomicBool::new(false);
    let (url, server) = serve(mpq());
    let file = Download {
        name: "test.mpq",
        url: &url,
        size: 32,
        hash: "8715c709521d0c9bb0f569748b5f37495e945e95fa17626d2bd66c8e7d974ba9",
    };
    install_archive(&file, &data, &work.0, &cancel, &|_, _, _| {}).unwrap();
    server.join().unwrap();
    assert_eq!(fs::read(data.join("test.mpq")).unwrap(), mpq());
    // A valid imported file is kept without a second request or a forced mirror checksum.
    let keep = Download {
        name: "TEST.MPQ",
        url: "http://127.0.0.1:9/not-requested",
        size: 123,
        hash: "other",
    };
    install_archive(&keep, &data, &work.0, &cancel, &|_, _, _| {}).unwrap();
    assert!(!data.join("TEST.MPQ").exists());
    let (url, server) = serve(mpq());
    let bad = Download {
        name: "bad.mpq",
        url: &url,
        size: 32,
        hash: "wrong-checksum",
    };
    assert!(
        install_archive(&bad, &data, &work.0, &cancel, &|_, _, _| {})
            .unwrap_err()
            .contains("checksum")
    );
    server.join().unwrap();
    assert!(!data.join("bad.mpq").exists());
    let (url, server) = serve(mpq());
    let short = Download {
        name: "short.mpq",
        url: &url,
        size: 40,
        hash: "",
    };
    assert!(
        install_archive(&short, &data, &work.0, &cancel, &|_, _, _| {})
            .unwrap_err()
            .contains("size")
    );
    server.join().unwrap();
    assert!(!data.join("short.mpq").exists());
    fs::write(data.join("broken.mpq"), "not a game file").unwrap();
    assert!(existing_archive(&data, "broken.mpq").is_err());
    assert_eq!(
        fs::read_to_string(data.join("broken.mpq")).unwrap(),
        "not a game file"
    );
    cancel.store(true, Ordering::Relaxed);
    assert!(
        download(&short, &work.0.join("canceled"), &cancel, &|_, _, _| {})
            .unwrap_err()
            .contains("canceled")
    );
    assert!(!work.0.join("canceled").exists());
}

#[test]
fn cancellation_stops_an_active_download_before_publication() {
    let fixture = Fixture::new();
    let data = fixture.0.join("data");
    fs::create_dir(&data).unwrap();
    let work = WorkDir::new(&data).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/slow", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = [0; 4096];
        let _ = stream.read(&mut request).unwrap();
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1048576\r\nConnection: close\r\n\r\n")
            .unwrap();
        stream.write_all(&vec![0; 32768]).unwrap();
        thread::sleep(Duration::from_millis(700));
    });
    let cancel = AtomicBool::new(false);
    let file = Download {
        name: "slow.mpq",
        url: &url,
        size: 1048576,
        hash: "",
    };
    let result = install_archive(&file, &data, &work.0, &cancel, &|_, bytes, _| {
        if bytes > 0 {
            cancel.store(true, Ordering::Relaxed);
        }
    });
    server.join().unwrap();
    assert!(result.unwrap_err().contains("canceled"));
    assert!(!data.join("slow.mpq").exists());
}

#[test]
fn engine_extraction_rejects_traversal_and_links() {
    let fixture = Fixture::new();
    let archive = fixture.0.join("package.tar.xz");
    let create = r#"
import io,sys,tarfile
with tarfile.open(sys.argv[1],'w:xz') as tar:
    member=tarfile.TarInfo(sys.argv[2])
    if sys.argv[3]=='link':
        member.type=tarfile.SYMTYPE
        member.linkname='../escape'
        tar.addfile(member)
    else:
        member.size=3
        tar.addfile(member,io.BytesIO(b'abc'))
"#;
    for (index, (name, kind)) in [
        ("../escape", "file"),
        ("link", "link"),
        ("devilutionx", "file"),
    ]
    .into_iter()
    .enumerate()
    {
        let status = Command::new("python3")
            .arg("-c")
            .arg(create)
            .arg(&archive)
            .arg(name)
            .arg(kind)
            .status()
            .unwrap();
        assert!(status.success());
        let target = fixture.0.join(format!("unpack-{index}"));
        let result = extract(&archive, &target);
        if index < 2 {
            assert!(result.is_err());
        } else {
            result.unwrap();
            assert_eq!(fs::read(target.join("devilutionx")).unwrap(), b"abc");
        }
        assert!(!fixture.0.join("escape").exists());
    }
}

#[test]
fn complete_installation_is_reused_without_changing_game_settings() {
    let fixture = Fixture::new();
    let data = fixture.0.join("data");
    fs::create_dir(&data).unwrap();
    for file in std::iter::once(&DIABLO).chain(HELLFIRE) {
        fs::write(data.join(file.name), mpq()).unwrap();
    }
    let sentinel = "[Graphics]\nWidth=1280\n";
    fs::write(data.join("diablo.ini"), sentinel).unwrap();
    fs::write(data.join("single_0.sv"), "keep-save").unwrap();
    let executable = fixture.0.join("engine");
    fs::write(&executable, "#!/bin/sh\nprintf 'test engine\\n'\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let plan = Plan {
        game: Game::Hellfire,
        source: None,
        data: data.clone(),
        executable: executable.clone(),
        managed_engine: fixture.0.join("not-used"),
    };
    let result = run(&plan, &AtomicBool::new(false), &|_, _, _| {
        panic!("No downloads should be needed")
    })
    .unwrap();
    assert_eq!(result.executable, executable);
    assert_eq!(result.game, Game::Hellfire);
    assert_eq!(
        fs::read_to_string(data.join("diablo.ini")).unwrap(),
        sentinel
    );
    assert_eq!(
        fs::read_to_string(data.join("single_0.sv")).unwrap(),
        "keep-save"
    );
    assert!(!plan.managed_engine.exists());
    assert!(!fs::read_dir(&data).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".devilution-install")
    }));
}

#[test]
#[ignore = "Downloads the real engine and about 700 MB of game data"]
fn live_fresh_install_from_configured_sources() {
    let fixture = Fixture::new();
    let plan = Plan {
        game: Game::Hellfire,
        source: None,
        data: fixture.0.join("game data with spaces"),
        executable: fixture.0.join("not-installed"),
        managed_engine: fixture.0.join("managed engine"),
    };
    let previous = std::sync::Mutex::new(String::new());
    let result = run(&plan, &AtomicBool::new(false), &|message, _, _| {
        let mut previous = previous.lock().unwrap();
        if *previous != message {
            eprintln!("{message}");
            *previous = message;
        }
    })
    .unwrap();
    assert!(result.executable.is_file());
    for file in std::iter::once(&DIABLO).chain(HELLFIRE) {
        assert_eq!(hash(&plan.data.join(file.name)).unwrap(), file.hash);
    }
    assert!(plan.data.join("devilutionx.mpq").is_file());
    assert!(!plan.data.join("diablo.ini").exists());
}

#[test]
fn diablo2_import_keeps_sources_saves_and_existing_mpqs() {
    let fixture = Fixture::new();
    let source = fixture.0.join("original data");
    let target = fixture.0.join("managed data");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    for name in crate::diablo2::ARCHIVES {
        fs::write(source.join(name.to_uppercase()), mpq()).unwrap();
    }
    fs::write(target.join("hero.d2s"), b"keep save").unwrap();
    let mut keep = mpq();
    keep.extend_from_slice(b"keep existing");
    fs::write(target.join("D2CHAR.MPQ"), &keep).unwrap();
    let work = WorkDir::new(&target).unwrap();
    let cancel = AtomicBool::new(false);
    super::import::archives(Some(&source), &target, &work.0, &cancel, &|_, _, _| {}).unwrap();
    assert_eq!(fs::read(target.join("D2CHAR.MPQ")).unwrap(), keep);
    assert!(!target.join("d2char.mpq").exists());
    assert_eq!(fs::read(target.join("hero.d2s")).unwrap(), b"keep save");
    for name in crate::diablo2::ARCHIVES {
        assert_eq!(fs::read(source.join(name.to_uppercase())).unwrap(), mpq());
    }
    assert_eq!(fs::read(target.join("d2exp.mpq")).unwrap(), mpq());
    // A completed installation needs neither the source folder nor another copy.
    super::import::archives(None, &target, &work.0, &cancel, &|_, _, _| {}).unwrap();
    fs::write(target.join("d2exp.mpq"), b"broken archive").unwrap();
    assert!(
        super::import::archives(Some(&source), &target, &work.0, &cancel, &|_, _, _| {}).is_err()
    );
    assert_eq!(
        fs::read(target.join("d2exp.mpq")).unwrap(),
        b"broken archive"
    );
}

#[test]
fn diablo2_import_cancellation_never_publishes_partial_archive() {
    let fixture = Fixture::new();
    let source = fixture.0.join("source");
    let target = fixture.0.join("target");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    let mut bytes = mpq();
    bytes.resize(2 * 1024 * 1024, 0);
    for name in crate::diablo2::ARCHIVES {
        fs::write(source.join(name), &bytes).unwrap();
    }
    let work = WorkDir::new(&target).unwrap();
    let cancel = AtomicBool::new(false);
    let result = super::import::archives(Some(&source), &target, &work.0, &cancel, &|_, _, _| {
        cancel.store(true, Ordering::Relaxed);
    });
    assert!(result.unwrap_err().contains("canceled"));
    assert!(!target.join("d2data.mpq").exists());
}

#[test]
fn diablo2_install_routes_to_its_engine_and_keeps_diablo1_out() {
    let fixture = Fixture::new();
    let source = fixture.0.join("source");
    fs::create_dir(&source).unwrap();
    for name in crate::diablo2::ARCHIVES {
        fs::write(source.join(name), mpq()).unwrap();
    }
    let executable = fixture.0.join("d2d");
    fs::write(&executable, "#!/bin/sh\n[ \"$1\" = --help ]\n").unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    let plan = Plan {
        game: Game::DiabloII,
        source: Some(source),
        data: fixture.0.join("data"),
        executable: executable.clone(),
        managed_engine: fixture.0.join("engine"),
    };
    let installed = run(&plan, &AtomicBool::new(false), &|_, _, _| {}).unwrap();
    assert_eq!(installed.game, Game::DiabloII);
    assert_eq!(installed.executable, executable);
    assert!(!plan.data.join("DIABDAT.MPQ").exists());
    assert!(!plan.data.join("devilutionx.mpq").exists());
    assert!(crate::diablo2::has_data(&plan.data));
}
