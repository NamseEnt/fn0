use anyhow::{Context, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ArrayExpressionElement, CallExpression, ChainElement, ClassElement, Declaration, Expression,
    ForStatementInit, ObjectPropertyKind, Program, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

fn flatten_id(id: &str) -> String {
    id.replace('/', "__").replace('@', "_")
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyMap {
    pub entries: HashMap<String, String>,
    pub cjs_modules: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheMetadata {
    lockfile_hash: String,
    config_hash: String,
    entries: HashMap<String, String>,
    #[serde(default)]
    cjs_modules: HashSet<String>,
}

pub struct DependencyPrebundler {
    project_root: PathBuf,
    cache_dir: PathBuf,
    dep_map: DependencyMap,
}

impl DependencyPrebundler {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let cache_dir = project_root.join("fe/.forte/deps");
        Self {
            project_root,
            cache_dir,
            dep_map: DependencyMap::default(),
        }
    }

    pub fn prebundle(&mut self) -> Result<DependencyMap> {
        std::fs::create_dir_all(&self.cache_dir)?;

        if !self.should_rebuild() {
            if let Some(metadata) = self.load_metadata() {
                tracing::info!("[deps] Using cached dependencies");
                self.dep_map = DependencyMap {
                    entries: metadata.entries,
                    cjs_modules: metadata.cjs_modules,
                };
                return Ok(self.dep_map.clone());
            }
        }

        tracing::info!("[deps] Rebuilding dependency cache");

        let package_json_path = self.project_root.join("fe/package.json");
        let package_json: PackageJson = serde_json::from_str(
            &std::fs::read_to_string(&package_json_path)
                .with_context(|| format!("Failed to read {:?}", package_json_path))?,
        )?;

        self.dep_map = DependencyMap::default();

        let mut all_imports = self.scan_imports_from_source()?;

        if let Some(deps) = &package_json.dependencies {
            for dep_name in deps.keys() {
                all_imports.insert(dep_name.clone());
            }
        }

        let mut packages_to_bundle: Vec<String> = Vec::new();
        for import_path in all_imports {
            if import_path.starts_with('.') || import_path.starts_with('/') {
                continue;
            }

            let base_package_name = get_package_name(&import_path);
            let is_installed = package_json
                .dependencies
                .as_ref()
                .map(|d| d.contains_key(&base_package_name))
                .unwrap_or(false);

            if is_installed {
                packages_to_bundle.push(import_path);
            }
        }

        if packages_to_bundle.iter().any(|p| p == "react") {
            packages_to_bundle.push("react/jsx-runtime".to_string());
        }

        let bundle_results = self.bundle_all_dependencies(&packages_to_bundle)?;

        for (package_name, (output_path, is_cjs)) in bundle_results {
            let url_path = format!(
                "/.forte/deps/bundled/{}",
                output_path.file_name().unwrap().to_string_lossy()
            );
            self.dep_map.entries.insert(package_name.clone(), url_path);
            if is_cjs {
                self.dep_map.cjs_modules.insert(package_name);
            }
        }

        if let Ok(output_path) = self.bundle_react_refresh_runtime() {
            let url_path = format!(
                "/.forte/deps/{}",
                output_path.file_name().unwrap().to_string_lossy()
            );
            self.dep_map
                .entries
                .insert("react-refresh/runtime".to_string(), url_path);
        }

        let map_path = self.cache_dir.join("dep-map.json");
        std::fs::write(&map_path, serde_json::to_string_pretty(&self.dep_map)?)?;

        let metadata = CacheMetadata {
            lockfile_hash: self.compute_lockfile_hash(),
            config_hash: self.compute_config_hash(),
            entries: self.dep_map.entries.clone(),
            cjs_modules: self.dep_map.cjs_modules.clone(),
        };
        self.save_metadata(&metadata)?;

        Ok(self.dep_map.clone())
    }

    fn scan_imports_from_source(&self) -> Result<HashSet<String>> {
        let mut imports = HashSet::new();
        let fe_src = self.project_root.join("fe/src");

        if !fe_src.exists() {
            return Ok(imports);
        }

        self.scan_directory_ast(&fe_src, &mut imports)?;
        Ok(imports)
    }

    fn scan_directory_ast(&self, dir: &Path, imports: &mut HashSet<String>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory_ast(&path, imports)?;
            } else if let Some(ext) = path.extension() {
                if ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx" {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let file_imports = scan_imports_ast(&content, &path);
                        imports.extend(file_imports);
                    }
                }
            }
        }
        Ok(())
    }

    fn bundle_all_dependencies(
        &self,
        packages: &[String],
    ) -> Result<HashMap<String, (PathBuf, bool)>> {
        if packages.is_empty() {
            return Ok(HashMap::new());
        }

        let entries_dir = self.cache_dir.join("_entries");
        std::fs::create_dir_all(&entries_dir)?;

        let output_dir = self.cache_dir.join("bundled");
        std::fs::create_dir_all(&output_dir)?;

        let mut entry_paths: Vec<String> = Vec::new();
        let mut package_to_flat_id: HashMap<String, String> = HashMap::new();

        for package_name in packages {
            let flat_id = flatten_id(package_name);
            let entry_filename = format!("{}.js", flat_id);
            let entry_path = entries_dir.join(&entry_filename);

            let entry_content = format!(
                r#"export * from "{}";
import * as _mod from "{}";
export default _mod.default ?? _mod;"#,
                package_name, package_name
            );
            std::fs::write(&entry_path, &entry_content)?;

            entry_paths.push(entry_path.to_str().unwrap().to_string());
            package_to_flat_id.insert(package_name.clone(), flat_id);
        }

        let fe_dir = self.project_root.join("fe");

        let mut args = vec!["rolldown".to_string()];
        args.extend(entry_paths.iter().cloned());
        args.extend([
            "-d".to_string(),
            output_dir.to_str().unwrap().to_string(),
            "-f".to_string(),
            "esm".to_string(),
            "--platform".to_string(),
            "browser".to_string(),
        ]);

        let result = Command::new("npx")
            .args(&args)
            .current_dir(&fe_dir)
            .output()
            .context("Failed to run rolldown for unified bundling")?;

        let _ = std::fs::remove_dir_all(&entries_dir);

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            anyhow::bail!("rolldown unified bundling failed: {}", stderr.trim());
        }

        let mut results: HashMap<String, (PathBuf, bool)> = HashMap::new();

        for (package_name, flat_id) in &package_to_flat_id {
            let output_path = output_dir.join(format!("{}.js", flat_id));
            if output_path.exists() {
                self.patch_cjs_requires(&output_path, &package_to_flat_id)?;
                let is_cjs = self.detect_cjs_module(&output_path);
                results.insert(package_name.clone(), (output_path, is_cjs));
            }
        }

        println!(
            "[deps] Pre-bundled {} dependencies (unified)",
            results.len()
        );
        Ok(results)
    }

    fn patch_cjs_requires(
        &self,
        bundled_path: &Path,
        package_to_flat_id: &HashMap<String, String>,
    ) -> Result<()> {
        let content = std::fs::read_to_string(bundled_path)?;

        if !content.contains("__require(") {
            return Ok(());
        }

        let require_pattern = regex::Regex::new(r#"__require\("([^"]+)"\)"#).unwrap();

        let mut patched = content.clone();
        let mut imports_to_add: Vec<(String, String)> = Vec::new();

        for cap in require_pattern.captures_iter(&content) {
            let full_match = cap.get(0).unwrap().as_str();
            let pkg_name = cap.get(1).unwrap().as_str();

            if let Some(flat_id) = package_to_flat_id.get(pkg_name) {
                let import_var = format!("__vite_injected_{}", flat_id.replace('-', "_"));
                imports_to_add.push((pkg_name.to_string(), import_var.clone()));
                patched = patched.replace(full_match, &import_var);
            }
        }

        if imports_to_add.is_empty() {
            return Ok(());
        }

        imports_to_add.dedup_by(|a, b| a.0 == b.0);

        let mut import_statements = String::new();
        for (pkg_name, import_var) in &imports_to_add {
            let flat_id = package_to_flat_id.get(pkg_name).unwrap();
            import_statements.push_str(&format!(
                "import * as {} from \"./{}.js\";\n",
                import_var, flat_id
            ));
        }

        let final_content = format!("{}{}", import_statements, patched);
        std::fs::write(bundled_path, final_content)?;

        tracing::info!(
            "[deps] Patched {} CJS requires in {:?}",
            imports_to_add.len(),
            bundled_path.file_name()
        );

        Ok(())
    }

    fn bundle_react_refresh_runtime(&self) -> Result<PathBuf> {
        let output_path = self.cache_dir.join("react-refresh-runtime.js");

        if output_path.exists() {
            return Ok(output_path);
        }

        let entry_content = r#"
export * from 'react-refresh/runtime';
import * as _mod from 'react-refresh/runtime';
export default _mod.default ?? _mod;
"#;
        let entry_path = self.cache_dir.join("_entry_react_refresh.js");
        std::fs::write(&entry_path, entry_content)?;

        let fe_dir = self.project_root.join("fe");
        let result = Command::new("npx")
            .args([
                "rolldown",
                entry_path.to_str().unwrap(),
                "-o",
                output_path.to_str().unwrap(),
                "-f",
                "esm",
                "--platform",
                "browser",
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

    fn bundle_dependency(&self, package_name: &str) -> Result<(PathBuf, bool)> {
        let hash = compute_short_hash(package_name);
        let output_filename = format!("{}-{}.js", sanitize_package_name(package_name), hash);
        let output_path = self.cache_dir.join(&output_filename);

        if output_path.exists() {
            let is_cjs = self.detect_cjs_module(&output_path);
            return Ok((output_path, is_cjs));
        }

        let entry_content = format!(
            r#"export * from "{}";
import * as _mod from "{}";
export default _mod.default ?? _mod;"#,
            package_name, package_name
        );
        let entry_path = self.cache_dir.join(format!("_entry_{}.js", hash));
        std::fs::write(&entry_path, &entry_content)?;

        let fe_dir = self.project_root.join("fe");

        let args = vec![
            "rolldown".to_string(),
            entry_path.to_str().unwrap().to_string(),
            "-o".to_string(),
            output_path.to_str().unwrap().to_string(),
            "-f".to_string(),
            "esm".to_string(),
            "--platform".to_string(),
            "browser".to_string(),
        ];

        let result = Command::new("npx")
            .args(&args)
            .current_dir(&fe_dir)
            .output()
            .with_context(|| format!("Failed to run rolldown for {}", package_name))?;

        let _ = std::fs::remove_file(&entry_path);

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            anyhow::bail!("rolldown failed for {}: {}", package_name, stderr.trim());
        }

        let is_cjs = self.detect_cjs_module(&output_path);
        Ok((output_path, is_cjs))
    }

    fn detect_cjs_module(&self, bundled_path: &Path) -> bool {
        let content = match std::fs::read_to_string(bundled_path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let has_named_exports = content.contains("export {")
            || content.contains("export const ")
            || content.contains("export function ")
            || content.contains("export class ")
            || content.contains("export let ")
            || content.contains("export var ");

        let has_default_only = content.contains("export { default }") || content.contains("export default");

        if has_default_only && !has_named_exports {
            return true;
        }

        let allocator = Allocator::default();
        let source_type = SourceType::from_path(bundled_path).unwrap_or_default();
        let ret = Parser::new(&allocator, &content, source_type).parse();

        if ret.panicked {
            return false;
        }

        let mut has_real_named_export = false;
        let mut has_default_export = false;

        for stmt in &ret.program.body {
            match stmt {
                Statement::ExportDefaultDeclaration(_) => {
                    has_default_export = true;
                }
                Statement::ExportNamedDeclaration(decl) => {
                    if decl.declaration.is_some() {
                        has_real_named_export = true;
                    } else {
                        for spec in &decl.specifiers {
                            let exported_name = spec.exported.name();
                            if exported_name != "default" {
                                has_real_named_export = true;
                                break;
                            } else {
                                has_default_export = true;
                            }
                        }
                    }
                }
                Statement::ExportAllDeclaration(_) => {
                    has_real_named_export = true;
                }
                _ => {}
            }
        }

        has_default_export && !has_real_named_export
    }

    pub fn get_dep_url(&self, package_name: &str) -> Option<String> {
        self.dep_map.entries.get(package_name).cloned()
    }

    pub fn get_dep_map(&self) -> &DependencyMap {
        &self.dep_map
    }

    pub fn get_dep_map_mut(&mut self) -> &mut DependencyMap {
        &mut self.dep_map
    }

    pub fn register_missing_import(&mut self, import_path: &str) -> Result<Option<String>> {
        if self.dep_map.entries.contains_key(import_path) {
            return Ok(self.dep_map.entries.get(import_path).cloned());
        }

        let package_json_path = self.project_root.join("fe/package.json");
        let package_json: PackageJson = serde_json::from_str(
            &std::fs::read_to_string(&package_json_path)
                .with_context(|| format!("Failed to read {:?}", package_json_path))?,
        )?;

        let base_package_name = get_package_name(import_path);
        let is_installed = package_json
            .dependencies
            .as_ref()
            .map(|d| d.contains_key(&base_package_name))
            .unwrap_or(false);

        if !is_installed {
            return Ok(None);
        }

        tracing::info!("[deps] Dynamically bundling new dependency: {}", import_path);

        match self.bundle_dependency(import_path) {
            Ok((output_path, is_cjs)) => {
                let url_path = format!(
                    "/.forte/deps/{}",
                    output_path.file_name().unwrap().to_string_lossy()
                );
                self.dep_map
                    .entries
                    .insert(import_path.to_string(), url_path.clone());
                if is_cjs {
                    self.dep_map.cjs_modules.insert(import_path.to_string());
                }

                let map_path = self.cache_dir.join("dep-map.json");
                std::fs::write(&map_path, serde_json::to_string_pretty(&self.dep_map)?)?;

                let metadata = CacheMetadata {
                    lockfile_hash: self.compute_lockfile_hash(),
                    config_hash: self.compute_config_hash(),
                    entries: self.dep_map.entries.clone(),
                    cjs_modules: self.dep_map.cjs_modules.clone(),
                };
                self.save_metadata(&metadata)?;

                Ok(Some(url_path))
            }
            Err(e) => {
                tracing::warn!("Failed to bundle {}: {}", import_path, e);
                Err(e)
            }
        }
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

    fn metadata_path(&self) -> PathBuf {
        self.cache_dir.join("cache-metadata.json")
    }

    fn load_metadata(&self) -> Option<CacheMetadata> {
        let path = self.metadata_path();
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save_metadata(&self, metadata: &CacheMetadata) -> Result<()> {
        let path = self.metadata_path();
        std::fs::write(&path, serde_json::to_string_pretty(metadata)?)?;
        Ok(())
    }

    fn compute_lockfile_hash(&self) -> String {
        let lockfiles = [
            "fe/package-lock.json",
            "fe/yarn.lock",
            "fe/pnpm-lock.yaml",
            "fe/bun.lock",
        ];

        for lockfile in lockfiles {
            let path = self.project_root.join(lockfile);
            if path.exists() {
                if let Ok(content) = std::fs::read(&path) {
                    let mut hasher = Sha256::new();
                    hasher.update(&content);
                    return hex::encode(hasher.finalize());
                }
            }
        }

        String::new()
    }

    fn compute_config_hash(&self) -> String {
        let package_json_path = self.project_root.join("fe/package.json");
        if let Ok(content) = std::fs::read(&package_json_path) {
            let mut hasher = Sha256::new();
            hasher.update(&content);
            return hex::encode(hasher.finalize());
        }
        String::new()
    }

    fn should_rebuild(&self) -> bool {
        let metadata = match self.load_metadata() {
            Some(m) => m,
            None => return true,
        };

        let current_lockfile_hash = self.compute_lockfile_hash();
        let current_config_hash = self.compute_config_hash();

        if metadata.lockfile_hash != current_lockfile_hash {
            tracing::info!("[deps] Lockfile changed, rebuilding cache");
            return true;
        }

        if metadata.config_hash != current_config_hash {
            tracing::info!("[deps] Config changed, rebuilding cache");
            return true;
        }

        for (_, bundled_path) in &metadata.entries {
            let file_path = self.cache_dir.join(
                bundled_path
                    .strip_prefix("/.forte/deps/")
                    .unwrap_or(bundled_path),
            );
            if !file_path.exists() {
                tracing::info!("[deps] Cached file missing: {:?}, rebuilding", file_path);
                return true;
            }
        }

        false
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

fn get_package_name(specifier: &str) -> String {
    if specifier.starts_with('@') {
        let parts: Vec<&str> = specifier.splitn(3, '/').collect();
        if parts.len() >= 2 {
            return format!("{}/{}", parts[0], parts[1]);
        }
        return specifier.to_string();
    }

    specifier
        .split('/')
        .next()
        .unwrap_or(specifier)
        .to_string()
}

fn scan_imports_ast(code: &str, file_path: &Path) -> Vec<String> {
    let source_type = SourceType::from_path(file_path).unwrap_or_default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, code, source_type).parse();

    if ret.panicked {
        tracing::warn!("Failed to parse {:?}, skipping", file_path);
        return Vec::new();
    }

    let mut imports = Vec::new();
    collect_imports_from_program(&ret.program, &mut imports);
    imports
}

fn collect_imports_from_program(program: &Program, imports: &mut Vec<String>) {
    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(decl) => {
                imports.push(decl.source.value.to_string());
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(source) = &decl.source {
                    imports.push(source.value.to_string());
                }
            }
            Statement::ExportAllDeclaration(decl) => {
                imports.push(decl.source.value.to_string());
            }
            _ => {}
        }
        collect_imports_from_statement(stmt, imports);
    }
}

fn collect_imports_from_statement(stmt: &Statement, imports: &mut Vec<String>) {
    match stmt {
        Statement::BlockStatement(block) => {
            for s in &block.body {
                collect_imports_from_statement(s, imports);
            }
        }
        Statement::ExpressionStatement(expr_stmt) => {
            collect_imports_from_expression(&expr_stmt.expression, imports);
        }
        Statement::IfStatement(if_stmt) => {
            collect_imports_from_expression(&if_stmt.test, imports);
            collect_imports_from_statement(&if_stmt.consequent, imports);
            if let Some(alt) = &if_stmt.alternate {
                collect_imports_from_statement(alt, imports);
            }
        }
        Statement::WhileStatement(while_stmt) => {
            collect_imports_from_expression(&while_stmt.test, imports);
            collect_imports_from_statement(&while_stmt.body, imports);
        }
        Statement::DoWhileStatement(do_while) => {
            collect_imports_from_statement(&do_while.body, imports);
            collect_imports_from_expression(&do_while.test, imports);
        }
        Statement::ForStatement(for_stmt) => {
            if let Some(init) = &for_stmt.init {
                match init {
                    ForStatementInit::VariableDeclaration(decl) => {
                        for var_decl in &decl.declarations {
                            if let Some(init) = &var_decl.init {
                                collect_imports_from_expression(init, imports);
                            }
                        }
                    }
                    _ => {}
                }
            }
            if let Some(test) = &for_stmt.test {
                collect_imports_from_expression(test, imports);
            }
            if let Some(update) = &for_stmt.update {
                collect_imports_from_expression(update, imports);
            }
            collect_imports_from_statement(&for_stmt.body, imports);
        }
        Statement::ForInStatement(for_in) => {
            collect_imports_from_expression(&for_in.right, imports);
            collect_imports_from_statement(&for_in.body, imports);
        }
        Statement::ForOfStatement(for_of) => {
            collect_imports_from_expression(&for_of.right, imports);
            collect_imports_from_statement(&for_of.body, imports);
        }
        Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                collect_imports_from_expression(arg, imports);
            }
        }
        Statement::SwitchStatement(switch) => {
            collect_imports_from_expression(&switch.discriminant, imports);
            for case in &switch.cases {
                if let Some(test) = &case.test {
                    collect_imports_from_expression(test, imports);
                }
                for s in &case.consequent {
                    collect_imports_from_statement(s, imports);
                }
            }
        }
        Statement::TryStatement(try_stmt) => {
            for s in &try_stmt.block.body {
                collect_imports_from_statement(s, imports);
            }
            if let Some(handler) = &try_stmt.handler {
                for s in &handler.body.body {
                    collect_imports_from_statement(s, imports);
                }
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                for s in &finalizer.body {
                    collect_imports_from_statement(s, imports);
                }
            }
        }
        Statement::ThrowStatement(throw) => {
            collect_imports_from_expression(&throw.argument, imports);
        }
        Statement::VariableDeclaration(var_decl) => {
            for decl in &var_decl.declarations {
                if let Some(init) = &decl.init {
                    collect_imports_from_expression(init, imports);
                }
            }
        }
        Statement::FunctionDeclaration(func) => {
            if let Some(body) = &func.body {
                for s in &body.statements {
                    collect_imports_from_statement(s, imports);
                }
            }
        }
        Statement::ClassDeclaration(class) => {
            collect_imports_from_class_elements(&class.body.body, imports);
        }
        Statement::ExportDefaultDeclaration(export_default) => {
            match &export_default.declaration {
                oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                    if let Some(body) = &func.body {
                        for s in &body.statements {
                            collect_imports_from_statement(s, imports);
                        }
                    }
                }
                oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                    collect_imports_from_class_elements(&class.body.body, imports);
                }
                _ => {}
            }
        }
        Statement::ExportNamedDeclaration(export_named) => {
            if let Some(decl) = &export_named.declaration {
                match decl {
                    Declaration::FunctionDeclaration(func) => {
                        if let Some(body) = &func.body {
                            for s in &body.statements {
                                collect_imports_from_statement(s, imports);
                            }
                        }
                    }
                    Declaration::ClassDeclaration(class) => {
                        collect_imports_from_class_elements(&class.body.body, imports);
                    }
                    Declaration::VariableDeclaration(var_decl) => {
                        for decl in &var_decl.declarations {
                            if let Some(init) = &decl.init {
                                collect_imports_from_expression(init, imports);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn collect_imports_from_expression(expr: &Expression, imports: &mut Vec<String>) {
    match expr {
        Expression::ImportExpression(import_expr) => {
            if let Expression::StringLiteral(lit) = &import_expr.source {
                imports.push(lit.value.to_string());
            }
        }
        Expression::CallExpression(call) => {
            collect_imports_from_call_expression(call, imports);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                if let Some(stmt) = arrow.body.statements.first() {
                    if let Statement::ExpressionStatement(expr_stmt) = stmt {
                        collect_imports_from_expression(&expr_stmt.expression, imports);
                    }
                }
            } else {
                for s in &arrow.body.statements {
                    collect_imports_from_statement(s, imports);
                }
            }
        }
        Expression::FunctionExpression(func) => {
            if let Some(body) = &func.body {
                for s in &body.statements {
                    collect_imports_from_statement(s, imports);
                }
            }
        }
        Expression::ConditionalExpression(cond) => {
            collect_imports_from_expression(&cond.test, imports);
            collect_imports_from_expression(&cond.consequent, imports);
            collect_imports_from_expression(&cond.alternate, imports);
        }
        Expression::SequenceExpression(seq) => {
            for e in &seq.expressions {
                collect_imports_from_expression(e, imports);
            }
        }
        Expression::LogicalExpression(logical) => {
            collect_imports_from_expression(&logical.left, imports);
            collect_imports_from_expression(&logical.right, imports);
        }
        Expression::BinaryExpression(binary) => {
            collect_imports_from_expression(&binary.left, imports);
            collect_imports_from_expression(&binary.right, imports);
        }
        Expression::UnaryExpression(unary) => {
            collect_imports_from_expression(&unary.argument, imports);
        }
        Expression::AssignmentExpression(assign) => {
            collect_imports_from_expression(&assign.right, imports);
        }
        Expression::ArrayExpression(arr) => {
            for element in &arr.elements {
                match element {
                    ArrayExpressionElement::SpreadElement(spread) => {
                        collect_imports_from_expression(&spread.argument, imports);
                    }
                    _ => {
                        if let Some(e) = element.as_expression() {
                            collect_imports_from_expression(e, imports);
                        }
                    }
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in &obj.properties {
                match prop {
                    ObjectPropertyKind::ObjectProperty(p) => {
                        collect_imports_from_expression(&p.value, imports);
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        collect_imports_from_expression(&spread.argument, imports);
                    }
                }
            }
        }
        Expression::ComputedMemberExpression(computed) => {
            collect_imports_from_expression(&computed.object, imports);
            collect_imports_from_expression(&computed.expression, imports);
        }
        Expression::StaticMemberExpression(static_member) => {
            collect_imports_from_expression(&static_member.object, imports);
        }
        Expression::PrivateFieldExpression(private) => {
            collect_imports_from_expression(&private.object, imports);
        }
        Expression::AwaitExpression(await_expr) => {
            collect_imports_from_expression(&await_expr.argument, imports);
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(arg) = &yield_expr.argument {
                collect_imports_from_expression(arg, imports);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            collect_imports_from_expression(&paren.expression, imports);
        }
        Expression::NewExpression(new_expr) => {
            collect_imports_from_expression(&new_expr.callee, imports);
            for arg in &new_expr.arguments {
                if let Some(e) = arg.as_expression() {
                    collect_imports_from_expression(e, imports);
                }
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            collect_imports_from_expression(&tagged.tag, imports);
            for expr in &tagged.quasi.expressions {
                collect_imports_from_expression(expr, imports);
            }
        }
        Expression::TemplateLiteral(template) => {
            for expr in &template.expressions {
                collect_imports_from_expression(expr, imports);
            }
        }
        Expression::ChainExpression(chain) => {
            match &chain.expression {
                ChainElement::CallExpression(call) => {
                    collect_imports_from_call_expression(call, imports);
                }
                ChainElement::TSNonNullExpression(ts_non_null) => {
                    collect_imports_from_expression(&ts_non_null.expression, imports);
                }
                ChainElement::ComputedMemberExpression(computed) => {
                    collect_imports_from_expression(&computed.object, imports);
                    collect_imports_from_expression(&computed.expression, imports);
                }
                ChainElement::StaticMemberExpression(static_member) => {
                    collect_imports_from_expression(&static_member.object, imports);
                }
                ChainElement::PrivateFieldExpression(private) => {
                    collect_imports_from_expression(&private.object, imports);
                }
            }
        }
        Expression::ClassExpression(class) => {
            collect_imports_from_class_elements(&class.body.body, imports);
        }
        Expression::TSAsExpression(ts_as) => {
            collect_imports_from_expression(&ts_as.expression, imports);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            collect_imports_from_expression(&ts_satisfies.expression, imports);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            collect_imports_from_expression(&ts_non_null.expression, imports);
        }
        Expression::TSTypeAssertion(ts_type) => {
            collect_imports_from_expression(&ts_type.expression, imports);
        }
        _ => {}
    }
}

fn collect_imports_from_call_expression(call: &CallExpression, imports: &mut Vec<String>) {
    if let Expression::Identifier(ident) = &call.callee {
        if ident.name == "require" {
            if let Some(arg) = call.arguments.first() {
                if let Some(expr) = arg.as_expression() {
                    if let Expression::StringLiteral(lit) = expr {
                        imports.push(lit.value.to_string());
                    }
                }
            }
        }
    }

    collect_imports_from_expression(&call.callee, imports);
    for arg in &call.arguments {
        if let Some(e) = arg.as_expression() {
            collect_imports_from_expression(e, imports);
        }
    }
}

fn collect_imports_from_class_elements(elements: &oxc_allocator::Vec<ClassElement>, imports: &mut Vec<String>) {
    for element in elements {
        match element {
            ClassElement::MethodDefinition(method) => {
                if let Some(body) = &method.value.body {
                    for s in &body.statements {
                        collect_imports_from_statement(s, imports);
                    }
                }
            }
            ClassElement::PropertyDefinition(prop) => {
                if let Some(value) = &prop.value {
                    collect_imports_from_expression(value, imports);
                }
            }
            ClassElement::StaticBlock(block) => {
                for s in &block.body {
                    collect_imports_from_statement(s, imports);
                }
            }
            _ => {}
        }
    }
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

    #[test]
    fn test_scan_imports_ast_basic() {
        let code = r#"
            import React from 'react';
            import { useState } from 'react';
            import * as lodash from 'lodash';
        "#;
        let path = std::path::Path::new("test.tsx");
        let imports = scan_imports_ast(code, path);
        assert!(imports.contains(&"react".to_string()));
        assert!(imports.contains(&"lodash".to_string()));
    }

    #[test]
    fn test_scan_imports_ast_export_from() {
        let code = r#"
            export { foo } from 'some-module';
            export * from 'another-module';
        "#;
        let path = std::path::Path::new("test.ts");
        let imports = scan_imports_ast(code, path);
        assert!(imports.contains(&"some-module".to_string()));
        assert!(imports.contains(&"another-module".to_string()));
    }

    #[test]
    fn test_scan_imports_ast_dynamic_import() {
        let code = r#"
            const module = await import('dynamic-module');
            import('another-dynamic').then(m => m.default);
        "#;
        let path = std::path::Path::new("test.ts");
        let imports = scan_imports_ast(code, path);
        assert!(imports.contains(&"dynamic-module".to_string()));
        assert!(imports.contains(&"another-dynamic".to_string()));
    }

    #[test]
    fn test_scan_imports_ast_require() {
        let code = r#"
            const fs = require('fs');
            const path = require('path');
        "#;
        let path = std::path::Path::new("test.js");
        let imports = scan_imports_ast(code, path);
        assert!(imports.contains(&"fs".to_string()));
        assert!(imports.contains(&"path".to_string()));
    }

    #[test]
    fn test_scan_imports_ast_ignores_comments() {
        let code = r#"
            // import fake from 'fake-module';
            /* import another from 'another-fake'; */
            import real from 'real-module';
        "#;
        let path = std::path::Path::new("test.ts");
        let imports = scan_imports_ast(code, path);
        assert!(imports.contains(&"real-module".to_string()));
        assert!(!imports.contains(&"fake-module".to_string()));
        assert!(!imports.contains(&"another-fake".to_string()));
    }

    #[test]
    fn test_scan_imports_ast_ignores_string_content() {
        let code = r#"
            const str = "import fake from 'fake-module'";
            const template = `import another from 'template-fake'`;
            import real from 'real-module';
        "#;
        let path = std::path::Path::new("test.ts");
        let imports = scan_imports_ast(code, path);
        assert!(imports.contains(&"real-module".to_string()));
        assert!(!imports.contains(&"fake-module".to_string()));
        assert!(!imports.contains(&"template-fake".to_string()));
    }

    #[test]
    fn test_scan_imports_ast_nested_function() {
        let code = r#"
            function outer() {
                const module = await import('nested-dynamic');
                return module;
            }
        "#;
        let path = std::path::Path::new("test.ts");
        let imports = scan_imports_ast(code, path);
        assert!(imports.contains(&"nested-dynamic".to_string()));
    }

    #[test]
    fn test_scan_imports_ast_arrow_function() {
        let code = r#"
            const loadModule = async () => {
                const m = await import('arrow-dynamic');
                return m;
            };
        "#;
        let path = std::path::Path::new("test.ts");
        let imports = scan_imports_ast(code, path);
        assert!(imports.contains(&"arrow-dynamic".to_string()));
    }

    #[test]
    fn test_compute_lockfile_hash() {
        let temp_dir = std::env::temp_dir().join("forte_test_lockfile_hash");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("fe/.forte/deps")).unwrap();

        let lockfile_path = temp_dir.join("fe/package-lock.json");
        std::fs::write(&lockfile_path, r#"{"lockfileVersion": 1}"#).unwrap();

        let prebundler = DependencyPrebundler::new(&temp_dir);
        let hash1 = prebundler.compute_lockfile_hash();

        assert!(!hash1.is_empty());
        assert_eq!(hash1.len(), 64);

        std::fs::write(&lockfile_path, r#"{"lockfileVersion": 2}"#).unwrap();
        let hash2 = prebundler.compute_lockfile_hash();

        assert_ne!(hash1, hash2);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_compute_config_hash() {
        let temp_dir = std::env::temp_dir().join("forte_test_config_hash");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("fe/.forte/deps")).unwrap();

        let package_json_path = temp_dir.join("fe/package.json");
        std::fs::write(&package_json_path, r#"{"name": "test", "version": "1.0.0"}"#).unwrap();

        let prebundler = DependencyPrebundler::new(&temp_dir);
        let hash1 = prebundler.compute_config_hash();

        assert!(!hash1.is_empty());
        assert_eq!(hash1.len(), 64);

        std::fs::write(&package_json_path, r#"{"name": "test", "version": "2.0.0"}"#).unwrap();
        let hash2 = prebundler.compute_config_hash();

        assert_ne!(hash1, hash2);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_should_rebuild_no_metadata() {
        let temp_dir = std::env::temp_dir().join("forte_test_no_metadata");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("fe/.forte/deps")).unwrap();

        let prebundler = DependencyPrebundler::new(&temp_dir);
        assert!(prebundler.should_rebuild());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_should_rebuild_lockfile_changed() {
        let temp_dir = std::env::temp_dir().join("forte_test_lockfile_changed");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("fe/.forte/deps")).unwrap();

        let lockfile_path = temp_dir.join("fe/package-lock.json");
        let package_json_path = temp_dir.join("fe/package.json");

        std::fs::write(&lockfile_path, r#"{"lockfileVersion": 1}"#).unwrap();
        std::fs::write(&package_json_path, r#"{"name": "test"}"#).unwrap();

        let prebundler = DependencyPrebundler::new(&temp_dir);

        let metadata = CacheMetadata {
            lockfile_hash: prebundler.compute_lockfile_hash(),
            config_hash: prebundler.compute_config_hash(),
            entries: HashMap::new(),
            cjs_modules: HashSet::new(),
        };
        prebundler.save_metadata(&metadata).unwrap();

        assert!(!prebundler.should_rebuild());

        std::fs::write(&lockfile_path, r#"{"lockfileVersion": 2}"#).unwrap();
        assert!(prebundler.should_rebuild());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_should_rebuild_cache_valid() {
        let temp_dir = std::env::temp_dir().join("forte_test_cache_valid");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("fe/.forte/deps")).unwrap();

        let lockfile_path = temp_dir.join("fe/package-lock.json");
        let package_json_path = temp_dir.join("fe/package.json");

        std::fs::write(&lockfile_path, r#"{"lockfileVersion": 1}"#).unwrap();
        std::fs::write(&package_json_path, r#"{"name": "test"}"#).unwrap();

        let prebundler = DependencyPrebundler::new(&temp_dir);

        let metadata = CacheMetadata {
            lockfile_hash: prebundler.compute_lockfile_hash(),
            config_hash: prebundler.compute_config_hash(),
            entries: HashMap::new(),
            cjs_modules: HashSet::new(),
        };
        prebundler.save_metadata(&metadata).unwrap();

        assert!(!prebundler.should_rebuild());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cache_metadata_save_load() {
        let temp_dir = std::env::temp_dir().join("forte_test_metadata_save_load");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("fe/.forte/deps")).unwrap();

        let prebundler = DependencyPrebundler::new(&temp_dir);

        let mut entries = HashMap::new();
        entries.insert("react".to_string(), "/.forte/deps/react-abc123.js".to_string());
        entries.insert("lodash".to_string(), "/.forte/deps/lodash-def456.js".to_string());

        let original_metadata = CacheMetadata {
            lockfile_hash: "abc123lockfile".to_string(),
            config_hash: "def456config".to_string(),
            entries,
            cjs_modules: HashSet::new(),
        };

        prebundler.save_metadata(&original_metadata).unwrap();

        let loaded_metadata = prebundler.load_metadata().expect("Failed to load metadata");

        assert_eq!(loaded_metadata.lockfile_hash, original_metadata.lockfile_hash);
        assert_eq!(loaded_metadata.config_hash, original_metadata.config_hash);
        assert_eq!(loaded_metadata.entries.len(), original_metadata.entries.len());
        assert_eq!(
            loaded_metadata.entries.get("react"),
            original_metadata.entries.get("react")
        );
        assert_eq!(
            loaded_metadata.entries.get("lodash"),
            original_metadata.entries.get("lodash")
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_register_missing_import_not_installed() {
        let temp_dir = std::env::temp_dir().join("forte_test_not_installed");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("fe/.forte/deps")).unwrap();

        let package_json_path = temp_dir.join("fe/package.json");
        std::fs::write(
            &package_json_path,
            r#"{"name": "test", "dependencies": {"react": "^18.0.0"}}"#,
        )
        .unwrap();

        let mut prebundler = DependencyPrebundler::new(&temp_dir);

        let result = prebundler.register_missing_import("not-installed-package");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_register_missing_import_already_bundled() {
        let temp_dir = std::env::temp_dir().join("forte_test_already_bundled");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("fe/.forte/deps")).unwrap();

        let package_json_path = temp_dir.join("fe/package.json");
        std::fs::write(
            &package_json_path,
            r#"{"name": "test", "dependencies": {"react": "^18.0.0"}}"#,
        )
        .unwrap();

        let mut prebundler = DependencyPrebundler::new(&temp_dir);

        let existing_url = "/.forte/deps/react-abc123.js".to_string();
        prebundler
            .dep_map
            .entries
            .insert("react".to_string(), existing_url.clone());

        let result = prebundler.register_missing_import("react");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(existing_url));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
