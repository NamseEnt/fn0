use assert_cmd::cargo;
use predicates::prelude::*;

fn setup_project(temp: &tempfile::TempDir) -> std::path::PathBuf {
    cargo::cargo_bin_cmd!("forte")
        .args(["init", "my-app", "--dev"])
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
    let index_path = project_dir.join("rs/src/pages/index/mod.rs");
    let index_source = std::fs::read_to_string(&index_path).unwrap();
    std::fs::write(
        index_path,
        index_source.replace(
            "pub async fn handler",
            "#[forte_sdk::cache_static]\npub async fn handler",
        ),
    )
    .unwrap();

    cargo::cargo_bin_cmd!("forte")
        .args(["build"])
        .current_dir(&project_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Build complete!"));

    assert!(project_dir.join("dist/backend.wasm").exists());
    assert!(project_dir.join("dist/server.js").exists());
    assert!(project_dir.join("fe/dist/robots.txt").exists());
    assert!(project_dir.join("fe/dist/client.js").exists());
    assert!(!project_dir.join("dist/static-pages.json").exists());
    let generated_routes =
        std::fs::read_to_string(project_dir.join("rs/src/route_generated.rs")).unwrap();
    assert!(generated_routes.contains("fn handle_cache_policy"));
    assert!(generated_routes.contains("x-fn0-cache-policy"));
    assert!(
        !project_dir
            .join("fe/dist/__forte/pages/manifest.json")
            .exists()
    );
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
