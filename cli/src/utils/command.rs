use color_eyre::{Result, eyre::eyre};
use std::path::Path;
use tokio::process::Command;

pub struct CommandExecutor;

impl CommandExecutor {
    pub async fn run(
        command: &str,
        args: &[&str],
        working_dir: &Path,
    ) -> Result<()> {
        let output = Command::new(command)
            .args(args)
            .current_dir(working_dir)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(eyre!(
                "Command '{}' failed:\n{}",
                command,
                stderr
            ));
        }

        Ok(())
    }
}
