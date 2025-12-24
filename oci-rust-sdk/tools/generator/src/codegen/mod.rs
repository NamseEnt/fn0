use crate::models::{ParsedField, ParsedModel};
use crate::type_mapper::TypeMapper;
use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use tera::{Tera, Context as TeraContext};

pub struct CodeGenerator {
    tera: Tera,
    type_mapper: TypeMapper,
}

impl CodeGenerator {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::new("tools/generator/templates/*.tera")
            .context("Failed to load Tera templates")?;

        // Disable auto-escaping since we're generating code
        tera.autoescape_on(vec![]);

        Ok(Self {
            tera,
            type_mapper: TypeMapper::new(),
        })
    }

    pub fn generate_model(&self, model: &ParsedModel) -> Result<String> {
        if model.is_enum() {
            self.generate_enum(model)
        } else {
            self.generate_struct(model)
        }
    }

    fn generate_enum(&self, model: &ParsedModel) -> Result<String> {
        // Clean documentation - merge multiple lines into single line to avoid clippy warnings
        let clean_doc = model.documentation
            .trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let mut context = TeraContext::new();
        context.insert("name", &model.name);
        context.insert("documentation", &clean_doc);
        context.insert("variants", model.variants.as_ref().unwrap_or(&vec![]));

        self.tera
            .render("enum.rs.tera", &context)
            .context("Failed to render enum template")
    }

    fn generate_struct(&self, model: &ParsedModel) -> Result<String> {
        let empty_fields = vec![];
        let fields = model.fields.as_ref().unwrap_or(&empty_fields);

        // Prepare field data for template
        let mut all_fields = vec![];
        let mut required_fields = vec![];
        let mut optional_fields = vec![];
        let mut needs_hashmap = false;
        let mut needs_datetime = false;

        for field in fields {
            let (rust_type, h, d) = self.type_mapper.map_type(&field.ts_type, !field.is_required);
            let base_rust_type = if field.is_required {
                rust_type.clone()
            } else {
                // Remove Option<> wrapper for base type
                rust_type
                    .strip_prefix("Option<")
                    .and_then(|s| s.strip_suffix(">"))
                    .unwrap_or(&rust_type)
                    .to_string()
            };

            needs_hashmap |= h;
            needs_datetime |= d;

            let is_string = self.type_mapper.is_string_type(&base_rust_type);

            // Clean documentation - merge multiple lines into single line to avoid clippy warnings
            let clean_doc = field.documentation
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ");

            // Strip r# prefix for method names
            let rust_name_base = field.rust_name.strip_prefix("r#").unwrap_or(&field.rust_name);

            let field_data = json!({
                "name": field.name,                    // Original TypeScript field name (for serde)
                "rust_name": field.rust_name,          // Rust field name (may have r# prefix)
                "rust_name_base": rust_name_base,      // Rust name without r# (for method names)
                "rust_type": rust_type,
                "base_rust_type": base_rust_type,
                "documentation": clean_doc,
                "is_string": is_string,
            });

            all_fields.push(field_data.clone());

            if field.is_required {
                required_fields.push(field_data);
            } else {
                optional_fields.push(field_data);
            }
        }

        // Clean model documentation - merge multiple lines into single line to avoid clippy warnings
        let clean_model_doc = model.documentation
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let mut context = TeraContext::new();
        context.insert("name", &model.name);
        context.insert("documentation", &clean_model_doc);
        context.insert("all_fields", &all_fields);
        context.insert("required_fields", &required_fields);
        context.insert("optional_fields", &optional_fields);
        context.insert("has_required_fields", &!required_fields.is_empty());
        context.insert("needs_hashmap", &needs_hashmap);
        context.insert("needs_datetime", &needs_datetime);

        self.tera
            .render("model.rs.tera", &context)
            .map_err(|e| {
                eprintln!("Tera error: {:?}", e);
                anyhow::Error::msg(format!("Failed to render model template: {}", e))
            })
    }
}
