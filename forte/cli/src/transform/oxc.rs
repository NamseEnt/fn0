use crate::transform::{ImportInfo, TransformConfig};
use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{JsxOptions, ReactRefreshOptions, TransformOptions, Transformer};
use std::path::Path;

/// Result from OXC transformation
pub struct OxcTransformResult {
    pub code: String,
    pub map: Option<String>,
    pub imports: Vec<ImportInfo>,
    pub has_react_components: bool,
}

/// Transform a TypeScript/JSX file using OXC
pub fn transform_with_oxc(
    file_path: &Path,
    source_text: &str,
    config: &TransformConfig,
) -> Result<OxcTransformResult> {
    let allocator = Allocator::default();

    // Determine source type from file extension
    let source_type = SourceType::from_path(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to determine source type: {:?}", e))?;

    // Parse the source code
    let parser_ret = Parser::new(&allocator, source_text, source_type).parse();

    if !parser_ret.errors.is_empty() {
        let errors: Vec<String> = parser_ret
            .errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();
        anyhow::bail!("Parse errors in {:?}:\n{}", file_path, errors.join("\n"));
    }

    let mut program = parser_ret.program;

    // Extract imports before transformation
    let imports = extract_imports(&program);

    // Build semantic information
    let semantic_ret = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .build(&program);

    if !semantic_ret.errors.is_empty() {
        let errors: Vec<String> = semantic_ret
            .errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();
        anyhow::bail!(
            "Semantic errors in {:?}:\n{}",
            file_path,
            errors.join("\n")
        );
    }

    let scoping = semantic_ret.semantic.into_scoping();

    // Configure transform options
    let mut transform_options = TransformOptions::default();

    // TypeScript transformation is enabled by default, just use default options

    // Enable JSX transformation with automatic runtime (React 17+)
    transform_options.jsx = JsxOptions {
        jsx_plugin: true,
        display_name_plugin: true,
        jsx_self_plugin: config.development,
        jsx_source_plugin: config.development,
        development: config.development,
        refresh: if config.react_refresh {
            Some(ReactRefreshOptions {
                refresh_reg: "$RefreshReg$".to_string(),
                refresh_sig: "$RefreshSig$".to_string(),
                emit_full_signatures: true,
            })
        } else {
            None
        },
        ..JsxOptions::default()
    };

    // Run the transformer
    let transform_ret = Transformer::new(&allocator, file_path, &transform_options)
        .build_with_scoping(scoping, &mut program);

    if !transform_ret.errors.is_empty() {
        let errors: Vec<String> = transform_ret
            .errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();
        anyhow::bail!(
            "Transform errors in {:?}:\n{}",
            file_path,
            errors.join("\n")
        );
    }

    // Generate output code
    let codegen = Codegen::new();
    let output = codegen.build(&program);

    // Check if the file has React components (heuristic: has JSX or imports React)
    let has_react_components = imports
        .iter()
        .any(|i| i.specifier == "react" || i.specifier.starts_with("react/"))
        || source_text.contains("jsx")
        || source_text.contains("JSX")
        || source_text.contains('<');

    Ok(OxcTransformResult {
        code: output.code,
        map: None, // TODO: Add source map support
        imports,
        has_react_components,
    })
}

/// Extract import information from the AST
fn extract_imports(program: &oxc_ast::ast::Program<'_>) -> Vec<ImportInfo> {
    let mut imports = Vec::new();

    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(decl) => {
                let specifier = decl.source.value.to_string();
                let kind = ImportInfo::classify(&specifier);
                imports.push(ImportInfo { specifier, kind });
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(source) = &decl.source {
                    let specifier = source.value.to_string();
                    let kind = ImportInfo::classify(&specifier);
                    imports.push(ImportInfo { specifier, kind });
                }
            }
            Statement::ExportAllDeclaration(decl) => {
                let specifier = decl.source.value.to_string();
                let kind = ImportInfo::classify(&specifier);
                imports.push(ImportInfo { specifier, kind });
            }
            _ => {}
        }
    }

    imports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::ImportKind;

    #[test]
    fn test_transform_simple_tsx() {
        let config = TransformConfig::new("/tmp/test");
        let source = r#"
            import React from 'react';

            export function Hello({ name }: { name: string }) {
                return <div>Hello, {name}!</div>;
            }
        "#;

        let result =
            transform_with_oxc(Path::new("test.tsx"), source, &config).expect("transform failed");

        // Check that JSX was transformed
        assert!(
            result.code.contains("createElement") || result.code.contains("jsx"),
            "JSX should be transformed. Got: {}",
            result.code
        );
        // Check that TypeScript types are removed (no `: string` type annotation)
        assert!(
            !result.code.contains(": string"),
            "TypeScript types should be removed. Got: {}",
            result.code
        );
        assert!(result.imports.iter().any(|i| i.specifier == "react"));
    }

    #[test]
    fn test_import_classification() {
        assert_eq!(ImportInfo::classify("react"), ImportKind::Bare);
        assert_eq!(ImportInfo::classify("react-dom"), ImportKind::Bare);
        assert_eq!(ImportInfo::classify("@scope/package"), ImportKind::Bare);
        assert_eq!(ImportInfo::classify("./Button"), ImportKind::Relative);
        assert_eq!(ImportInfo::classify("../utils"), ImportKind::Relative);
        assert_eq!(ImportInfo::classify("@/lib/utils"), ImportKind::Alias);
    }

    #[test]
    fn test_react_refresh_transform() {
        let mut config = TransformConfig::new("/tmp/test");
        config.react_refresh = true;

        let source = r#"
            import React from 'react';

            export function Counter() {
                const [count, setCount] = React.useState(0);
                return <button onClick={() => setCount(c => c + 1)}>{count}</button>;
            }
        "#;

        let result =
            transform_with_oxc(Path::new("Counter.tsx"), source, &config).expect("transform failed");

        assert!(
            result.code.contains("$RefreshReg$") || result.code.contains("_s"),
            "React Refresh registration should be present. Got: {}",
            result.code
        );
    }
}
