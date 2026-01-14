use crate::deps::DependencyMap;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct ImportRewriter {
    dep_map: DependencyMap,
    aliases: HashMap<String, String>,
    src_base: String,
}

#[derive(Debug)]
struct CjsImportTransform {
    start: usize,
    end: usize,
    specifier: String,
    resolved_url: String,
    named_imports: Vec<(String, String)>,
    default_import: Option<String>,
    namespace_import: Option<String>,
}

fn strip_json_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escape_next = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if !in_string && c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        if ch == '\n' {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                } else if next == '*' {
                    chars.next();
                    while let Some(ch) = chars.next() {
                        if ch == '*' {
                            if let Some(&'/') = chars.peek() {
                                chars.next();
                                break;
                            }
                        }
                    }
                    continue;
                }
            }
        }

        result.push(c);
    }

    result
}

pub fn load_aliases_from_tsconfig(project_root: &Path) -> HashMap<String, String> {
    let tsconfig_path = project_root.join("fe/tsconfig.json");

    let content = match std::fs::read_to_string(&tsconfig_path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    let stripped = strip_json_comments(&content);

    let json: serde_json::Value = match serde_json::from_str(&stripped) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };

    let mut aliases = HashMap::new();

    let compiler_options = match json.get("compilerOptions") {
        Some(co) => co,
        None => return aliases,
    };

    let base_url = compiler_options
        .get("baseUrl")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    let paths = match compiler_options.get("paths") {
        Some(p) => p,
        None => return aliases,
    };

    let paths_obj = match paths.as_object() {
        Some(o) => o,
        None => return aliases,
    };

    for (alias_pattern, targets) in paths_obj {
        let target_array = match targets.as_array() {
            Some(a) => a,
            None => continue,
        };

        if target_array.is_empty() {
            continue;
        }

        let first_target = match target_array[0].as_str() {
            Some(s) => s,
            None => continue,
        };

        let alias_key = alias_pattern.trim_end_matches("/*");

        let mut target_path = first_target.trim_end_matches("/*").to_string();

        if target_path.starts_with("./") {
            target_path = target_path[2..].to_string();
        }

        if base_url == "." {
            target_path = format!("/{}", target_path);
        } else {
            let base = base_url.trim_start_matches("./").trim_end_matches('/');
            target_path = format!("/{}/{}", base, target_path);
        }

        aliases.insert(alias_key.to_string(), target_path);
    }

    aliases
}

impl ImportRewriter {
    pub fn new(dep_map: DependencyMap, project_root: Option<&Path>) -> Self {
        let mut aliases = if let Some(root) = project_root {
            load_aliases_from_tsconfig(root)
        } else {
            HashMap::new()
        };

        if aliases.is_empty() {
            aliases.insert("@".to_string(), "/src".to_string());
        }

        Self {
            dep_map,
            aliases,
            src_base: "/src".to_string(),
        }
    }

    pub fn rewrite(&self, code: &str, current_file: &Path, timestamp: u64) -> String {
        let cjs_transforms = self.analyze_cjs_imports(code, current_file, timestamp);

        if !cjs_transforms.is_empty() {
            let transformed = self.apply_cjs_transforms(code, current_file, &cjs_transforms, timestamp);
            return transformed;
        }

        self.rewrite_simple(code, current_file, timestamp)
    }

    fn rewrite_simple(&self, code: &str, current_file: &Path, timestamp: u64) -> String {
        let mut result = code.to_string();

        let patterns = [
            (r#"from\s*["']([^"']+)["']"#, "from"),
            (r#"import\s*\(\s*["']([^"']+)["']\s*\)"#, "dynamic"),
            (r#"import\s+["']([^"']+)["']"#, "sideeffect"),
        ];

        for (pattern, kind) in patterns {
            let re = regex::Regex::new(pattern).unwrap();
            let mut offset: i64 = 0;

            let current_code = result.clone();
            let matches: Vec<_> = re.captures_iter(&current_code).collect();
            for cap in matches {
                let full_match = cap.get(0).unwrap();
                let specifier = cap.get(1).unwrap().as_str();

                if let Some(new_path) = self.resolve_specifier(specifier, current_file, timestamp) {
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

    fn analyze_cjs_imports(
        &self,
        code: &str,
        current_file: &Path,
        timestamp: u64,
    ) -> Vec<CjsImportTransform> {
        let mut transforms = Vec::new();

        let allocator = Allocator::default();
        let source_type = SourceType::from_path(current_file).unwrap_or_default();
        let ret = Parser::new(&allocator, code, source_type).parse();

        if ret.panicked {
            return transforms;
        }

        for stmt in &ret.program.body {
            if let Statement::ImportDeclaration(decl) = stmt {
                let specifier = decl.source.value.to_string();

                if !self.is_cjs_module(&specifier) {
                    continue;
                }

                let resolved_url = match self.resolve_specifier(&specifier, current_file, timestamp) {
                    Some(url) => url,
                    None => continue,
                };

                let mut named_imports = Vec::new();
                let mut default_import = None;
                let mut namespace_import = None;

                if let Some(specifiers) = &decl.specifiers {
                    for spec in specifiers {
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(named) => {
                                let imported = named.imported.name().to_string();
                                let local = named.local.name.to_string();
                                named_imports.push((imported, local));
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(def) => {
                                default_import = Some(def.local.name.to_string());
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => {
                                namespace_import = Some(ns.local.name.to_string());
                            }
                        }
                    }
                }

                if !named_imports.is_empty() {
                    transforms.push(CjsImportTransform {
                        start: decl.span.start as usize,
                        end: decl.span.end as usize,
                        specifier,
                        resolved_url,
                        named_imports,
                        default_import,
                        namespace_import,
                    });
                }
            }
        }

        transforms
    }

    fn apply_cjs_transforms(
        &self,
        code: &str,
        current_file: &Path,
        transforms: &[CjsImportTransform],
        timestamp: u64,
    ) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        let mut sorted_transforms: Vec<_> = transforms.iter().collect();
        sorted_transforms.sort_by_key(|t| t.start);

        let mut cjs_module_counter = 0;
        let mut processed_specifiers: HashSet<String> = HashSet::new();

        for transform in &sorted_transforms {
            result.push_str(&code[last_end..transform.start]);

            let cjs_var = if processed_specifiers.contains(&transform.specifier) {
                format!("__cjs_{}__", transform.specifier.replace(['/', '-', '@', '.'], "_"))
            } else {
                cjs_module_counter += 1;
                let var = format!("__cjs_{}__", transform.specifier.replace(['/', '-', '@', '.'], "_"));
                processed_specifiers.insert(transform.specifier.clone());
                var
            };

            let mut import_parts = Vec::new();

            if let Some(ref default_name) = transform.default_import {
                import_parts.push(default_name.clone());
            }

            if !processed_specifiers.contains(&transform.specifier) || cjs_module_counter == 1 {
                let import_stmt = format!(
                    "import {} from \"{}\"",
                    cjs_var, transform.resolved_url
                );
                result.push_str(&import_stmt);

                if let Some(ref default_name) = transform.default_import {
                    result.push_str(&format!(";\nconst {} = {}", default_name, cjs_var));
                }

                if let Some(ref ns_name) = transform.namespace_import {
                    result.push_str(&format!(";\nconst {} = {}", ns_name, cjs_var));
                }

                if !transform.named_imports.is_empty() {
                    let destructure: Vec<String> = transform
                        .named_imports
                        .iter()
                        .map(|(imported, local)| {
                            if imported == local {
                                format!("{}", local)
                            } else {
                                format!("{}: {}", imported, local)
                            }
                        })
                        .collect();
                    result.push_str(&format!(
                        ";\nconst {{ {} }} = {}",
                        destructure.join(", "),
                        cjs_var
                    ));
                }
            } else {
                if let Some(ref default_name) = transform.default_import {
                    result.push_str(&format!("const {} = {}", default_name, cjs_var));
                }

                if let Some(ref ns_name) = transform.namespace_import {
                    if transform.default_import.is_some() {
                        result.push_str(";\n");
                    }
                    result.push_str(&format!("const {} = {}", ns_name, cjs_var));
                }

                if !transform.named_imports.is_empty() {
                    if transform.default_import.is_some() || transform.namespace_import.is_some() {
                        result.push_str(";\n");
                    }
                    let destructure: Vec<String> = transform
                        .named_imports
                        .iter()
                        .map(|(imported, local)| {
                            if imported == local {
                                format!("{}", local)
                            } else {
                                format!("{}: {}", imported, local)
                            }
                        })
                        .collect();
                    result.push_str(&format!("const {{ {} }} = {}", destructure.join(", "), cjs_var));
                }
            }

            last_end = transform.end;
        }

        result.push_str(&code[last_end..]);

        self.rewrite_simple(&result, current_file, timestamp)
    }

    fn is_cjs_module(&self, specifier: &str) -> bool {
        self.dep_map.cjs_modules.contains(specifier)
    }

    fn resolve_specifier(
        &self,
        specifier: &str,
        current_file: &Path,
        timestamp: u64,
    ) -> Option<String> {
        // Check if it's a bare import (from node_modules)
        if !specifier.starts_with('.') && !specifier.starts_with('/') && !specifier.starts_with('@')
        {
            // Try exact match first (for subpath imports like react-dom/client)
            if let Some(url) = self.dep_map.entries.get(specifier) {
                return Some(url.clone());
            }
            // Fallback to base package name
            let package_name = get_package_name(specifier);
            if let Some(url) = self.dep_map.entries.get(&package_name) {
                return Some(url.clone());
            }
            return None;
        }

        // Check if it's a scoped package like @scope/package
        if specifier.starts_with('@') {
            // Check if it's an alias like @/components
            for (alias, target) in &self.aliases {
                let alias_prefix = format!("{}/", alias);
                if specifier.starts_with(&alias_prefix) {
                    let rest = &specifier[alias_prefix.len()..];
                    let resolved = format!("{}/{}?t={}", target, rest, timestamp);
                    return Some(add_extension_if_needed(&resolved));
                }
                if specifier == alias {
                    return Some(format!("{}?t={}", target, timestamp));
                }
            }

            // Try exact match first (for subpath imports like @scope/package/subpath)
            if let Some(url) = self.dep_map.entries.get(specifier) {
                return Some(url.clone());
            }
            // Fallback to base package name
            let package_name = get_package_name(specifier);
            if let Some(url) = self.dep_map.entries.get(&package_name) {
                return Some(url.clone());
            }
            return None;
        }

        // Relative import
        if specifier.starts_with('.') {
            let current_dir = current_file.parent().unwrap_or(Path::new(""));
            let resolved = current_dir.join(specifier);

            // Normalize the path
            let normalized = normalize_path(&resolved);
            let url_path = format!("{}/{}?t={}", self.src_base, normalized, timestamp);
            return Some(add_extension_if_needed(&url_path));
        }

        None
    }
}

fn get_package_name(specifier: &str) -> String {
    if specifier.starts_with('@') {
        // Scoped package: @scope/package/subpath -> @scope/package
        let parts: Vec<&str> = specifier.splitn(3, '/').collect();
        if parts.len() >= 2 {
            return format!("{}/{}", parts[0], parts[1]);
        }
    }
    // Regular package: package/subpath -> package
    specifier.split('/').next().unwrap_or(specifier).to_string()
}

fn normalize_path(path: &Path) -> String {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(s) => {
                components.push(s.to_string_lossy().to_string());
            }
            _ => {}
        }
    }
    components.join("/")
}

fn add_extension_if_needed(path: &str) -> String {
    let (base, query) = if let Some(idx) = path.find('?') {
        (&path[..idx], &path[idx..])
    } else {
        (path, "")
    };

    if base.ends_with(".css") {
        if query.is_empty() {
            return format!("{}?import", base);
        } else {
            return format!("{}{}&import", base, query);
        }
    }

    path.to_string()
}

pub fn rewrite_imports(
    code: &str,
    dep_entries: &std::collections::HashMap<String, String>,
) -> String {
    let mut dep_map = DependencyMap::default();
    dep_map.entries = dep_entries.clone();
    let rewriter = ImportRewriter::new(dep_map, None);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    rewriter.rewrite(code, std::path::Path::new(""), timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_package_name() {
        assert_eq!(get_package_name("react"), "react");
        assert_eq!(get_package_name("react-dom/client"), "react-dom");
        assert_eq!(get_package_name("@scope/package"), "@scope/package");
        assert_eq!(get_package_name("@scope/package/sub"), "@scope/package");
    }

    #[test]
    fn test_add_extension() {
        assert_eq!(add_extension_if_needed("/src/Button"), "/src/Button");
        assert_eq!(add_extension_if_needed("/src/Button?t=123"), "/src/Button?t=123");
        assert_eq!(add_extension_if_needed("/src/Button.tsx"), "/src/Button.tsx");
        assert_eq!(add_extension_if_needed("/src/button.css"), "/src/button.css?import");
        assert_eq!(
            add_extension_if_needed("/src/button.css?t=123"),
            "/src/button.css?t=123&import"
        );
    }

    #[test]
    fn test_rewrite_imports() {
        let mut dep_map = DependencyMap::default();
        dep_map
            .entries
            .insert("react".to_string(), "/.forte/deps/react-abc.js".to_string());

        let rewriter = ImportRewriter::new(dep_map, None);
        let code = r#"import React from "react";
import { useState } from "react";
import { Button } from "./Button";
import { utils } from "@/lib/utils";"#;

        let result = rewriter.rewrite(code, Path::new("components/App.tsx"), 12345);

        assert!(result.contains("/.forte/deps/react-abc.js"));
        assert!(result.contains("/src/components/Button?t=12345"));
    }

    #[test]
    fn test_subpath_imports() {
        let mut dep_map = DependencyMap::default();
        dep_map
            .entries
            .insert("react".to_string(), "/.forte/deps/react-abc.js".to_string());
        dep_map.entries.insert(
            "react-dom/client".to_string(),
            "/.forte/deps/react-dom__client-def.js".to_string(),
        );
        dep_map.entries.insert(
            "jotai/vanilla".to_string(),
            "/.forte/deps/jotai__vanilla-ghi.js".to_string(),
        );

        let rewriter = ImportRewriter::new(dep_map, None);
        let code = r#"import { createRoot } from "react-dom/client";
import { atom } from "jotai/vanilla";"#;

        let result = rewriter.rewrite(code, Path::new("main.tsx"), 12345);

        assert!(result.contains("/.forte/deps/react-dom__client-def.js"));
        assert!(result.contains("/.forte/deps/jotai__vanilla-ghi.js"));
    }

    #[test]
    fn test_scoped_subpath_imports() {
        let mut dep_map = DependencyMap::default();
        dep_map.entries.insert(
            "@tanstack/react-query".to_string(),
            "/.forte/deps/tanstack__react-query-abc.js".to_string(),
        );
        dep_map.entries.insert(
            "@tanstack/react-query/devtools".to_string(),
            "/.forte/deps/tanstack__react-query__devtools-def.js".to_string(),
        );

        let rewriter = ImportRewriter::new(dep_map, None);
        let code = r#"import { useQuery } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query/devtools";"#;

        let result = rewriter.rewrite(code, Path::new("App.tsx"), 12345);

        assert!(result.contains("/.forte/deps/tanstack__react-query-abc.js"));
        assert!(result.contains("/.forte/deps/tanstack__react-query__devtools-def.js"));
    }

    #[test]
    fn test_strip_json_comments() {
        let json_with_comments = r#"{
  // single line comment
  "compilerOptions": {
    "target": "ES2022", /* inline comment */
    "baseUrl": ".",
    /*
     * multi-line comment
     */
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}"#;

        let stripped = strip_json_comments(json_with_comments);
        let parsed: serde_json::Value = serde_json::from_str(&stripped).unwrap();

        assert_eq!(
            parsed["compilerOptions"]["target"].as_str().unwrap(),
            "ES2022"
        );
        assert!(parsed["compilerOptions"]["paths"]["@/*"].is_array());
    }

    #[test]
    fn test_load_aliases_from_tsconfig() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir().join(format!("forte_test_{}", std::process::id()));
        let fe_dir = temp_dir.join("fe");
        std::fs::create_dir_all(&fe_dir).unwrap();

        let tsconfig = r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"],
      "@components/*": ["./src/components/*"],
      "@utils/*": ["./src/lib/utils/*"]
    }
  }
}"#;
        let mut file = std::fs::File::create(fe_dir.join("tsconfig.json")).unwrap();
        file.write_all(tsconfig.as_bytes()).unwrap();

        let aliases = load_aliases_from_tsconfig(&temp_dir);

        assert_eq!(aliases.get("@"), Some(&"/src".to_string()));
        assert_eq!(aliases.get("@components"), Some(&"/src/components".to_string()));
        assert_eq!(aliases.get("@utils"), Some(&"/src/lib/utils".to_string()));

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_custom_alias_rewrite() {
        let mut dep_map = DependencyMap::default();
        dep_map
            .entries
            .insert("react".to_string(), "/.forte/deps/react-abc.js".to_string());

        let mut aliases = HashMap::new();
        aliases.insert("@".to_string(), "/src".to_string());
        aliases.insert("@components".to_string(), "/src/components".to_string());
        aliases.insert("@utils".to_string(), "/src/lib/utils".to_string());

        let rewriter = ImportRewriter {
            dep_map,
            aliases,
            src_base: "/src".to_string(),
        };

        let code = r#"import { Button } from "@components/Button";
import { format } from "@utils/format";
import { api } from "@/api";"#;

        let result = rewriter.rewrite(code, Path::new("App.tsx"), 12345);

        assert!(result.contains("/src/components/Button?t=12345"));
        assert!(result.contains("/src/lib/utils/format?t=12345"));
        assert!(result.contains("/src/api?t=12345"));
    }

    #[test]
    fn test_cjs_named_import_transform() {
        let mut dep_map = DependencyMap::default();
        dep_map.entries.insert(
            "react-dom/client".to_string(),
            "/.forte/deps/react-dom__client-abc.js".to_string(),
        );
        dep_map.cjs_modules.insert("react-dom/client".to_string());

        let rewriter = ImportRewriter::new(dep_map, None);
        let code = r#"import { hydrateRoot, createRoot } from "react-dom/client";"#;

        let result = rewriter.rewrite(code, Path::new("main.tsx"), 12345);

        assert!(result.contains("import __cjs_react_dom_client__"));
        assert!(result.contains("/.forte/deps/react-dom__client-abc.js"));
        assert!(result.contains("const { hydrateRoot, createRoot } = __cjs_react_dom_client__"));
    }

    #[test]
    fn test_cjs_mixed_import_transform() {
        let mut dep_map = DependencyMap::default();
        dep_map.entries.insert(
            "some-cjs-lib".to_string(),
            "/.forte/deps/some-cjs-lib-xyz.js".to_string(),
        );
        dep_map.cjs_modules.insert("some-cjs-lib".to_string());

        let rewriter = ImportRewriter::new(dep_map, None);
        let code = r#"import DefaultExport, { namedOne, namedTwo as alias } from "some-cjs-lib";"#;

        let result = rewriter.rewrite(code, Path::new("app.tsx"), 12345);

        assert!(result.contains("import __cjs_some_cjs_lib__"));
        assert!(result.contains("const DefaultExport = __cjs_some_cjs_lib__"));
        assert!(result.contains("const { namedOne, namedTwo: alias } = __cjs_some_cjs_lib__"));
    }

    #[test]
    fn test_non_cjs_not_transformed() {
        let mut dep_map = DependencyMap::default();
        dep_map.entries.insert(
            "react".to_string(),
            "/.forte/deps/react-abc.js".to_string(),
        );

        let rewriter = ImportRewriter::new(dep_map, None);
        let code = r#"import { useState, useEffect } from "react";"#;

        let result = rewriter.rewrite(code, Path::new("app.tsx"), 12345);

        assert!(result.contains(r#"from "/.forte/deps/react-abc.js""#));
        assert!(!result.contains("__cjs_"));
    }
}
