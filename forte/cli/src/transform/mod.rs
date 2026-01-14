pub mod import_rewrite;
mod inject;
mod oxc;

pub use import_rewrite::*;
pub use inject::*;
pub use oxc::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    pub code: String,
    pub map: Option<String>,
    pub imports: Vec<ImportInfo>,
    pub hash: String,
    pub has_react_components: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub specifier: String,
    pub kind: ImportKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportKind {
    Bare,
    Relative,
    Alias,
}

impl ImportInfo {
    pub fn classify(specifier: &str) -> ImportKind {
        if specifier.starts_with("./") || specifier.starts_with("../") {
            ImportKind::Relative
        } else if specifier.starts_with('@') && specifier.contains('/') {
            // Could be scoped package (@scope/pkg) or alias (@/path)
            // Check if it looks like an alias (starts with @/ or @alias/)
            let parts: Vec<&str> = specifier.splitn(2, '/').collect();
            if parts.len() == 2 && (parts[0] == "@" || !parts[0].contains('@')) {
                // @/path or @alias/path patterns
                if parts[0].len() == 1 || !parts[1].contains('/') {
                    ImportKind::Alias
                } else {
                    ImportKind::Bare
                }
            } else {
                ImportKind::Bare
            }
        } else if specifier.starts_with('@') {
            ImportKind::Alias
        } else {
            ImportKind::Bare
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub project_root: PathBuf,
    pub fe_src_dir: PathBuf,
    pub react_refresh: bool,
    pub development: bool,
    pub aliases: HashMap<String, PathBuf>,
}

impl TransformConfig {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let fe_src_dir = project_root.join("fe/src");

        let mut aliases = HashMap::new();
        aliases.insert("@".to_string(), fe_src_dir.clone());

        Self {
            project_root,
            fe_src_dir,
            react_refresh: true,
            development: true,
            aliases,
        }
    }

    pub fn with_react_refresh(mut self, enabled: bool) -> Self {
        self.react_refresh = enabled;
        self
    }
}

pub struct TransformPipeline {
    config: TransformConfig,
    cache: Arc<RwLock<HashMap<PathBuf, CachedTransform>>>,
}

#[derive(Debug, Clone)]
struct CachedTransform {
    result: TransformResult,
    mtime: std::time::SystemTime,
}

impl TransformPipeline {
    pub fn new(config: TransformConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn transform(&self, file_path: &Path) -> Result<TransformResult> {
        // Check cache first
        if let Some(cached) = self.get_cached(file_path)? {
            return Ok(cached);
        }

        // Read source file
        let source_text = std::fs::read_to_string(file_path)?;
        let hash = compute_hash(&source_text);

        // Transform using OXC
        let result = transform_with_oxc(file_path, &source_text, &self.config)?;

        // Update cache
        let cached = CachedTransform {
            result: TransformResult {
                code: result.code,
                map: result.map,
                imports: result.imports,
                hash,
                has_react_components: result.has_react_components,
            },
            mtime: std::fs::metadata(file_path)?.modified()?,
        };

        self.cache
            .write()
            .unwrap()
            .insert(file_path.to_path_buf(), cached.clone());

        Ok(cached.result)
    }

    pub fn invalidate(&self, file_path: &Path) {
        self.cache.write().unwrap().remove(file_path);
    }

    pub fn invalidate_all(&self) {
        self.cache.write().unwrap().clear();
    }

    fn get_cached(&self, file_path: &Path) -> Result<Option<TransformResult>> {
        let cache = self.cache.read().unwrap();
        if let Some(cached) = cache.get(file_path) {
            let current_mtime = std::fs::metadata(file_path)?.modified()?;
            if current_mtime == cached.mtime {
                return Ok(Some(cached.result.clone()));
            }
        }
        Ok(None)
    }

    pub fn config(&self) -> &TransformConfig {
        &self.config
    }
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8]) // Use first 8 bytes for shorter hash
}
