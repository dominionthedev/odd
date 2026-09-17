use std::fs;
use std::process::Command;

fn odd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_odd"))
}

/// This is the test that matters most in this repo. In the old design,
/// path-specific metadata (original path, mode, mtime) lived on the
/// shared, content-addressed object. Two names with byte-identical
/// content collapsed to one object row via INSERT OR REPLACE, and
/// whichever was remembered last silently overwrote the other's
/// original_path — so `resurrect` with no explicit --dest could only
/// ever restore one of the two locations. Content dedup working
/// correctly was, in effect, a data-loss bug for anything relying on
/// per-file provenance.
///
/// Metadata now lives on the binding, not the object, specifically so
/// this can never happen again. This test fails immediately if that
/// design decision is ever reverted.
#[test]
fn deduped_object_resurrects_each_binding_to_its_own_path() {
    let odd_home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    odd()
        .arg("init")
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap();

    fs::write(project.path().join("note.txt"), b"hello from odd").unwrap();
    fs::write(project.path().join("note-copy.txt"), b"hello from odd").unwrap();

    assert!(odd()
        .args(["remember", "note.txt"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap()
        .success());
    assert!(odd()
        .args(["remember", "note-copy.txt"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap()
        .success());

    fs::remove_file(project.path().join("note.txt")).unwrap();
    fs::remove_file(project.path().join("note-copy.txt")).unwrap();

    // No --dest on either — each must find its own way back from the
    // binding it was remembered under, not from the shared object.
    assert!(odd()
        .args(["resurrect", "note.txt"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap()
        .success());
    assert!(odd()
        .args(["resurrect", "note-copy.txt"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap()
        .success());

    assert!(
        project.path().join("note.txt").is_file(),
        "note.txt did not come back"
    );
    assert!(
        project.path().join("note-copy.txt").is_file(),
        "note-copy.txt did not come back — this is the dedup metadata bug"
    );

    let a = fs::read(project.path().join("note.txt")).unwrap();
    let b = fs::read(project.path().join("note-copy.txt")).unwrap();
    assert_eq!(a, b);
    assert_eq!(a, b"hello from odd");
}
