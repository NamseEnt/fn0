use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct SsrBundler {
    project_root: PathBuf,
    output_path: PathBuf,
}

impl SsrBundler {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let output_path = project_root.join("fe/.forte/ssr/server.js");
        Self {
            project_root,
            output_path,
        }
    }

    pub fn build(&self) -> Result<PathBuf> {
        let fe_dir = self.project_root.join("fe");
        let entry_path = fe_dir.join("src/server.tsx");

        // Create output directory
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        println!("[ssr] Building server bundle...");

        let result = Command::new("npx")
            .args([
                "esbuild",
                entry_path.to_str().unwrap(),
                "--bundle",
                "--format=esm",
                "--platform=browser", // ski uses browser-like environment
                "--target=es2020",
                &format!("--outfile={}", self.output_path.to_str().unwrap()),
                // External packages that ski provides
                "--external:react",
                "--external:react-dom",
                "--external:react-dom/server",
                // Don't bundle node_modules that should be handled separately
                "--packages=external",
            ])
            .current_dir(&fe_dir)
            .output()
            .context("Failed to run esbuild for SSR bundle")?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            anyhow::bail!("esbuild SSR bundle failed: {}", stderr.trim());
        }

        println!("[ssr] Server bundle built: {}", self.output_path.display());
        Ok(self.output_path.clone())
    }

    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    pub fn bundle_exists(&self) -> bool {
        self.output_path.exists()
    }

    pub fn invalidate(&self) -> Result<()> {
        if self.output_path.exists() {
            std::fs::remove_file(&self.output_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssr_bundler_paths() {
        let bundler = SsrBundler::new("/tmp/project");
        assert!(bundler.output_path().to_string_lossy().contains(".forte/ssr/server.js"));
    }
}
