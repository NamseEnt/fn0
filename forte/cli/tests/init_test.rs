use assert_cmd::cargo;
use predicates::prelude::*;

#[test]
fn test_init_creates_project_structure() {
    let temp = tempfile::tempdir().unwrap();

    cargo::cargo_bin_cmd!("forte")
        .args(["init", "my-app"])
        .current_dir(&temp)
        .assert()
        .success();

    let project_dir = temp.path().join("my-app");

    assert!(project_dir.join("Forte.toml").exists());
    assert!(project_dir.join(".gitignore").exists());
    assert!(project_dir.join("Cargo.toml").exists());

    assert!(project_dir.join("rs/.gitignore").exists());
    assert!(project_dir.join("rs/Cargo.toml").exists());
    assert!(project_dir.join("rs/.cargo/config.toml").exists());
    assert!(project_dir.join("rs/src/lib.rs").exists());
    assert!(project_dir.join("rs/src/pages/index/mod.rs").exists());

    assert!(project_dir.join("fe/.gitignore").exists());
    assert!(project_dir.join("fe/package.json").exists());
    assert!(project_dir.join("fe/tsconfig.json").exists());
    assert!(project_dir.join("fe/vite.config.ts").exists());
    assert!(project_dir.join("fe/src/server.tsx").exists());
    assert!(project_dir.join("fe/src/client.tsx").exists());
    assert!(project_dir.join("fe/src/pages/index/page.tsx").exists());
    assert!(project_dir.join("fe/public/robots.txt").exists());
}

#[test]
fn test_init_fails_if_directory_exists() {
    let temp = tempfile::tempdir().unwrap();

    std::fs::create_dir(temp.path().join("my-app")).unwrap();

    cargo::cargo_bin_cmd!("forte")
        .args(["init", "my-app"])
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_init_requires_project_name() {
    let temp = tempfile::tempdir().unwrap();

    cargo::cargo_bin_cmd!("forte")
        .args(["init"])
        .current_dir(&temp)
        .assert()
        .failure();
}
