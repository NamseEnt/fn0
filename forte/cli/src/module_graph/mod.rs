use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Module {
    pub id: String,
    pub file_path: PathBuf,
    pub importers: HashSet<String>,
    pub imports: HashSet<String>,
    pub accepts_hmr: bool,
    pub is_self_accepting: bool,
    pub hash: String,
}

impl Module {
    pub fn new(id: String, file_path: PathBuf) -> Self {
        Self {
            id,
            file_path,
            importers: HashSet::new(),
            imports: HashSet::new(),
            accepts_hmr: false,
            is_self_accepting: false,
            hash: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HmrUpdate {
    pub boundary: String,
    pub chain: Vec<String>,
    pub needs_full_reload: bool,
}

pub struct ModuleGraph {
    modules: HashMap<String, Module>,
    project_root: PathBuf,
    src_dir: PathBuf,
}

impl ModuleGraph {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let src_dir = project_root.join("fe/src");
        Self {
            modules: HashMap::new(),
            project_root,
            src_dir,
        }
    }

    pub fn file_to_module_id(&self, file_path: &Path) -> Option<String> {
        if let Ok(relative) = file_path.strip_prefix(&self.src_dir) {
            Some(format!("/src/{}", relative.to_string_lossy()))
        } else {
            None
        }
    }

    pub fn module_id_to_file(&self, module_id: &str) -> Option<PathBuf> {
        if module_id.starts_with("/src/") {
            let relative = &module_id[5..]; // Remove "/src/"
            Some(self.src_dir.join(relative))
        } else {
            None
        }
    }

    pub fn update_module(&mut self, module_id: &str, file_path: &Path, imports: Vec<String>, hash: &str) {
        // Remove old import relationships
        if let Some(old_module) = self.modules.get(module_id) {
            let old_imports = old_module.imports.clone();
            for import in old_imports {
                if let Some(imported_module) = self.modules.get_mut(&import) {
                    imported_module.importers.remove(module_id);
                }
            }
        }

        // Get or create module
        let module = self.modules.entry(module_id.to_string()).or_insert_with(|| {
            Module::new(module_id.to_string(), file_path.to_path_buf())
        });

        // Update module info
        module.file_path = file_path.to_path_buf();
        module.hash = hash.to_string();
        module.imports = imports.iter().cloned().collect();

        // By default, all React component files are self-accepting for HMR
        // This will be refined when we have better component detection
        module.accepts_hmr = true;
        module.is_self_accepting = true;

        // Update importer relationships
        let module_id_owned = module_id.to_string();
        for import in &imports {
            // Create imported module if it doesn't exist
            let file_path = self.module_id_to_file(import).unwrap_or_default();
            let imported = self.modules.entry(import.clone()).or_insert_with(|| {
                Module::new(import.clone(), file_path.clone())
            });
            imported.importers.insert(module_id_owned.clone());
        }
    }

    pub fn get_module(&self, module_id: &str) -> Option<&Module> {
        self.modules.get(module_id)
    }

    pub fn remove_module(&mut self, module_id: &str) {
        if let Some(module) = self.modules.remove(module_id) {
            // Remove from importers of all imports
            for import in &module.imports {
                if let Some(imported) = self.modules.get_mut(import) {
                    imported.importers.remove(module_id);
                }
            }
            // Remove from imports of all importers
            for importer in &module.importers {
                if let Some(importer_module) = self.modules.get_mut(importer) {
                    importer_module.imports.remove(module_id);
                }
            }
        }
    }

    pub fn get_hmr_update(&self, changed_module_id: &str) -> HmrUpdate {
        // Start from the changed module and walk up the importer chain
        // Stop when we find a module that accepts HMR (boundary)

        let mut visited = HashSet::new();
        let mut chain = Vec::new();
        let mut queue = vec![changed_module_id.to_string()];
        let mut boundary: Option<String> = None;

        while let Some(current_id) = queue.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());
            chain.push(current_id.clone());

            if let Some(module) = self.modules.get(&current_id) {
                // If this module accepts HMR, it's a boundary
                if module.is_self_accepting {
                    boundary = Some(current_id.clone());
                    break;
                }

                // Otherwise, propagate to importers
                for importer in &module.importers {
                    if !visited.contains(importer) {
                        queue.push(importer.clone());
                    }
                }
            }
        }

        // If we found a boundary, return the update
        if let Some(boundary_id) = boundary {
            HmrUpdate {
                boundary: boundary_id,
                chain,
                needs_full_reload: false,
            }
        } else {
            // No boundary found - need full reload
            HmrUpdate {
                boundary: String::new(),
                chain,
                needs_full_reload: true,
            }
        }
    }

    pub fn get_affected_modules(&self, changed_module_id: &str) -> Vec<String> {
        let update = self.get_hmr_update(changed_module_id);
        update.chain
    }

    pub fn clear(&mut self) {
        self.modules.clear();
    }
}

pub struct SharedModuleGraph {
    inner: Arc<RwLock<ModuleGraph>>,
}

impl SharedModuleGraph {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ModuleGraph::new(project_root))),
        }
    }

    pub fn update_module(&self, module_id: &str, file_path: &Path, imports: Vec<String>, hash: &str) {
        self.inner.write().unwrap().update_module(module_id, file_path, imports, hash);
    }

    pub fn get_hmr_update(&self, changed_module_id: &str) -> HmrUpdate {
        self.inner.read().unwrap().get_hmr_update(changed_module_id)
    }

    pub fn file_to_module_id(&self, file_path: &Path) -> Option<String> {
        self.inner.read().unwrap().file_to_module_id(file_path)
    }

    pub fn clear(&self) {
        self.inner.write().unwrap().clear();
    }
}

impl Clone for SharedModuleGraph {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_graph_basic() {
        let mut graph = ModuleGraph::new("/tmp/project");

        // Add modules
        graph.update_module(
            "/src/App.tsx",
            Path::new("/tmp/project/fe/src/App.tsx"),
            vec!["/src/components/Button.tsx".to_string()],
            "hash1",
        );

        graph.update_module(
            "/src/components/Button.tsx",
            Path::new("/tmp/project/fe/src/components/Button.tsx"),
            vec![],
            "hash2",
        );

        // Check relationships
        let app = graph.get_module("/src/App.tsx").unwrap();
        assert!(app.imports.contains("/src/components/Button.tsx"));

        let button = graph.get_module("/src/components/Button.tsx").unwrap();
        assert!(button.importers.contains("/src/App.tsx"));
    }

    #[test]
    fn test_hmr_boundary() {
        let mut graph = ModuleGraph::new("/tmp/project");

        // App imports Button, Button is self-accepting
        graph.update_module(
            "/src/App.tsx",
            Path::new("/tmp/project/fe/src/App.tsx"),
            vec!["/src/components/Button.tsx".to_string()],
            "hash1",
        );

        graph.update_module(
            "/src/components/Button.tsx",
            Path::new("/tmp/project/fe/src/components/Button.tsx"),
            vec![],
            "hash2",
        );

        // When Button changes, it should be its own boundary (self-accepting)
        let update = graph.get_hmr_update("/src/components/Button.tsx");
        assert!(!update.needs_full_reload);
        assert_eq!(update.boundary, "/src/components/Button.tsx");
    }

    #[test]
    fn test_file_to_module_id() {
        let graph = ModuleGraph::new("/tmp/project");

        let id = graph.file_to_module_id(Path::new("/tmp/project/fe/src/components/Button.tsx"));
        assert_eq!(id, Some("/src/components/Button.tsx".to_string()));
    }
}
