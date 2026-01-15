use deno_core::error::ModuleLoaderError;
use deno_core::{
    ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse, ModuleLoader, ModuleSource,
    ModuleSourceCode, ModuleSpecifier, ModuleType, ResolutionKind,
};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::source_map::extract_source_map;

pub trait ModuleFetcher: Send + Sync + 'static {
    fn fetch(&self, specifier: &str) -> Result<FetchedModule, String>;
}

pub struct FetchedModule {
    pub code: String,
    pub source_map: Option<Vec<u8>>,
}

pub struct SsrModuleLoader {
    fetcher: Arc<dyn ModuleFetcher>,
    source_maps: RefCell<HashMap<String, Vec<u8>>>,
    base_url: String,
}

impl SsrModuleLoader {
    pub fn new(fetcher: Arc<dyn ModuleFetcher>, base_url: String) -> Self {
        Self {
            fetcher,
            source_maps: RefCell::new(HashMap::new()),
            base_url,
        }
    }

    fn resolve_specifier(&self, specifier: &str, referrer: &str) -> Result<String, String> {
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            return Ok(specifier.to_string());
        }

        if specifier.starts_with("./") || specifier.starts_with("../") {
            let referrer_url = if referrer.starts_with("http://") || referrer.starts_with("https://")
            {
                referrer.to_string()
            } else {
                format!("{}{}", self.base_url, referrer)
            };

            let base = url::Url::parse(&referrer_url).map_err(|e| e.to_string())?;
            let resolved = base.join(specifier).map_err(|e| e.to_string())?;
            return Ok(resolved.to_string());
        }

        if specifier.starts_with('/') {
            return Ok(format!("{}{}", self.base_url, specifier));
        }

        Ok(format!("{}/node_modules/{}", self.base_url, specifier))
    }
}

impl ModuleLoader for SsrModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, ModuleLoaderError> {
        let resolved = self
            .resolve_specifier(specifier, referrer)
            .map_err(|e| ModuleLoaderError::generic(format!("Failed to resolve {}: {}", specifier, e)))?;

        ModuleSpecifier::parse(&resolved)
            .map_err(|e| ModuleLoaderError::generic(format!("Invalid URL {}: {}", resolved, e)))
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        let specifier = module_specifier.to_string();
        let fetcher = self.fetcher.clone();

        let result = fetcher.fetch(&specifier);

        match result {
            Ok(module) => {
                let source_map = module
                    .source_map
                    .or_else(|| extract_source_map(module.code.as_bytes()));

                if let Some(sm) = source_map {
                    self.source_maps
                        .borrow_mut()
                        .insert(specifier.clone(), sm);
                }

                let module_type = if specifier.ends_with(".json") {
                    ModuleType::Json
                } else {
                    ModuleType::JavaScript
                };

                ModuleLoadResponse::Sync(Ok(ModuleSource::new(
                    module_type,
                    ModuleSourceCode::String(module.code.into()),
                    module_specifier,
                    None,
                )))
            }
            Err(e) => ModuleLoadResponse::Sync(Err(ModuleLoaderError::generic(format!(
                "Failed to load module {}: {}",
                specifier, e
            )))),
        }
    }

    fn get_source_map(&self, file_name: &str) -> Option<Cow<'_, [u8]>> {
        self.source_maps
            .borrow()
            .get(file_name)
            .map(|v| Cow::Owned(v.clone()))
    }

    fn source_map_source_exists(&self, _source_url: &str) -> Option<bool> {
        Some(true)
    }
}
