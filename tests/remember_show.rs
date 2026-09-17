use std::fs;
use std::process::Command;

fn odd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_odd"))
}

/// Two files with byte-identical content must dedupe to the same object
/// id. This is the whole point of content addressing — if it stops being
/// true, everything downstream (graft, resurrect defaults, GC) is wrong.
#[test]
fn identical_content_dedupes_to_one_object() {
    let odd_home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    let init = odd()
        .arg("init")
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap();
    assert!(init.success());

    fs::write(project.path().join("a.txt"), b"same bytes").unwrap();
    fs::write(project.path().join("b.txt"), b"same bytes").unwrap();

    let out_a = odd()
        .args(["remember", "a.txt"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .output()
        .unwrap();
    let out_b = odd()
        .args(["remember", "b.txt"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .output()
        .unwrap();
    assert!(out_a.status.success());
    assert!(out_b.status.success());

    let stdout_a = String::from_utf8_lossy(&out_a.stdout);
    let stdout_b = String::from_utf8_lossy(&out_b.stdout);
    let id_a = stdout_a
        .trim()
        .rsplit('(')
        .next()
        .unwrap()
        .trim_end_matches(')');
    let id_b = stdout_b
        .trim()
        .rsplit('(')
        .next()
        .unwrap()
        .trim_end_matches(')');
    assert_eq!(
        id_a, id_b,
        "identical content must produce the same object id"
    );
}

#[test]
fn show_rejects_a_name_that_was_never_remembered() {
    let odd_home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    odd()
        .arg("init")
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .status()
        .unwrap();

    // "nonexistent-file" is 17 characters — long enough that the old
    // length-threshold heuristic would have misread this as an object id
    // instead of an unresolved name. Exact-length resolution must not.
    let out = odd()
        .args(["show", "nonexistent-file"])
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not remembered"),
        "expected a clear 'not remembered' error, got: {stderr}"
    );
}
