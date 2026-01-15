use assert_cmd::cargo;
use predicates::prelude::*;

fn setup_project(temp: &tempfile::TempDir) -> std::path::PathBuf {
    cargo::cargo_bin_cmd!("forte")
        .args(["init", "my-app"])
        .current_dir(temp)
        .assert()
        .success();

    let project_dir = temp.path().join("my-app");

    std::process::Command::new("npm")
        .arg("install")
        .current_dir(project_dir.join("fe"))
        .status()
        .expect("Failed to run npm install");

    project_dir
}

#[test]
fn test_build_creates_dist() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = setup_project(&temp);

    cargo::cargo_bin_cmd!("forte")
        .args(["build"])
        .current_dir(&project_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Build complete!"));

    assert!(project_dir.join("dist/backend.wasm").exists());
    assert!(project_dir.join("dist/server.js").exists());
    assert!(project_dir.join("dist/public/robots.txt").exists());
}

#[test]
fn test_build_fails_outside_project() {
    let temp = tempfile::tempdir().unwrap();

    cargo::cargo_bin_cmd!("forte")
        .args(["build"])
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a Forte project"));
}
