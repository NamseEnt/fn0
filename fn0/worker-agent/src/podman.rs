use std::process::Stdio;
use tokio::process::Command;
use tracing::*;

pub struct Podman {
    bin: String,
}

impl Podman {
    pub fn from_env() -> Self {
        Self {
            bin: std::env::var("FN0_AGENT_PODMAN_BIN").unwrap_or_else(|_| "podman".to_string()),
        }
    }

    pub async fn pull_image(&self, image_ref: &str) -> Result<(), PodmanError> {
        debug!(%image_ref, "podman pull");
        self.run_checked(&["pull", image_ref]).await
    }

    pub async fn run_detached(&self, args: RunArgs<'_>) -> Result<(), PodmanError> {
        let mut argv: Vec<String> = vec!["run".into(), "-d".into()];
        argv.push("--name".into());
        argv.push(args.container_name.into());
        argv.push("--network=host".into());
        for (k, v) in args.env {
            argv.push("--env".into());
            argv.push(format!("{k}={v}"));
        }
        argv.push("--env-file".into());
        argv.push(args.env_file.into());
        argv.push("--restart=no".into());
        argv.push(args.image_ref.into());
        debug!(?argv, "podman run");
        let argv_refs: Vec<&str> = argv.iter().map(String::as_str).collect();
        self.run_checked(&argv_refs).await
    }

    pub async fn stop(&self, container_name: &str, timeout_secs: u32) -> Result<(), PodmanError> {
        debug!(%container_name, timeout_secs, "podman stop");
        let timeout = timeout_secs.to_string();
        self.run_checked(&["stop", "--time", &timeout, container_name])
            .await
    }

    pub async fn remove(&self, container_name: &str) -> Result<(), PodmanError> {
        debug!(%container_name, "podman rm");
        self.run_checked(&["rm", "-f", container_name]).await
    }

    pub async fn is_running(&self, container_name: &str) -> Result<bool, PodmanError> {
        let output = Command::new(&self.bin)
            .args([
                "inspect",
                "--format",
                "{{.State.Running}}",
                container_name,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(PodmanError::Spawn)?;
        if !output.status.success() {
            return Ok(false);
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
    }

    async fn run_checked(&self, args: &[&str]) -> Result<(), PodmanError> {
        let output = Command::new(&self.bin)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(PodmanError::Spawn)?;
        if !output.status.success() {
            return Err(PodmanError::NonZeroExit {
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }
        Ok(())
    }
}

pub struct RunArgs<'a> {
    pub container_name: &'a str,
    pub image_ref: &'a str,
    pub env: &'a [(&'a str, &'a str)],
    pub env_file: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum PodmanError {
    #[error("failed to spawn podman: {0}")]
    Spawn(std::io::Error),
    #[error("podman exited with code {code:?}: {stderr}")]
    NonZeroExit {
        code: Option<i32>,
        stderr: String,
    },
}
