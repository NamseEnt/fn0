use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyMap {
    pub entries: HashMap<String, String>,
}

pub struct DependencyPrebundler {
    project_root: PathBuf,
    cache_dir: PathBuf,
}

impl DependencyPrebundler {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let cache_dir = project_root.join("fe/.forte/deps");
        Self {
            project_root,
            cache_dir,
        }
    }

    pub fn prebundle(&self) -> Result<DependencyMap> {
        // Create cache directory
        std::fs::create_dir_all(&self.cache_dir)?;

        // Read package.json
        let package_json_path = self.project_root.join("fe/package.json");
        let package_json: PackageJson = serde_json::from_str(
            &std::fs::read_to_string(&package_json_path)
                .with_context(|| format!("Failed to read {:?}", package_json_path))?,
        )?;

        let mut dep_map = DependencyMap::default();

        // Collect all dependencies
        let mut all_deps: Vec<String> = Vec::new();
        if let Some(deps) = &package_json.dependencies {
            all_deps.extend(deps.keys().cloned());
        }

        // Bundle each dependency
        for dep_name in all_deps {
            match self.bundle_dependency(&dep_name) {
                Ok(output_path) => {
                    let url_path = format!(
                        "/.forte/deps/{}",
                        output_path.file_name().unwrap().to_string_lossy()
                    );
                    dep_map.entries.insert(dep_name.clone(), url_path);
                    tracing::debug!("Bundled dependency: {}", dep_name);
                }
                Err(e) => {
                    tracing::warn!("Failed to bundle {}: {}", dep_name, e);
                }
            }
        }

        // Bundle react-refresh runtime for Fast Refresh
        if let Ok(output_path) = self.bundle_react_refresh_runtime() {
            let url_path = format!(
                "/.forte/deps/{}",
                output_path.file_name().unwrap().to_string_lossy()
            );
            dep_map.entries.insert("react-refresh/runtime".to_string(), url_path);
        }

        // Write dependency map for later use
        let map_path = self.cache_dir.join("dep-map.json");
        std::fs::write(&map_path, serde_json::to_string_pretty(&dep_map)?)?;

        Ok(dep_map)
    }

    fn bundle_react_refresh_runtime(&self) -> Result<PathBuf> {
        let output_path = self.cache_dir.join("react-refresh-runtime.js");

        if output_path.exists() {
            return Ok(output_path);
        }

        let entry_content = r#"
import RefreshRuntime from 'react-refresh/runtime';
export default RefreshRuntime;
export * from 'react-refresh/runtime';
"#;
        let entry_path = self.cache_dir.join("_entry_react_refresh.js");
        std::fs::write(&entry_path, entry_content)?;

        let fe_dir = self.project_root.join("fe");
        let result = Command::new("npx")
            .args([
                "esbuild",
                entry_path.to_str().unwrap(),
                "--bundle",
                "--format=esm",
                "--platform=browser",
                "--target=es2020",
                &format!("--outfile={}", output_path.to_str().unwrap()),
            ])
            .current_dir(&fe_dir)
            .output()
            .context("Failed to bundle react-refresh runtime")?;

        let _ = std::fs::remove_file(&entry_path);

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            anyhow::bail!("Failed to bundle react-refresh: {}", stderr.trim());
        }

        println!("[deps] Bundled react-refresh runtime");
        Ok(output_path)
    }

    fn bundle_dependency(&self, package_name: &str) -> Result<PathBuf> {
        let hash = compute_short_hash(package_name);
        let output_filename = format!("{}-{}.js", sanitize_package_name(package_name), hash);
        let output_path = self.cache_dir.join(&output_filename);

        // Skip if already bundled
        if output_path.exists() {
            return Ok(output_path);
        }

        // Create a temporary entry file that exports the package
        let entry_content = format!(
            r#"export * from "{}";
import defaultExport from "{}";
export default defaultExport;"#,
            package_name, package_name
        );
        let entry_path = self.cache_dir.join(format!("_entry_{}.js", hash));
        std::fs::write(&entry_path, &entry_content)?;

        // Run esbuild
        let fe_dir = self.project_root.join("fe");
        let result = Command::new("npx")
            .args([
                "esbuild",
                entry_path.to_str().unwrap(),
                "--bundle",
                "--format=esm",
                "--platform=browser",
                "--target=es2020",
                &format!("--outfile={}", output_path.to_str().unwrap()),
                // Mark external packages that shouldn't be bundled
                "--external:*.node",
            ])
            .current_dir(&fe_dir)
            .output()
            .with_context(|| format!("Failed to run esbuild for {}", package_name))?;

        // Clean up entry file
        let _ = std::fs::remove_file(&entry_path);

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            anyhow::bail!(
                "esbuild failed for {}: {}",
                package_name,
                stderr.trim()
            );
        }

        Ok(output_path)
    }

    pub fn get_dep_url(&self, package_name: &str) -> Option<String> {
        let map_path = self.cache_dir.join("dep-map.json");
        if let Ok(content) = std::fs::read_to_string(&map_path) {
            if let Ok(map) = serde_json::from_str::<DependencyMap>(&content) {
                return map.entries.get(package_name).cloned();
            }
        }
        None
    }

    pub fn invalidate_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

fn compute_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..4]) // 8 characters
}

fn sanitize_package_name(name: &str) -> String {
    name.replace('/', "__").replace('@', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_package_name() {
        assert_eq!(sanitize_package_name("react"), "react");
        assert_eq!(sanitize_package_name("react-dom"), "react-dom");
        assert_eq!(sanitize_package_name("@scope/package"), "scope__package");
    }

    #[test]
    fn test_compute_short_hash() {
        let hash1 = compute_short_hash("react");
        let hash2 = compute_short_hash("react-dom");
        assert_ne!(hash1, hash2);
        assert_eq!(hash1.len(), 8);
    }
}
