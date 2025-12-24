use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedField {
    pub name: String,           // camelCase from TypeScript
    #[serde(rename = "rustName")]
    pub rust_name: String,      // snake_case for Rust
    #[serde(rename = "tsType")]
    pub ts_type: String,        // TypeScript type
    #[serde(rename = "isRequired")]
    pub is_required: bool,
    pub documentation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedModel {
    pub name: String,           // PascalCase
    #[serde(rename = "fileName")]
    pub file_name: String,      // kebab-case
    pub kind: ModelKind,
    pub documentation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<ParsedField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<EnumVariant>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
    #[serde(rename = "baseType", skip_serializing_if = "Option::is_none")]
    pub base_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelKind {
    Interface,
    Enum,
    #[serde(rename = "type-alias")]
    TypeAlias,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: String,
}

impl ParsedModel {
    pub fn is_interface(&self) -> bool {
        self.kind == ModelKind::Interface
    }

    pub fn is_enum(&self) -> bool {
        self.kind == ModelKind::Enum
    }

    pub fn required_fields(&self) -> Vec<&ParsedField> {
        self.fields
            .as_ref()
            .map(|fields| fields.iter().filter(|f| f.is_required).collect())
            .unwrap_or_default()
    }

    pub fn optional_fields(&self) -> Vec<&ParsedField> {
        self.fields
            .as_ref()
            .map(|fields| fields.iter().filter(|f| !f.is_required).collect())
            .unwrap_or_default()
    }

    pub fn has_required_fields(&self) -> bool {
        !self.required_fields().is_empty()
    }
}
