use crate::deps::DependencyMap;
use crate::module_graph::SharedModuleGraph;
use crate::transform::{
    get_react_refresh_preamble, inject_hmr_code, inject_react_refresh_code, rewrite_imports,
    TransformConfig, TransformPipeline,
};
use anyhow::Result;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use std::path::{Path, PathBuf};

pub struct TransformServer {
    pipeline: TransformPipeline,
    module_graph: SharedModuleGraph,
    dep_map: DependencyMap,
    fe_dir: PathBuf,
    cache_dir: PathBuf,
}

impl TransformServer {
    pub fn new(project_root: &Path, dep_map: DependencyMap) -> Self {
        let config = TransformConfig::new(project_root);
        let pipeline = TransformPipeline::new(config);
        let module_graph = SharedModuleGraph::new(project_root);
        let fe_dir = project_root.join("fe");
        let cache_dir = project_root.join("fe/.forte");

        Self {
            pipeline,
            module_graph,
            dep_map,
            fe_dir,
            cache_dir,
        }
    }

    pub async fn serve_module(&self, path: &str) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
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
            if path.ends_with(".css") {
                return self.serve_css_module(path).await;
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
        let file_path = self.resolve_path(path)?;

        if !file_path.exists() {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full_body(format!("File not found: {}", path)))?);
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

        let code = rewrite_imports(&result.code, &self.dep_map.entries);
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

    async fn serve_css_module(
        &self,
        path: &str,
    ) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
        let file_path = self.resolve_path(path)?;

        if !file_path.exists() {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full_body(format!("CSS file not found: {}", path)))?);
        }

        let css_content = tokio::fs::read_to_string(&file_path).await?;
        let module_id = path.to_string();

        let js_code = generate_css_module(&css_content, &module_id);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/javascript; charset=utf-8")
            .header("cache-control", "no-cache")
            .body(full_body(js_code))?)
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
                .body(full_body(format!(
                    "Bundled dependency not found: {}",
                    path
                )))?),
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

    async fn serve_react_refresh_preamble(&self) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
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

        if !has_extension(&path) {
            for ext in &["tsx", "ts", "jsx", "js"] {
                let with_ext = path.with_extension(ext);
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

fn has_extension(path: &Path) -> bool {
    path.extension().is_some()
}

fn full_body(s: impl Into<String>) -> BoxBody<Bytes, anyhow::Error> {
    use http_body_util::BodyExt;
    Full::new(Bytes::from(s.into()))
        .map_err(|e| anyhow::anyhow!("{e}"))
        .boxed()
}

fn full_body_bytes(bytes: Bytes) -> BoxBody<Bytes, anyhow::Error> {
    use http_body_util::BodyExt;
    Full::new(bytes)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .boxed()
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
