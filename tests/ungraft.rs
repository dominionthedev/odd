use std::fs;
use std::process::Command;

fn odd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_odd"))
}

fn init(home: &std::path::Path, project: &std::path::Path) {
    assert!(odd()
        .arg("init")
        .current_dir(project)
        .env("HOME", home)
        .status()
        .unwrap()
        .success());
}

fn remember_file(home: &std::path::Path, project: &std::path::Path, path: &str) {
    assert!(odd()
        .args(["remember", path])
        .current_dir(project)
        .env("HOME", home)
        .status()
        .unwrap()
        .success());
}

fn show(home: &std::path::Path, project: &std::path::Path, target: &str) -> String {
    let out = odd()
        .args(["show", target])
        .current_dir(project)
        .env("HOME", home)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Default behavior: ungraft rebinds the same name to the result, so it
/// reads like an edit even though it's a new object underneath.
#[test]
fn ungraft_rebinds_the_same_name_by_default() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    init(home.path(), project.path());

    fs::write(project.path().join("a.txt"), b"a").unwrap();
    fs::write(project.path().join("b.txt"), b"b").unwrap();
    remember_file(home.path(), project.path(), "a.txt");
    remember_file(home.path(), project.path(), "b.txt");

    odd()
        .args(["graft", "bundle", "A=a.txt", "B=b.txt"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();

    let before = show(home.path(), project.path(), "bundle");
    assert!(before.contains('A') && before.contains('B'));

    let ungraft = odd()
        .args(["ungraft", "bundle", "B"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();
    assert!(ungraft.success());

    let after = show(home.path(), project.path(), "bundle");
    assert!(after.contains('A'));
    assert!(
        !after.contains('B'),
        "B should be gone after ungraft: {after}"
    );
}

/// `--as` leaves the original binding completely untouched — same
/// immutability guarantee graft's --into already has, mirrored here.
#[test]
fn ungraft_as_leaves_the_original_binding_untouched() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    init(home.path(), project.path());

    fs::write(project.path().join("a.txt"), b"a").unwrap();
    fs::write(project.path().join("b.txt"), b"b").unwrap();
    remember_file(home.path(), project.path(), "a.txt");
    remember_file(home.path(), project.path(), "b.txt");

    odd()
        .args(["graft", "bundle", "A=a.txt", "B=b.txt"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();

    let before = show(home.path(), project.path(), "bundle");

    odd()
        .args(["ungraft", "bundle", "B", "--as", "bundle-without-b"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();

    let after = show(home.path(), project.path(), "bundle");
    assert_eq!(
        before, after,
        "ungraft --as must not touch the original binding"
    );

    let new = show(home.path(), project.path(), "bundle-without-b");
    assert!(new.contains('A'));
    assert!(!new.contains('B'));
}

#[test]
fn ungraft_rejects_a_non_composed_target() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    init(home.path(), project.path());

    fs::write(project.path().join("a.txt"), b"a").unwrap();
    remember_file(home.path(), project.path(), "a.txt");

    let out = odd()
        .args(["ungraft", "a.txt", "whatever"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("composed"), "got: {stderr}");
}

#[test]
fn ungraft_rejects_an_entry_that_does_not_exist() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    init(home.path(), project.path());

    fs::write(project.path().join("a.txt"), b"a").unwrap();
    remember_file(home.path(), project.path(), "a.txt");
    odd()
        .args(["graft", "bundle", "A=a.txt"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .status()
        .unwrap();

    let out = odd()
        .args(["ungraft", "bundle", "does-not-exist"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no such entry"), "got: {stderr}");
}
