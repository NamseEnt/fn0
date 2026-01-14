use crate::deps::DependencyMap;
use std::collections::HashMap;
use std::path::Path;

pub struct ImportRewriter {
    dep_map: DependencyMap,
    aliases: HashMap<String, String>,
    src_base: String,
}

impl ImportRewriter {
    pub fn new(dep_map: DependencyMap) -> Self {
        let mut aliases = HashMap::new();
        aliases.insert("@".to_string(), "/src".to_string());

        Self {
            dep_map,
            aliases,
            src_base: "/src".to_string(),
        }
    }

    pub fn rewrite(&self, code: &str, current_file: &Path, timestamp: u64) -> String {
        let mut result = code.to_string();

        // Find and replace all import statements
        // This is a simple regex-based approach. For production, consider using a proper parser.

        // Match: import ... from "specifier"
        // Match: import ... from 'specifier'
        // Match: export ... from "specifier"
        // Match: export ... from 'specifier'
        // Match: import("specifier")
        // Match: import "specifier" (side-effect)

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
    let rewriter = ImportRewriter::new(dep_map);
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

        let rewriter = ImportRewriter::new(dep_map);
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

        let rewriter = ImportRewriter::new(dep_map);
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

        let rewriter = ImportRewriter::new(dep_map);
        let code = r#"import { useQuery } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query/devtools";"#;

        let result = rewriter.rewrite(code, Path::new("App.tsx"), 12345);

        assert!(result.contains("/.forte/deps/tanstack__react-query-abc.js"));
        assert!(result.contains("/.forte/deps/tanstack__react-query__devtools-def.js"));
    }
}
