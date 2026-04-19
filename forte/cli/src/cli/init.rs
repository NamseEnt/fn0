use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run(name: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(project_dir.join("rs/src/pages/index"))?;
    fs::create_dir_all(project_dir.join("rs/.cargo"))?;
    fs::create_dir_all(project_dir.join("fe/src/pages/index"))?;
    fs::create_dir_all(project_dir.join("fe/public"))?;

    fs::write(project_dir.join("Forte.toml"), generate_forte_toml(name))?;
    fs::write(project_dir.join(".gitignore"), generate_root_gitignore())?;
    fs::write(
        project_dir.join("Cargo.toml"),
        generate_workspace_cargo_toml(),
    )?;

    fs::write(project_dir.join("rs/.gitignore"), generate_rs_gitignore())?;
    fs::write(project_dir.join("rs/Cargo.toml"), generate_cargo_toml())?;

    fs::write(
        project_dir.join("rs/.cargo/config.toml"),
        generate_cargo_config(),
    )?;

    fs::write(project_dir.join("rs/src/lib.rs"), generate_lib_rs())?;

    fs::write(
        project_dir.join("rs/src/pages/index/mod.rs"),
        generate_index_mod_rs(),
    )?;

    fs::write(project_dir.join("rs/build.rs"), generate_build_rs())?;

    fs::write(project_dir.join("fe/.gitignore"), generate_fe_gitignore())?;
    fs::write(
        project_dir.join("fe/package.json"),
        generate_package_json(name),
    )?;

    fs::write(project_dir.join("fe/tsconfig.json"), generate_tsconfig())?;

    fs::write(project_dir.join("fe/src/app.tsx"), generate_app_tsx())?;

    fs::write(
        project_dir.join("fe/src/pages/index/page.tsx"),
        generate_index_page_tsx(),
    )?;

    fs::write(
        project_dir.join("fe/public/robots.txt"),
        generate_robots_txt(),
    )?;

    install_npm_packages(project_dir)?;

    println!("Created project '{}'", name);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  forte dev");

    Ok(())
}

fn install_npm_packages(project_dir: &Path) -> Result<()> {
    let fe_dir = project_dir.join("fe");

    println!("Installing npm packages...");

    let deps = ["react", "react-dom", "zod"];
    let status = Command::new("npm")
        .arg("install")
        .args(deps)
        .current_dir(&fe_dir)
        .status()
        .context("Failed to run npm install")?;

    if !status.success() {
        anyhow::bail!("npm install failed");
    }

    let dev_deps = [
        "@types/react",
        "@types/react-dom",
        "@vitejs/plugin-react",
        "typescript",
        "vite",
    ];
    let status = Command::new("npm")
        .arg("install")
        .arg("-D")
        .args(dev_deps)
        .current_dir(&fe_dir)
        .status()
        .context("Failed to run npm install -D")?;

    if !status.success() {
        anyhow::bail!("npm install -D failed");
    }

    Ok(())
}

fn generate_forte_toml(name: &str) -> String {
    format!(
        r#"[project]
name = "{name}"
"#
    )
}

fn generate_root_gitignore() -> &'static str {
    "/target\n"
}

fn generate_workspace_cargo_toml() -> &'static str {
    r#"[workspace]
resolver = "3"
members = ["rs"]
"#
}

fn generate_rs_gitignore() -> &'static str {
    "/target\n"
}

fn generate_fe_gitignore() -> &'static str {
    "/node_modules\n/dist\n/.forte\n"
}

fn generate_cargo_toml() -> String {
    let forte_json_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Failed to get parent of CARGO_MANIFEST_DIR")
        .join("forte-json");

    format!(
        r#"[package]
name = "backend"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
anyhow = "1"
cookie = "0.18"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
http = "1"
wstd = "0.6"
forte-json = {{ path = "{}" }}
"#,
        forte_json_path.display()
    )
}

fn generate_cargo_config() -> &'static str {
    r#"[build]
target = "wasm32-wasip2"

[target.wasm32-wasip2]
runner = "wasmtime -Shttp"
"#
}

fn generate_lib_rs() -> &'static str {
    r#"mod route_generated;
"#
}

fn generate_index_mod_rs() -> &'static str {
    r#"use anyhow::Result;
use cookie::CookieJar;
use http::HeaderMap;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { message: String },
}

pub async fn handler(_headers: HeaderMap, _jar: CookieJar) -> Result<Props> {
    Ok(Props::Ok {
        message: "Hello from Forte!".to_string(),
    })
}
"#
}

fn generate_build_rs() -> &'static str {
    r##"use std::env;
use std::fs;
use std::path::Path;

fn main() {
    generate_routes();
}

fn write_if_changed(path: &Path, content: &str) {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == content {
            return;
        }
    }
    fs::write(path, content).unwrap();
}

fn generate_routes() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let pages_dir = Path::new(&manifest_dir).join("src/pages");
    let output_path = Path::new(&manifest_dir).join("src/route_generated.rs");

    println!("cargo:rerun-if-changed=src/pages");

    let mut output = String::new();
    output.push_str("// Auto-generated by build.rs\n\n");

    if pages_dir.join("index/mod.rs").exists() {
        output.push_str("#[path = \"pages/index/mod.rs\"]\n");
        output.push_str("mod pages_index;\n\n");
    }

    output.push_str("use anyhow::Result;\n");
    output.push_str("use http::header::COOKIE;\n");
    output.push_str("use http::HeaderMap;\n");
    output.push_str("use wstd::http::{Error, Request, Response, StatusCode, body::Body};\n\n");

    output.push_str("fn make_cookie_jar(headers: &HeaderMap) -> cookie::CookieJar {\n");
    output.push_str("    let mut jar = cookie::CookieJar::new();\n");
    output.push_str("    let Some(cookie) = headers.get(COOKIE) else {\n");
    output.push_str("        return jar;\n");
    output.push_str("    };\n");
    output.push_str("    let Ok(cookie_str) = cookie.to_str() else {\n");
    output.push_str("        return jar;\n");
    output.push_str("    };\n\n");
    output.push_str("    for cookie in cookie::Cookie::split_parse(cookie_str) {\n");
    output.push_str("        let Ok(cookie) = cookie else { continue };\n");
    output.push_str("        jar.add_original(cookie.into_owned());\n");
    output.push_str("    }\n\n");
    output.push_str("    jar\n");
    output.push_str("}\n\n");

    output.push_str("#[wstd::http_server]\n");
    output.push_str("pub async fn main(request: Request<Body>) -> Result<Response<Body>, Error> {\n");
    output.push_str("    let (parts, _body) = request.into_parts();\n");
    output.push_str("    let headers = parts.headers;\n");
    output.push_str("    let jar = make_cookie_jar(&headers);\n");
    output.push_str("    let path = parts.uri.path();\n\n");

    output.push_str("    if path == \"/\" {\n");
    output.push_str("        match pages_index::handler(headers, jar).await {\n");
    output.push_str("            Ok(props) => {\n");
    output.push_str("                let stream = forte_json::to_stream(&props);\n");
    output.push_str("                return Ok(Response::new(Body::from_stream(stream)));\n");
    output.push_str("            }\n");
    output.push_str("            Err(e) => {\n");
    output.push_str("                return Ok(Response::builder()\n");
    output.push_str("                    .status(StatusCode::INTERNAL_SERVER_ERROR)\n");
    output.push_str("                    .body(Body::from(format!(\"Error: {:?}\", e)))\n");
    output.push_str("                    .unwrap());\n");
    output.push_str("            }\n");
    output.push_str("        }\n");
    output.push_str("    }\n\n");

    output.push_str("    Ok(Response::builder()\n");
    output.push_str("        .status(StatusCode::NOT_FOUND)\n");
    output.push_str("        .body(Body::empty())\n");
    output.push_str("        .unwrap())\n");
    output.push_str("}\n");

    write_if_changed(&output_path, &output);
}
"##
}

fn generate_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}-frontend",
  "private": true,
  "type": "module"
}}
"#
    )
}

fn generate_tsconfig() -> &'static str {
    r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "outDir": "dist"
  },
  "include": ["src/**/*"]
}
"#
}

fn generate_app_tsx() -> &'static str {
    r#"export function Head() {
    return (
        <>
            <meta charSet="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <title>Forte App</title>
        </>
    );
}
"#
}

fn generate_index_page_tsx() -> &'static str {
    r#"import type { Props } from "./.props";

export default function IndexPage(props: Props) {
    if (props.t !== "Ok") {
        return <div>Error loading page</div>;
    }

    return (
        <div>
            <h1>Welcome to Forte</h1>
            <p>{props.v.message}</p>
        </div>
    );
}
"#
}

fn generate_robots_txt() -> &'static str {
    r#"User-agent: *
Allow: /
"#
}
