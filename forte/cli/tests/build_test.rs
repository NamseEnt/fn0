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
        .stderr(predicate::str::contains(
            "requires `pub async fn cache_static_eligible",
        ));
}

const RAW_RESPONSE_API: &str = r#"
use anyhow::Result;
use forte_sdk::http::{Body, Response};
use forte_sdk::{ForteRequest, ForteResponse};

pub type Props = ForteResponse;

pub async fn handler(req: ForteRequest<'_>) -> Result<Props> {
    if req.headers.get("x-webhook-signature").is_none() {
        return Ok(Response::builder().status(401).body(Body::empty())?);
    }
    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("x-fn0-next", "js")
        .body(Body::from(serde_json::to_vec(
            &serde_json::json!({ "type": 1 }),
        )?))?)
}
"#;

#[test]
fn test_build_raw_response_api() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = setup_project(&temp);
    let apis_dir = project_dir.join("rs/src/apis");
    std::fs::create_dir_all(&apis_dir).unwrap();
    std::fs::write(apis_dir.join("webhook.rs"), RAW_RESPONSE_API).unwrap();

    cargo::cargo_bin_cmd!("forte")
        .args(["build"])
        .current_dir(&project_dir)
        .assert()
        .success();

    let generated_routes =
        std::fs::read_to_string(project_dir.join("rs/src/route_generated.rs")).unwrap();
    assert!(generated_routes.contains("stripped_header_names"));
}

#[test]
fn test_build_raw_response_on_page_fails() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = setup_project(&temp);
    let page_dir = project_dir.join("rs/src/pages/raw");
    std::fs::create_dir_all(&page_dir).unwrap();
    std::fs::write(page_dir.join("mod.rs"), RAW_RESPONSE_API).unwrap();

    cargo::cargo_bin_cmd!("forte")
        .args(["build"])
        .current_dir(&project_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("only supported under src/apis/"));
}

const WEBSOCKET_ROUTE: &str = r#"
use forte_sdk::anyhow::Result;
use forte_sdk::websocket::{
    ConnectDecision, ConnectEvent, DisconnectEvent, MessageEvent,
};

pub async fn on_connect(_event: ConnectEvent) -> Result<ConnectDecision> {
    Ok(ConnectDecision::accept())
}

pub async fn on_message(_event: MessageEvent) -> Result<()> {
    Ok(())
}

pub async fn on_disconnect(_event: DisconnectEvent) -> Result<()> {
    Ok(())
}
"#;

const DYNAMIC_WEBSOCKET_ROUTE: &str = r#"
use forte_sdk::anyhow::Result;
use forte_sdk::websocket::{ConnectDecision, ConnectEvent, MessageEvent};

pub struct PathParams {
    pub room_id: String,
}

pub async fn on_connect(
    _event: ConnectEvent,
    _path_params: PathParams,
) -> Result<ConnectDecision> {
    Ok(ConnectDecision::accept())
}

pub async fn on_message(_event: MessageEvent, _path_params: PathParams) -> Result<()> {
    Ok(())
}
"#;

const OUTBOUND_WEBSOCKET_ROUTE: &str = r#"
use forte_sdk::anyhow::Result;
use forte_sdk::websocket::{DisconnectEvent, MessageEvent};

pub async fn on_message(_event: MessageEvent) -> Result<()> {
    Ok(())
}

pub async fn on_disconnect(_event: DisconnectEvent) -> Result<()> {
    Ok(())
}
"#;

#[test]
fn test_build_websocket_route() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = setup_project(&temp);
    let ws_in_dir = project_dir.join("rs/src/ws_in");
    let ws_out_dir = project_dir.join("rs/src/ws_out");
    std::fs::create_dir_all(&ws_in_dir).unwrap();
    std::fs::create_dir_all(&ws_out_dir).unwrap();
    std::fs::write(ws_in_dir.join("chat.rs"), WEBSOCKET_ROUTE).unwrap();
    std::fs::write(ws_in_dir.join("[room_id].rs"), DYNAMIC_WEBSOCKET_ROUTE).unwrap();
    std::fs::write(ws_out_dir.join("slack.rs"), OUTBOUND_WEBSOCKET_ROUTE).unwrap();

    cargo::cargo_bin_cmd!("forte")
        .args(["build"])
        .current_dir(&project_dir)
        .assert()
        .success();

    let generated_routes =
        std::fs::read_to_string(project_dir.join("rs/src/route_generated.rs")).unwrap();
    let generated_lib = std::fs::read_to_string(project_dir.join("rs/src/lib.rs")).unwrap();
    assert!(generated_routes.contains("handle_websocket_event"));
    assert!(generated_routes.contains("websockets_ws_in_chat::on_connect"));
    assert!(generated_routes.contains("websockets_ws_in_chat::on_message"));
    assert!(generated_routes.contains("websockets_ws_in__room_id_::on_connect"));
    assert!(generated_routes.contains("let path_params = websockets_ws_in__room_id_::PathParams"));
    assert!(generated_routes.contains("websockets_ws_out_slack::on_message"));
    assert!(!generated_routes.contains("websockets_ws_out_slack::on_connect"));
    assert!(generated_routes.contains("const PATH: &str = \"/ws_out/slack\";"));
    assert!(generated_routes.contains("pub async fn connect"));
    assert!(generated_routes.contains("pub mod ws_out"));
    assert!(generated_lib.contains("pub use route_generated::ws_out;"));
    assert!(
        generated_routes
            .find("websockets_ws_in_chat::on_connect")
            .expect("static route")
            < generated_routes
                .find("websockets_ws_in__room_id_::on_connect")
                .expect("dynamic route")
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
