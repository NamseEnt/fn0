use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const INIT_FORTE_JSON_VERSION: &str = "0.1.3";
const INIT_FORTE_SDK_VERSION: &str = "0.11.0";
const INIT_FORTE_CODEGEN_VERSION: &str = "0.5.1";
const INIT_FN0_DOC_DB_VERSION: &str = "0.4.16";
const INIT_FN0_OBJECT_STORAGE_VERSION: &str = "0.5.9";

const DEV_WORKSPACE_MANIFESTS: &[&str] = &[
    "forte/json/Cargo.toml",
    "forte/sdk/Cargo.toml",
    "forte/codegen/Cargo.toml",
    "doc-db/Cargo.toml",
    "object-storage/Cargo.toml",
];

pub fn run(name: &str, dev: bool) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    let rust_manifest = rs_cargo_toml(name, dev)?;

    fs::create_dir_all(project_dir.join("rs/.cargo"))?;
    fs::create_dir_all(project_dir.join("rs/src/pages/index"))?;
    fs::create_dir_all(project_dir.join("fe/public"))?;
    fs::create_dir_all(project_dir.join("fe/src/pages/index"))?;

    fs::write(project_dir.join(".gitignore"), ROOT_GITIGNORE)?;
    fs::write(project_dir.join("Forte.toml"), "")?;

    fs::write(project_dir.join("rs/.gitignore"), RS_GITIGNORE)?;
    fs::write(project_dir.join("rs/.cargo/config.toml"), RS_CARGO_CONFIG)?;
    fs::write(project_dir.join("rs/Cargo.toml"), rust_manifest)?;
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

fn rs_cargo_toml(name: &str, dev: bool) -> Result<String> {
    let (forte_json_dep, forte_sdk_dep, doc_db_dep, object_storage_dep, forte_codegen_dep) = if dev
    {
        let workspace_root = workspace_root_path(Path::new(env!("CARGO_MANIFEST_DIR")))?;
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
            format!(r#""={INIT_FORTE_JSON_VERSION}""#),
            format!(r#""={INIT_FORTE_SDK_VERSION}""#),
            format!(r#"{{ package = "fn0-doc-db", version = "={INIT_FN0_DOC_DB_VERSION}" }}"#),
            format!(
                r#"{{ package = "fn0-object-storage", version = "={INIT_FN0_OBJECT_STORAGE_VERSION}" }}"#
            ),
            format!(r#""={INIT_FORTE_CODEGEN_VERSION}""#),
        )
    };

    Ok(format!(
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
    ))
}

fn workspace_root_path(manifest_dir: &Path) -> Result<PathBuf> {
    for candidate in manifest_dir.ancestors() {
        let workspace_manifest_path = candidate.join("Cargo.toml");
        let Ok(workspace_manifest) = fs::read_to_string(&workspace_manifest_path) else {
            continue;
        };
        let Ok(workspace_manifest) = workspace_manifest.parse::<toml::Value>() else {
            continue;
        };
        if workspace_manifest.get("workspace").is_some()
            && DEV_WORKSPACE_MANIFESTS
                .iter()
                .all(|relative_path| candidate.join(relative_path).is_file())
        {
            return Ok(candidate.to_path_buf());
        }
    }

    anyhow::bail!(
        "`forte init --dev` requires a Forte source workspace; use `forte init <name>` with an installed CLI"
    )
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

const ROOT_GITIGNORE: &str =
    "/target\n/dist\n/.forte\n/env.local.yaml\n/rs/src/route_generated.rs\n";
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_versions_match_workspace_packages() {
        let workspace_root = workspace_root_path(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let expected_packages = [
            (
                "forte/json/Cargo.toml",
                "forte-json",
                INIT_FORTE_JSON_VERSION,
            ),
            ("forte/sdk/Cargo.toml", "forte-sdk", INIT_FORTE_SDK_VERSION),
            (
                "forte/codegen/Cargo.toml",
                "forte-codegen",
                INIT_FORTE_CODEGEN_VERSION,
            ),
            ("doc-db/Cargo.toml", "fn0-doc-db", INIT_FN0_DOC_DB_VERSION),
            (
                "object-storage/Cargo.toml",
                "fn0-object-storage",
                INIT_FN0_OBJECT_STORAGE_VERSION,
            ),
        ];

        for (relative_path, expected_name, expected_version) in expected_packages {
            let manifest_path = workspace_root.join(relative_path);
            let manifest_text = fs::read_to_string(&manifest_path).unwrap();
            let manifest_value = manifest_text.parse::<toml::Value>().unwrap();
            let package = manifest_value.get("package").unwrap();

            assert_eq!(
                package.get("name").and_then(toml::Value::as_str),
                Some(expected_name)
            );
            assert_eq!(
                package.get("version").and_then(toml::Value::as_str),
                Some(expected_version)
            );
        }
    }

    #[test]
    fn dev_workspace_resolution_fails_without_workspace() {
        let temporary_directory = tempfile::tempdir().unwrap();
        let error = workspace_root_path(temporary_directory.path()).unwrap_err();

        assert!(error.to_string().contains("forte init --dev"));
    }

    #[test]
    fn release_manifest_uses_registry_dependencies() {
        let manifest = rs_cargo_toml("my-app", false).unwrap();

        assert!(manifest.contains("forte-json = \"=0.1.3\""));
        assert!(manifest.contains("forte-sdk = \"=0.11.0\""));
        assert!(manifest.contains("package = \"fn0-doc-db\", version = \"=0.4.16\""));
        assert!(manifest.contains("package = \"fn0-object-storage\", version = \"=0.5.9\""));
        assert!(manifest.contains("forte-codegen = \"=0.5.1\""));
        assert!(!manifest.contains("path = "));
    }
}
