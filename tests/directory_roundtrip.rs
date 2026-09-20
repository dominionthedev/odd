use std::fs;
use std::process::Command;

fn odd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_odd"))
}

fn read(path: &std::path::Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Remember a directory recursively, delete it, resurrect it by name with
/// no --dest, and the tree must come back byte-for-byte — including a
/// nested subdirectory, which is the part a flat file-only model can't
/// represent at all.
#[test]
fn directory_round_trips_including_nested_subdir() {
    let odd_home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    odd()
        .arg("init")
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap();

    let app = project.path().join("app");
    fs::create_dir_all(app.join("src")).unwrap();
    fs::write(app.join("README.md"), b"hello app").unwrap();
    fs::write(app.join("src").join("main.rs"), b"fn main() {}").unwrap();

    let remember = odd()
        .args(["remember", "app", "-r"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap();
    assert!(remember.success());

    fs::remove_dir_all(&app).unwrap();
    assert!(!app.exists());

    let resurrect = odd()
        .args(["resurrect", "app"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap();
    assert!(resurrect.success());

    assert_eq!(read(&app.join("README.md")), b"hello app");
    assert_eq!(read(&app.join("src").join("main.rs")), b"fn main() {}");
}

/// Every nested file gets its own binding under `<name>/<relpath>` — so
/// `resurrect app/src/main.rs` on its own, without touching the rest of
/// the tree, must also work.
#[test]
fn nested_file_has_its_own_independent_binding() {
    let odd_home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    odd()
        .arg("init")
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap();

    let app = project.path().join("app");
    fs::create_dir_all(app.join("src")).unwrap();
    fs::write(app.join("src").join("main.rs"), b"fn main() {}").unwrap();

    odd()
        .args(["remember", "app", "-r"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap();

    let show = odd()
        .args(["show", "app/src/main.rs"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .output()
        .unwrap();
    assert!(
        show.status.success(),
        "nested file should be independently showable: {}",
        String::from_utf8_lossy(&show.stderr)
    );
}

/// A directory without -r is refused, not silently mishandled.
#[test]
fn directory_without_recursive_flag_is_refused() {
    let odd_home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    odd()
        .arg("init")
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap();

    fs::create_dir(project.path().join("app")).unwrap();

    let out = odd()
        .args(["remember", "app"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("recursive"),
        "expected a clear error, got: {stderr}"
    );
}
