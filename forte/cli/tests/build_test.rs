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

const DYNAMIC_CACHE_STATIC_PAGE: &str = r#"
use forte_sdk::ForteRequest;
use serde::Serialize;

pub struct PathParams {
    pub id: u32,
}

#[derive(Serialize)]
pub enum Props {
    Ok { id: u32 },
}

pub async fn cache_static_eligible(params: PathParams) -> anyhow::Result<bool> {
    Ok(params.id == 1)
}

#[forte_sdk::cache_static]
pub async fn handler(_req: ForteRequest<'_>, params: PathParams) -> anyhow::Result<Props> {
    Ok(Props::Ok { id: params.id })
}
"#;

fn write_dynamic_page(project_dir: &std::path::Path, page_source: &str) {
    let page_dir = project_dir.join("rs/src/pages/episode/[id]");
    std::fs::create_dir_all(&page_dir).unwrap();
    std::fs::write(page_dir.join("mod.rs"), page_source).unwrap();

    let component_dir = project_dir.join("fe/src/pages/episode/[id]");
    std::fs::create_dir_all(&component_dir).unwrap();
    std::fs::write(
        component_dir.join("page.tsx"),
        "import type { Props } from \"./.props\";\n\
         export default function EpisodePage(props: Props) {\n\
         \treturn <div>{props.t}</div>;\n\
         }\n",
    )
    .unwrap();
}

#[test]
fn test_build_cache_static_on_dynamic_route() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = setup_project(&temp);
    write_dynamic_page(&project_dir, DYNAMIC_CACHE_STATIC_PAGE);

    cargo::cargo_bin_cmd!("forte")
        .args(["build"])
        .current_dir(&project_dir)
        .assert()
        .success();

    let generated_routes =
        std::fs::read_to_string(project_dir.join("rs/src/route_generated.rs")).unwrap();
    assert!(generated_routes.contains("cache_static_eligible(path_params).await"));
    // The concrete path is matched segment by segment, not compared against
    // the route pattern.
    assert!(generated_routes.contains("path_segments.first() == Some(&\"episode\")"));
}

#[test]
fn test_build_cache_static_on_dynamic_route_without_validator_fails() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = setup_project(&temp);
    write_dynamic_page(
        &project_dir,
        &DYNAMIC_CACHE_STATIC_PAGE.replace("pub async fn cache_static_eligible", "async fn unused"),
    );

    cargo::cargo_bin_cmd!("forte")
        .args(["build"])
        .current_dir(&project_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires `pub async fn cache_static_eligible"));
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
