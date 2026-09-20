use std::fs;
use std::process::Command;

fn odd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_odd"))
}

fn init(home: &std::path::Path, project: &std::path::Path) {
    let ok = odd()
        .arg("init")
        .current_dir(project)
        .env("HOME", home)
        .status()
        .unwrap()
        .success();
    assert!(ok);
}

fn remember_file(home: &std::path::Path, project: &std::path::Path, path: &str) {
    let ok = odd()
        .args(["remember", path])
        .current_dir(project)
        .env("HOME", home)
        .status()
        .unwrap()
        .success();
    assert!(ok);
}

/// Grafting two remembered files into a new composed object, then
/// resurrecting the composed object, must produce a directory on disk
/// with both files under their entry names — even though no such
/// directory ever existed on disk before the graft.
#[test]
fn graft_composes_without_touching_disk_then_resurrects() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    init(home.path(), project.path());

    fs::write(project.path().join("readme.txt"), b"hi").unwrap();
    fs::write(project.path().join("license.txt"), b"mit").unwrap();
    remember_file(home.path(), project.path(), "readme.txt");
    remember_file(home.path(), project.path(), "license.txt");

    let graft = odd()
        .args([
            "graft",
            "bundle",
            "README=readme.txt",
            "LICENSE=license.txt",
        ])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();
    assert!(graft.success());

    // The graft must not have touched disk on its own.
    assert!(!project.path().join("bundle").exists());

    let resurrect = odd()
        .args(["resurrect", "bundle", "--dest", "out"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();
    assert!(resurrect.success());

    assert_eq!(fs::read(project.path().join("out/README")).unwrap(), b"hi");
    assert_eq!(
        fs::read(project.path().join("out/LICENSE")).unwrap(),
        b"mit"
    );
}

/// `--into` produces a *new* composed object rather than mutating the
/// original — the original binding, grafted again independently, must
/// still resolve to its original (smaller) content.
#[test]
fn graft_into_does_not_mutate_the_original() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    init(home.path(), project.path());

    fs::write(project.path().join("a.txt"), b"a").unwrap();
    fs::write(project.path().join("b.txt"), b"b").unwrap();
    remember_file(home.path(), project.path(), "a.txt");
    remember_file(home.path(), project.path(), "b.txt");

    odd()
        .args(["graft", "bundle", "A=a.txt"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();

    let show_before = odd()
        .args(["show", "bundle"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .output()
        .unwrap();
    let before = String::from_utf8_lossy(&show_before.stdout).to_string();
    assert!(before.contains('A'));
    assert!(!before.contains('B'));

    odd()
        .args(["graft", "bundle2", "B=b.txt", "--into", "bundle"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();

    // The original binding must be unchanged.
    let show_after = odd()
        .args(["show", "bundle"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .output()
        .unwrap();
    let after = String::from_utf8_lossy(&show_after.stdout).to_string();
    assert_eq!(
        before, after,
        "grafting --into must not mutate the original binding"
    );

    // The new binding has both.
    let show_new = odd()
        .args(["show", "bundle2"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .output()
        .unwrap();
    let new = String::from_utf8_lossy(&show_new.stdout).to_string();
    assert!(new.contains('A'));
    assert!(new.contains('B'));
}

/// `--into` refuses a plain remembered directory — it must name an
/// actual composed object, not just something directory-shaped.
#[test]
fn graft_into_rejects_a_non_composed_target() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    init(home.path(), project.path());

    fs::create_dir(project.path().join("app")).unwrap();
    fs::write(project.path().join("app/f.txt"), b"x").unwrap();
    odd()
        .args(["remember", "app", "-r"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();
    fs::write(project.path().join("g.txt"), b"y").unwrap();
    remember_file(home.path(), project.path(), "g.txt");

    let out = odd()
        .args(["graft", "x", "G=g.txt", "--into", "app"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("composed"),
        "expected a clear error naming the kind mismatch, got: {stderr}"
    );
}
