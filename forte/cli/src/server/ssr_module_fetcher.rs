use crate::deps::DependencyPrebundler;
use crate::server::ssr_entry::generate_ssr_entry_code;
use crate::transform::{ImportRewriter, TransformConfig, TransformPipeline};
use ski::{FetchedModule, ModuleFetcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const SSR_ENTRY_PATH: &str = "/__ssr_entry__.js";

pub struct SsrModuleFetcher {
    pipeline: TransformPipeline,
    prebundler: Arc<Mutex<DependencyPrebundler>>,
    project_root: PathBuf,
    fe_dir: PathBuf,
    env_vars: HashMap<String, String>,
}

impl SsrModuleFetcher {
    pub fn new(project_root: &Path, prebundler: Arc<Mutex<DependencyPrebundler>>) -> Self {
        let config = TransformConfig::new(project_root)
            .with_react_refresh(false)
            .with_development(false);
        let pipeline = TransformPipeline::new(config);
        let fe_dir = project_root.join("fe");
        let env_vars = load_env_vars(project_root);

        Self {
            pipeline,
            prebundler,
            project_root: project_root.to_path_buf(),
            fe_dir,
            env_vars,
        }
    }

    fn url_to_file_path(&self, url: &str) -> Option<PathBuf> {
        let path = if url.starts_with("http://") || url.starts_with("https://") {
            let url_obj = url::Url::parse(url).ok()?;
            url_obj.path().to_string()
        } else {
            url.to_string()
        };

        let clean_path = path.split('?').next().unwrap_or(&path);

        if clean_path.starts_with("/src/") {
            let relative = clean_path.strip_prefix("/").unwrap_or(clean_path);
            let file_path = self.fe_dir.join(relative);
            if file_path.exists() {
                return Some(file_path);
            }
            return self.resolve_path_with_extensions(&file_path);
        }

        if clean_path.starts_with("/.forte/deps/") {
            let relative = clean_path.strip_prefix("/.forte/deps/").unwrap_or(clean_path);
            let file_path = self.fe_dir.join(".forte/deps").join(relative);
            if file_path.exists() {
                return Some(file_path);
            }
        }

        None
    }

    fn resolve_path_with_extensions(&self, base_path: &Path) -> Option<PathBuf> {
        if base_path.exists() {
            return Some(base_path.to_path_buf());
        }

        let path_str = base_path.to_string_lossy();
        for ext in &["tsx", "ts", "jsx", "js"] {
            let with_ext = PathBuf::from(format!("{}.{}", path_str, ext));
            if with_ext.exists() {
                return Some(with_ext);
            }
        }

        let index_path = base_path.join("index.tsx");
        if index_path.exists() {
            return Some(index_path);
        }

        None
    }

    fn transform_for_ssr(&self, file_path: &Path) -> Result<FetchedModule, String> {
        let result = self
            .pipeline
            .transform(file_path)
            .map_err(|e| format!("Transform error: {}", e))?;

        let dep_map = {
            let prebundler = self.prebundler.lock().unwrap();
            prebundler.get_dep_map().clone()
        };

        let rewriter = ImportRewriter::new(dep_map, Some(&self.project_root));
        let relative_path = file_path
            .strip_prefix(&self.fe_dir.join("src"))
            .unwrap_or(file_path);
        let code = rewriter.rewrite(&result.code, relative_path, 0);
        let code = rewrite_css_imports_for_ssr(&code);
        let code = replace_env_vars_for_ssr(&code, &self.env_vars);

        let source_map = result.map.map(|s| s.into_bytes());

        Ok(FetchedModule { code, source_map })
    }

    fn load_bundled_dep(&self, path: &str) -> Result<FetchedModule, String> {
        let relative = path.strip_prefix("/.forte/deps/").unwrap_or(path);
        let file_path = self.fe_dir.join(".forte/deps").join(relative);

        let code = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read bundled dep {}: {}", path, e))?;

        let source_map = crate::source_map_utils::extract_inline_source_map(&code);

        Ok(FetchedModule { code, source_map })
    }

    fn load_css_module(&self, path: &str) -> Result<FetchedModule, String> {
        let clean_path = path.split('?').next().unwrap_or(path);
        let file_path = if clean_path == "/src/styles/globals.css" {
            let built_path = self.fe_dir.join(".forte/styles/globals.css");
            if built_path.exists() {
                built_path
            } else {
                let relative = clean_path.strip_prefix("/").unwrap_or(clean_path);
                self.fe_dir.join(relative)
            }
        } else {
            let relative = clean_path.strip_prefix("/").unwrap_or(clean_path);
            self.fe_dir.join(relative)
        };

        let css_content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read CSS {}: {}", path, e))?;

        let code = generate_ssr_css_module(&css_content);

        Ok(FetchedModule {
            code,
            source_map: None,
        })
    }
}

impl ModuleFetcher for SsrModuleFetcher {
    fn fetch(&self, specifier: &str) -> Result<FetchedModule, String> {
        let path = if specifier.starts_with("http://") || specifier.starts_with("https://") {
            let url_obj = url::Url::parse(specifier).map_err(|e| e.to_string())?;
            url_obj.path().to_string()
        } else {
            specifier.to_string()
        };

        if path == SSR_ENTRY_PATH {
            let code = generate_ssr_entry_code("/src/server.tsx");
            return Ok(FetchedModule {
                code,
                source_map: None,
            });
        }

        if path.starts_with("/.forte/deps/") {
            return self.load_bundled_dep(&path);
        }

        if path.contains(".css") {
            return self.load_css_module(&path);
        }

        if path.starts_with("/src/") {
            let file_path = self
                .url_to_file_path(specifier)
                .ok_or_else(|| format!("Cannot resolve path: {}", specifier))?;
            return self.transform_for_ssr(&file_path);
        }

        Err(format!("Unknown module path: {}", specifier))
    }
}

fn rewrite_css_imports_for_ssr(code: &str) -> String {
    let re = regex::Regex::new(r#"import\s+["']([^"']+\.css)["']"#).unwrap();
    re.replace_all(code, |caps: &regex::Captures| {
        let css_path = &caps[1];
        if css_path.contains('?') {
            format!(r#"import "{}&import&ssr""#, css_path)
        } else {
            format!(r#"import "{}?import&ssr""#, css_path)
        }
    })
    .to_string()
}

fn replace_env_vars_for_ssr(code: &str, env_vars: &HashMap<String, String>) -> String {
    let mut result = code.to_string();

    result = result.replace("import.meta.env.SSR", "true");
    result = result.replace("import.meta.env.DEV", "true");

    for (key, value) in env_vars {
        let pattern = format!("import.meta.env.{}", key);
        let replacement = format!("\"{}\"", value);
        result = result.replace(&pattern, &replacement);
    }

    result
}

fn generate_ssr_css_module(css_content: &str) -> String {
    let escaped_css = css_content
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    format!(r#"export default `{escaped_css}`;"#)
}

fn load_env_vars(project_root: &Path) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    let env_path = project_root.join(".env");

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
