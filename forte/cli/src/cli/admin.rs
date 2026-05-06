use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;

#[derive(Deserialize)]
struct ForteConfig {
    project_id: Option<String>,
}

fn load_project_id(project_dir: &std::path::Path) -> Result<String> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;
    config
        .project_id
        .ok_or_else(|| anyhow!("'project_id' field missing in Forte.toml. Run `forte deploy` first to register the project."))
}

fn read_input(input_file: Option<PathBuf>, input: Option<String>) -> Result<Vec<u8>> {
    match (input_file, input) {
        (Some(_), Some(_)) => Err(anyhow!("specify at most one of --input-file or --input")),
        (Some(path), None) => {
            std::fs::read(&path).map_err(|e| anyhow!("Failed to read {}: {}", path.display(), e))
        }
        (None, Some(s)) => Ok(s.into_bytes()),
        (None, None) => Ok(b"{}".to_vec()),
    }
}

pub async fn run(
    task: String,
    project_dir: PathBuf,
    input_file: Option<PathBuf>,
    input: Option<String>,
    timeout_seconds: u64,
) -> Result<()> {
    let project_id = load_project_id(&project_dir)?;
    let input_body = read_input(input_file, input)?;

    let output = fn0_deploy::admin_run(&project_id, &task, input_body, timeout_seconds).await?;

    if (200..300).contains(&output.status) {
        std::io::stdout().write_all(&output.body)?;
        if !output.body.ends_with(b"\n") {
            println!();
        }
        Ok(())
    } else {
        std::io::stderr().write_all(&output.body)?;
        if !output.body.ends_with(b"\n") {
            eprintln!();
        }
        Err(anyhow!("admin task returned HTTP {}", output.status))
    }
}

pub async fn run_local(
    task: String,
    port: u16,
    input_file: Option<PathBuf>,
    input: Option<String>,
    timeout_seconds: u64,
) -> Result<()> {
    let input_body = read_input(input_file, input)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_seconds))
        .build()?;

    let url = format!("http://localhost:{port}/__forte_admin/{task}");
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(input_body)
        .send()
        .await?;

    let status = resp.status().as_u16();
    let body = resp.bytes().await?;

    if (200..300).contains(&status) {
        std::io::stdout().write_all(&body)?;
        if !body.ends_with(b"\n") {
            println!();
        }
        Ok(())
    } else {
        std::io::stderr().write_all(&body)?;
        if !body.ends_with(b"\n") {
            eprintln!();
        }
        Err(anyhow!("admin task returned HTTP {}", status))
    }
}
