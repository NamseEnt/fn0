use serde::Deserialize;
use std::process::Stdio;
use tokio::process::Command;
use tracing::*;

pub struct Podman {
    bin: String,
    remote_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ContainerSnapshot {
    #[serde(rename = "Names", default)]
    pub names: Vec<String>,
    #[serde(rename = "Image", default)]
    pub image_ref: String,
    #[serde(rename = "State", default)]
    pub state: String,
}

impl ContainerSnapshot {
    pub fn primary_name(&self) -> Option<&str> {
        self.names.first().map(String::as_str)
    }

    pub fn is_running(&self) -> bool {
        self.state == "running"
    }
}

impl Podman {
    pub fn from_env() -> Self {
        Self {
            bin: std::env::var("FN0_WORKER_AGENT_PODMAN_BIN")
                .unwrap_or_else(|_| "podman".to_string()),
            remote_url: std::env::var("FN0_WORKER_AGENT_PODMAN_REMOTE_URL").ok(),
        }
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::new(&self.bin);
        if let Some(url) = &self.remote_url {
            cmd.args(["--remote", "--url", url]);
        }
        cmd
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

    pub async fn list_with_name_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<ContainerSnapshot>, PodmanError> {
        let filter = format!("name={prefix}");
        let output = self
            .cmd()
            .args(["ps", "--all", "--filter", &filter, "--format", "json"])
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
        let stdout =
            String::from_utf8(output.stdout).map_err(|e| PodmanError::Parse(e.to_string()))?;
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let parsed: Vec<ContainerSnapshot> =
            serde_json::from_str(trimmed).map_err(|e| PodmanError::Parse(e.to_string()))?;
        // podman's `--filter name=` is a substring match, not exact; re-filter by prefix.
        Ok(parsed
            .into_iter()
            .filter(|c| c.primary_name().is_some_and(|n| n.starts_with(prefix)))
            .collect())
    }

    pub async fn image_digest(&self, image_ref: &str) -> Result<String, PodmanError> {
        let output = self
            .cmd()
            .args(["image", "inspect", image_ref, "--format", "{{.Digest}}"])
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
        let digest = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if digest.is_empty() {
            return Err(PodmanError::Parse(format!(
                "empty digest for image {image_ref}"
            )));
        }
        Ok(digest)
    }

    pub async fn is_running(&self, container_name: &str) -> Result<bool, PodmanError> {
        let output = self
            .cmd()
            .args(["inspect", "--format", "{{.State.Running}}", container_name])
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
        let output = self
            .cmd()
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
    NonZeroExit { code: Option<i32>, stderr: String },
    #[error("parse: {0}")]
    Parse(String),
}
