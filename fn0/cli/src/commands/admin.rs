use color_eyre::{Result, eyre::eyre};
use std::io::Write;
use std::path::PathBuf;

pub async fn run(
    task: String,
    project: Option<String>,
    input_file: Option<PathBuf>,
    input: Option<String>,
    timeout_seconds: u64,
) -> Result<()> {
    let project_name = match project {
        Some(p) => p,
        None => {
            let config = crate::config::Config::load("fn0.toml").map_err(|_| {
                eyre!("fn0.toml not found. Pass --project <name> or run in a project directory.")
            })?;
            config
                .name
                .ok_or_else(|| eyre!("'name' field missing in fn0.toml"))?
        }
    };

    let input_body: Vec<u8> = match (input_file, input) {
        (Some(_), Some(_)) => {
            return Err(eyre!("specify at most one of --input-file or --input"));
        }
        (Some(path), None) => std::fs::read(&path)
            .map_err(|e| eyre!("Failed to read {}: {}", path.display(), e))?,
        (None, Some(s)) => s.into_bytes(),
        (None, None) => b"{}".to_vec(),
    };

    let output = fn0_deploy::admin_run(&project_name, &task, input_body, timeout_seconds)
        .await
        .map_err(|e| eyre!(e))?;

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
        Err(eyre!("admin task returned HTTP {}", output.status))
    }
}
