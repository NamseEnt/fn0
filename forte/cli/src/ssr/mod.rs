use anyhow::{Context, Result};
use std::collections::HashMap;
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

    fn load_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        let env_path = self.project_root.join(".env");

        if let Ok(content) = std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    if key.starts_with("PUBLIC_") {
                        env_vars.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        env_vars
    }

    fn generate_define_config(&self, env_vars: &HashMap<String, String>) -> String {
        let mut defines = vec![
            r#""import.meta.env.SSR": "true""#.to_string(),
            r#""import.meta.env.DEV": "true""#.to_string(),
        ];

        for (key, value) in env_vars {
            let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");
            defines.push(format!(
                r#""import.meta.env.{}": "'{}'""#,
                key, escaped
            ));
        }

        format!(
            "  transform: {{\n    define: {{\n      {}\n    }}\n  }},",
            defines.join(",\n      ")
        )
    }

    pub fn build(&self) -> Result<PathBuf> {
        let fe_dir = self.project_root.join("fe");
        let entry_path = fe_dir.join("src/server.tsx");

        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        println!("[ssr] Building server bundle...");

        let env_vars = self.load_env_vars();
        let define_config = self.generate_define_config(&env_vars);

        let config_path = self.output_path.parent().unwrap().join("rolldown.config.mjs");
        let config_content = format!(
            r#"// SSR CSS Plugin - returns empty export for CSS files (no DOM in SSR)
function ssrCssPlugin() {{
  return {{
    name: 'ssr-css',
    transform(code, id) {{
      if (id.endsWith('.css')) {{
        return {{ code: 'export default "";', map: null }};
      }}
      return null;
    }}
  }};
}}

export default {{
  input: "{}",
  output: {{
    file: "{}",
    format: "iife",
    inlineDynamicImports: true,
    sourcemap: 'inline',
  }},
  tsconfig: "tsconfig.json",
  plugins: [ssrCssPlugin()],
{define_config}
}};
"#,
            entry_path.to_str().unwrap(),
            self.output_path.to_str().unwrap()
        );
        std::fs::write(&config_path, &config_content)?;

        let result = Command::new("npx")
            .args(["rolldown", "-c", config_path.to_str().unwrap()])
            .current_dir(&fe_dir)
            .output()
            .context("Failed to run rolldown for SSR bundle")?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            anyhow::bail!("rolldown SSR bundle failed: {}", stderr.trim());
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
        assert!(
            bundler
                .output_path()
                .to_string_lossy()
                .contains(".forte/ssr/server.js")
        );
    }
}
