use std::process::Command;

/// Fails without the fix it covers: before the default-slug change, an
/// `init` with no --store produced a raw UUID directory name. This checks
/// that a workspace and a readable, non-UUID store both actually exist
/// on disk after `odd init`.
#[test]
fn init_creates_workspace_and_readable_store_name() {
    let odd_home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_odd"))
        .arg("init")
        .current_dir(project.path())
        .env("HOME", odd_home.path())
        .env_remove("ODD_HOME")
        .status()
        .expect("run odd init");
    assert!(status.success());

    let odd_dir = project.path().join(".odd");
    assert!(odd_dir.is_dir(), ".odd was not created");

    let store_id = std::fs::read_to_string(odd_dir.join("store_id"))
        .expect("store_id locator missing")
        .trim()
        .to_string();

    // The store id should be a slug of the project dir name, not a raw
    // UUID — a UUID always contains hyphens in the 8-4-4-4-12 shape.
    assert!(
        !store_id.contains('-') || store_id.len() != 36,
        "expected a readable slug, got what looks like a raw UUID: {store_id}"
    );

    let store_dir = odd_home
        .path()
        .join(".local")
        .join("share")
        .join("odd")
        .join("stores")
        .join(&store_id);
    assert!(
        store_dir.join("objects").is_dir(),
        "store objects/ dir missing"
    );
}
