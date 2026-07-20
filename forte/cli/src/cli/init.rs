use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const FORTE_JSON_VERSION: &str = env!("FORTE_JSON_VERSION");
const FORTE_SDK_VERSION: &str = env!("FORTE_SDK_VERSION");
const FORTE_CODEGEN_VERSION: &str = env!("FORTE_CODEGEN_VERSION");
const FN0_DOC_DB_VERSION: &str = env!("FN0_DOC_DB_VERSION");
const FN0_OBJECT_STORAGE_VERSION: &str = env!("FN0_OBJECT_STORAGE_VERSION");

pub fn run(name: &str, dev: bool) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(project_dir.join("rs/.cargo"))?;
    fs::create_dir_all(project_dir.join("rs/src/pages/index"))?;
    fs::create_dir_all(project_dir.join("fe/public"))?;
    fs::create_dir_all(project_dir.join("fe/src/pages/index"))?;

    fs::write(project_dir.join(".gitignore"), ROOT_GITIGNORE)?;
    fs::write(project_dir.join("Forte.toml"), "")?;

    fs::write(project_dir.join("rs/.gitignore"), RS_GITIGNORE)?;
    fs::write(project_dir.join("rs/.cargo/config.toml"), RS_CARGO_CONFIG)?;
    fs::write(project_dir.join("rs/Cargo.toml"), rs_cargo_toml(name, dev))?;
    fs::write(project_dir.join("rs/build.rs"), RS_BUILD_RS)?;
    fs::write(project_dir.join("rs/src/lib.rs"), RS_LIB_RS)?;
    fs::write(
        project_dir.join("rs/src/pages/index/mod.rs"),
        RS_INDEX_MOD_RS,
    )?;

    fs::write(project_dir.join("fe/.gitignore"), FE_GITIGNORE)?;
    fs::write(project_dir.join("fe/package.json"), fe_package_json(name))?;
    fs::write(project_dir.join("fe/tsconfig.json"), FE_TSCONFIG)?;
    fs::write(project_dir.join("fe/public/robots.txt"), ROBOTS_TXT)?;
    fs::write(project_dir.join("fe/src/app.tsx"), FE_APP_TSX)?;
    fs::write(
        project_dir.join("fe/src/pages/index/page.tsx"),
        FE_INDEX_PAGE_TSX,
    )?;

    npm_install(&project_dir.join("fe"))?;

    println!("Created project '{name}'");
    println!();
    println!("Next steps:");
    println!("  cd {name}");
    println!("  forte dev");

    Ok(())
}

fn npm_install(fe_dir: &Path) -> Result<()> {
    println!("Installing npm packages...");
    let status = Command::new("npm")
        .arg("install")
        .current_dir(fe_dir)
        .status()
        .context("Failed to run npm install")?;
    if !status.success() {
        anyhow::bail!("npm install failed");
    }
    Ok(())
}

fn rs_cargo_toml(name: &str, dev: bool) -> String {
    let (forte_json_dep, forte_sdk_dep, doc_db_dep, object_storage_dep, forte_codegen_dep) = if dev
    {
        let workspace_root = workspace_root_path();
        (
            format!(
                r#"{{ path = "{}" }}"#,
                workspace_root.join("forte/json").display()
            ),
            format!(
                r#"{{ path = "{}" }}"#,
                workspace_root.join("forte/sdk").display()
            ),
            format!(
                r#"{{ package = "fn0-doc-db", path = "{}" }}"#,
                workspace_root.join("doc-db").display()
            ),
            format!(
                r#"{{ package = "fn0-object-storage", path = "{}" }}"#,
                workspace_root.join("object-storage").display()
            ),
            format!(
                r#"{{ path = "{}" }}"#,
                workspace_root.join("forte/codegen").display()
            ),
        )
    } else {
        (
            format!(r#""={FORTE_JSON_VERSION}""#),
            format!(r#""={FORTE_SDK_VERSION}""#),
            format!(r#"{{ package = "fn0-doc-db", version = "={FN0_DOC_DB_VERSION}" }}"#),
            format!(
                r#"{{ package = "fn0-object-storage", version = "={FN0_OBJECT_STORAGE_VERSION}" }}"#
            ),
            format!(r#""={FORTE_CODEGEN_VERSION}""#),
        )
    };

    format!(
        r#"[workspace]

[package]
name = "{name}"
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
tracing = "0.1"
forte-json = {forte_json_dep}
forte-sdk = {forte_sdk_dep}
doc-db = {doc_db_dep}
object-storage = {object_storage_dep}

[build-dependencies]
forte-codegen = {forte_codegen_dep}
"#
    )
}

fn workspace_root_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root from forte/cli manifest dir")
        .to_path_buf()
}

fn fe_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}-frontend",
  "private": true,
  "type": "module",
  "dependencies": {{
    "react": "^19.2",
    "react-dom": "^19.2",
    "zod": "^4"
  }},
  "devDependencies": {{
    "@types/react": "^19.2",
    "@types/react-dom": "^19.2",
    "@vitejs/plugin-react": "^6",
    "typescript": "^5.9",
    "vite": "^8"
  }}
}}
"#
    )
}

const ROOT_GITIGNORE: &str = "/target\n/dist\n/.forte\n/env.local.yaml\n";
const RS_GITIGNORE: &str = "/target\n";
const FE_GITIGNORE: &str = "/node_modules\n/dist\n/.forte\n";

const RS_CARGO_CONFIG: &str = "[build]\ntarget = \"wasm32-wasip2\"\n";

const RS_BUILD_RS: &str =
    "fn main() {\n    forte_codegen::generate_routes();\n    forte_codegen::generate_env();\n}\n";

const RS_LIB_RS: &str = "// === FORTE-MANAGED START ===\n// Auto-managed by `forte build`. Do not edit between the START/END markers.\nmod route_generated;\n// === FORTE-MANAGED END ===\n\nmod env_generated;\n";

const RS_INDEX_MOD_RS: &str = r#"use anyhow::Result;
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
"#;

const FE_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src", ".forte"]
}
"#;

const ROBOTS_TXT: &str = "User-agent: *\nAllow: /\n";

const FE_APP_TSX: &str = r#"export const head = [
    { title: "Forte App" },
    { name: "viewport", content: "width=device-width, initial-scale=1.0" },
];

export function Head() {
    return <meta charSet="utf-8" />;
}
"#;

const FE_INDEX_PAGE_TSX: &str = r#"import type { Props } from "./.props";

export default function IndexPage(props: Props) {
    if (props.t !== "Ok") {
        return <div>Error loading page</div>;
    }

    return (
        <div>
            <h1>Welcome to Forte</h1>
            <p>{props.message}</p>
        </div>
    );
}
"#;
