use crate::deps::DependencyPrebundler;
use crate::module_graph::SharedModuleGraph;
use crate::server::HmrBroadcaster;
use crate::transform::{
    TransformConfig, TransformPipeline, get_react_refresh_preamble, inject_hmr_code,
    inject_react_refresh_code,
};
use anyhow::Result;
use bytes::Bytes;
use http_body_util::Full;
use http_body_util::combinators::BoxBody;
use hyper::{Response, StatusCode};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct TransformServer {
    pipeline: TransformPipeline,
    module_graph: SharedModuleGraph,
    prebundler: Arc<Mutex<DependencyPrebundler>>,
    hmr: HmrBroadcaster,
    fe_dir: PathBuf,
    cache_dir: PathBuf,
}

impl TransformServer {
    pub fn new(
        project_root: &Path,
        prebundler: Arc<Mutex<DependencyPrebundler>>,
        hmr: HmrBroadcaster,
    ) -> Self {
        let config = TransformConfig::new(project_root);
        let pipeline = TransformPipeline::new(config);
        let module_graph = SharedModuleGraph::new(project_root);
        let fe_dir = project_root.join("fe");
        let cache_dir = project_root.join("fe/.forte");

        Self {
            pipeline,
            module_graph,
            prebundler,
            hmr,
            fe_dir,
            cache_dir,
        }
    }

    pub async fn serve_module(
        &self,
        path: &str,
    ) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
        if path.starts_with("/.forte/deps/") {
            return self.serve_bundled_dep(path).await;
        }

        if path == "/__hmr-client.js" {
            return self.serve_hmr_client().await;
        }

        if path == "/__react-refresh.js" {
            return self.serve_react_refresh_preamble().await;
        }

        if path == "/__error-overlay.js" {
            return self.serve_error_overlay().await;
        }

        if path.starts_with("/src/") {
            if path.contains(".css") {
                return self.serve_css(path).await;
            }
            return self.serve_transformed_module(path).await;
        }

        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(full_body("Not Found"))?)
    }

    async fn serve_transformed_module(
        &self,
        path: &str,
    ) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
        let clean_path = path.split('?').next().unwrap_or(path);
        let file_path = self.resolve_path(clean_path)?;

        if !file_path.exists() {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full_body(format!("File not found: {}", clean_path)))?);
        }

        let result = self.pipeline.transform(&file_path)?;

        let module_id = self
            .module_graph
            .file_to_module_id(&file_path)
            .unwrap_or_else(|| path.to_string());

        let imports: Vec<String> = result
            .imports
            .iter()
            .filter(|i| i.kind == crate::transform::ImportKind::Relative)
            .map(|i| i.specifier.clone())
            .collect();

        self.module_graph
            .update_module(&module_id, &file_path, imports, &result.hash);

        let bare_imports = extract_bare_imports(&result.code);
        let mut new_deps_bundled = false;

        {
            let mut prebundler = self.prebundler.lock().unwrap();

            for import_path in &bare_imports {
                if !prebundler.get_dep_map().entries.contains_key(import_path) {
                    match prebundler.register_missing_import(import_path) {
                        Ok(Some(_url)) => {
                            new_deps_bundled = true;
                            tracing::info!(
                                "[deps] Dynamically bundled new dependency: {}",
                                import_path
                            );
                        }
                        Ok(None) => {}
                        Err(e) => {
                            tracing::warn!("Failed to bundle {}: {}", import_path, e);
                        }
                    }
                }
            }
        }

        if new_deps_bundled {
            self.hmr.send_reload();
        }

        let dep_entries = {
            let prebundler = self.prebundler.lock().unwrap();
            prebundler.get_dep_map().entries.clone()
        };

        let code = rewrite_imports(&result.code, &dep_entries);
        let code = rewrite_css_imports(&code);

        let code = if result.has_react_components {
            inject_react_refresh_code(&code, &module_id)
        } else {
            inject_hmr_code(&code, &module_id, false)
        };

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/javascript; charset=utf-8")
            .header("cache-control", "no-cache")
            .body(full_body(code))?)
    }

    async fn serve_css(
        &self,
        path: &str,
    ) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
        let has_import_query = path
            .split('?')
            .nth(1)
            .map(|q| q.split('&').any(|p| p == "import" || p.starts_with("import=")))
            .unwrap_or(false);
        let clean_path = path.split('?').next().unwrap_or(path);

        let file_path = if clean_path == "/src/styles/globals.css" {
            let built_path = self.cache_dir.join("styles/globals.css");
            if built_path.exists() {
                built_path
            } else {
                self.resolve_path(clean_path)?
            }
        } else {
            self.resolve_path(clean_path)?
        };

        if !file_path.exists() {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full_body(format!("CSS file not found: {}", clean_path)))?);
        }

        let css_content = tokio::fs::read_to_string(&file_path).await?;

        if has_import_query {
            let js_code = generate_css_module(&css_content, clean_path);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/javascript; charset=utf-8")
                .header("cache-control", "no-cache")
                .body(full_body(js_code))?)
        } else {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/css; charset=utf-8")
                .header("cache-control", "no-cache")
                .body(full_body(css_content))?)
        }
    }

    async fn serve_bundled_dep(
        &self,
        path: &str,
    ) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
        let relative = path.strip_prefix("/.forte/deps/").unwrap_or(path);
        let file_path = self.cache_dir.join("deps").join(relative);

        match tokio::fs::read(&file_path).await {
            Ok(contents) => Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/javascript; charset=utf-8")
                .header("cache-control", "public, max-age=31536000, immutable")
                .body(full_body_bytes(contents.into()))?),
            Err(_) => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full_body(format!("Bundled dependency not found: {}", path)))?),
        }
    }

    async fn serve_hmr_client(&self) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
        let client_code = include_str!("../../assets/hmr-client.js");
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/javascript; charset=utf-8")
            .header("cache-control", "no-cache")
            .body(full_body(client_code.to_string()))?)
    }

    async fn serve_react_refresh_preamble(
        &self,
    ) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
        let preamble = get_react_refresh_preamble();
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/javascript; charset=utf-8")
            .header("cache-control", "no-cache")
            .body(full_body(preamble.to_string()))?)
    }

    async fn serve_error_overlay(&self) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
        let overlay_code = include_str!("../../assets/error-overlay.js");
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/javascript; charset=utf-8")
            .header("cache-control", "no-cache")
            .body(full_body(overlay_code.to_string()))?)
    }

    fn resolve_path(&self, url_path: &str) -> Result<PathBuf> {
        let relative = url_path.strip_prefix("/").unwrap_or(url_path);
        let path = self.fe_dir.join(relative);

        if path.exists() {
            return Ok(path);
        }

        if !has_known_js_extension(&path) {
            let path_str = path.to_string_lossy();
            for ext in &["tsx", "ts", "jsx", "js"] {
                let with_ext = PathBuf::from(format!("{}.{}", path_str, ext));
                if with_ext.exists() {
                    return Ok(with_ext);
                }
            }

            let index_path = path.join("index.tsx");
            if index_path.exists() {
                return Ok(index_path);
            }
        }

        Ok(path)
    }

    pub fn invalidate_module(&self, file_path: &Path) {
        self.pipeline.invalidate(file_path);
    }

    pub fn get_hmr_update(&self, file_path: &Path) -> Option<crate::module_graph::HmrUpdate> {
        let module_id = self.module_graph.file_to_module_id(file_path)?;
        Some(self.module_graph.get_hmr_update(&module_id))
    }
}

fn generate_css_module(css_content: &str, module_id: &str) -> String {
    let escaped_css = css_content
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    format!(
        r#"const css = `{escaped_css}`;
const styleId = "{module_id}";

function updateStyle() {{
  let style = document.getElementById(styleId);
  if (!style) {{
    style = document.createElement('style');
    style.id = styleId;
    style.setAttribute('data-forte-css', '');
    document.head.appendChild(style);
  }}
  style.textContent = css;
}}

updateStyle();

if (import.meta.hot) {{
  import.meta.hot = window.__forte_createHotContext(styleId);
  import.meta.hot.accept();
  import.meta.hot.dispose(() => {{
    const style = document.getElementById(styleId);
    if (style) style.remove();
  }});
}}

export default css;
"#
    )
}

fn rewrite_css_imports(code: &str) -> String {
    let re = regex::Regex::new(r#"import\s+["']([^"']+\.css)["']"#).unwrap();
    re.replace_all(code, |caps: &regex::Captures| {
        let css_path = &caps[1];
        format!(r#"import "{}""#, css_path)
    })
    .to_string()
}

fn has_known_js_extension(path: &Path) -> bool {
    let known_extensions = ["tsx", "ts", "jsx", "js", "mjs", "cjs", "json"];
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| known_extensions.contains(&e))
        .unwrap_or(false)
}

fn full_body(s: impl Into<String>) -> BoxBody<Bytes, anyhow::Error> {
    use http_body_util::BodyExt;
    Full::new(Bytes::from(s.into()))
        .map_err(|e| anyhow::anyhow!("{e}"))
        .boxed()
}

fn full_body_bytes(bytes: Bytes) -> BoxBody<Bytes, anyhow::Error> {
    use http_body_util::BodyExt;
    Full::new(bytes).map_err(|e| anyhow::anyhow!("{e}")).boxed()
}

fn extract_bare_imports(code: &str) -> HashSet<String> {
    let mut imports = HashSet::new();
    let re = regex::Regex::new(
        r#"(?:import|export)\s+.*?from\s*["']([^"']+)["']|import\s*\(\s*["']([^"']+)["']\s*\)|import\s+["']([^"']+)["']"#,
    )
    .unwrap();

    for cap in re.captures_iter(code) {
        let specifier = cap.get(1).or(cap.get(2)).or(cap.get(3));
        if let Some(s) = specifier {
            let import_path = s.as_str();
            if !import_path.starts_with('.') && !import_path.starts_with('/') {
                imports.insert(import_path.to_string());
            }
        }
    }

    imports
}

fn get_package_name(specifier: &str) -> String {
    if specifier.starts_with('@') {
        let parts: Vec<&str> = specifier.splitn(3, '/').collect();
        if parts.len() >= 2 {
            return format!("{}/{}", parts[0], parts[1]);
        }
    }
    specifier.split('/').next().unwrap_or(specifier).to_string()
}

fn rewrite_imports(
    code: &str,
    dep_entries: &std::collections::HashMap<String, String>,
) -> String {
    let patterns = [
        (r#"from\s*["']([^"']+)["']"#, "from"),
        (r#"import\s*\(\s*["']([^"']+)["']\s*\)"#, "dynamic"),
        (r#"import\s+["']([^"']+)["']"#, "sideeffect"),
    ];

    let mut result = code.to_string();

    for (pattern, kind) in patterns {
        let re = regex::Regex::new(pattern).unwrap();
        let mut offset: i64 = 0;

        let current_code = result.clone();
        let matches: Vec<_> = re.captures_iter(&current_code).collect();
        for cap in matches {
            let full_match = cap.get(0).unwrap();
            let specifier = cap.get(1).unwrap().as_str();

            if specifier.starts_with('.') || specifier.starts_with('/') {
                continue;
            }

            let resolved = if let Some(url) = dep_entries.get(specifier) {
                Some(url.clone())
            } else {
                let package_name = get_package_name(specifier);
                dep_entries.get(&package_name).cloned()
            };

            if let Some(new_path) = resolved {
                let replacement = match kind {
                    "from" => format!(r#"from "{}""#, new_path),
                    "dynamic" => format!(r#"import("{}")"#, new_path),
                    "sideeffect" => format!(r#"import "{}""#, new_path),
                    _ => continue,
                };

                let start = (full_match.start() as i64 + offset) as usize;
                let end = (full_match.end() as i64 + offset) as usize;

                result.replace_range(start..end, &replacement);
                offset += replacement.len() as i64 - full_match.len() as i64;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_css_module() {
        let css = ".button { color: red; }";
        let result = generate_css_module(css, "/src/button.css");

        assert!(result.contains(".button { color: red; }"));
        assert!(result.contains("import.meta.hot"));
        assert!(result.contains("/src/button.css"));
    }

    #[test]
    fn test_css_escaping() {
        let css = ".icon::before { content: `test`; color: ${var}; }";
        let result = generate_css_module(css, "/src/icon.css");

        assert!(result.contains("\\`test\\`"));
        assert!(result.contains("\\${var}"));
    }
}
