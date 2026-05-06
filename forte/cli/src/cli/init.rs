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

    fs::write(project_dir.join("Forte.toml"), generate_forte_toml())?;
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

fn generate_forte_toml() -> &'static str {
    ""
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
    let forte_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Failed to get parent of CARGO_MANIFEST_DIR");
    let workspace_root = forte_dir
        .parent()
        .expect("Failed to get workspace root from forte dir");
    let forte_json_path = forte_dir.join("json");
    let forte_sdk_path = forte_dir.join("sdk");
    let doc_db_path = workspace_root.join("doc-db");
    let forte_codegen_path = forte_dir.join("codegen");

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
forte-json = {{ path = "{forte_json}" }}
forte-sdk = {{ path = "{forte_sdk}" }}
doc-db = {{ path = "{doc_db}" }}

[build-dependencies]
forte-codegen = {{ path = "{forte_codegen}" }}
"#,
        forte_json = forte_json_path.display(),
        forte_sdk = forte_sdk_path.display(),
        doc_db = doc_db_path.display(),
        forte_codegen = forte_codegen_path.display(),
    )
}

fn generate_cargo_config() -> &'static str {
    r#"[build]
target = "wasm32-wasip2"
"#
}

fn generate_lib_rs() -> &'static str {
    r#"// === FORTE-MANAGED START ===
// Auto-managed by `forte build`. Do not edit between the START/END markers.
mod route_generated;
// === FORTE-MANAGED END ===
"#
}

fn generate_index_mod_rs() -> &'static str {
    r#"use anyhow::Result;
use forte_sdk::ForteRequest;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { message: String },
}

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    Ok(Props::Ok {
        message: "Hello from Forte!".to_string(),
    })
}
"#
}

fn generate_build_rs() -> &'static str {
    r#"fn main() {
    forte_codegen::generate_routes();
}
"#
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
