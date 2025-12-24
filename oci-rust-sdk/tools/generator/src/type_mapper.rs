use heck::{ToSnakeCase, ToPascalCase};

#[derive(Debug, Clone)]
pub struct TypeMapper;

impl TypeMapper {
    pub fn new() -> Self {
        Self
    }

    /// Map TypeScript type to Rust type
    /// Returns (rust_type, needs_hashmap_import, needs_datetime_import)
    pub fn map_type(&self, ts_type: &str, is_optional: bool) -> (String, bool, bool) {
        let (base_type, needs_hashmap, needs_datetime) = self.map_base_type(ts_type);

        let final_type = if is_optional {
            format!("Option<{}>", base_type)
        } else {
            base_type
        };

        (final_type, needs_hashmap, needs_datetime)
    }

    fn map_base_type(&self, ts_type: &str) -> (String, bool, bool) {
        let cleaned = ts_type.trim();

        // Handle primitive types
        match cleaned {
            "string" => return ("String".to_string(), false, false),
            "boolean" => return ("bool".to_string(), false, false),
            "number" => return ("i64".to_string(), false, false),
            "Date" => return ("DateTime<Utc>".to_string(), false, true),
            "any" => return ("serde_json::Value".to_string(), false, false),
            _ => {}
        }

        // Handle Array<T>
        if cleaned.starts_with("Array<") {
            let inner = self.extract_generic(cleaned, "Array");
            let (inner_type, h, d) = self.map_base_type(&inner);
            return (format!("Vec<{}>", inner_type), h, d);
        }

        // Handle { [key: string]: V } - TypeScript dictionary/map
        if cleaned.contains("[key:") || cleaned.contains("{ [key") {
            let value_type = self.extract_dict_value_type(cleaned);
            let (v_type, _h, d) = self.map_base_type(&value_type);

            // Handle nested maps: { [key: string]: { [key: string]: any } }
            if cleaned.matches("{ [key").count() > 1 {
                return (
                    format!("HashMap<String, HashMap<String, serde_json::Value>>"),
                    true,
                    d
                );
            }

            return (format!("HashMap<String, {}>", v_type), true, d);
        }

        // Handle union types (A | B | C) - will become Rust enum
        if cleaned.contains(" | ") && !cleaned.contains("undefined") {
            // For now, treat as the first type or custom enum
            // More complex handling would require generating discriminated union enums
            let first_type = cleaned.split(" | ").next().unwrap_or("String");
            return self.map_base_type(first_type);
        }

        // Custom types - assume it's a model name, convert to PascalCase
        let rust_name = self.to_rust_type_name(cleaned);
        (rust_name, false, false)
    }

    fn extract_generic(&self, type_str: &str, _wrapper: &str) -> String {
        let start = type_str.find('<').unwrap_or(0) + 1;
        let end = type_str.rfind('>').unwrap_or(type_str.len());
        type_str[start..end].trim().to_string()
    }

    fn extract_dict_value_type(&self, type_str: &str) -> String {
        // Pattern: { [key: string]: VALUE_TYPE }
        if let Some(colon_pos) = type_str.rfind("]:") {
            let after_colon = &type_str[colon_pos + 2..];
            let end = after_colon.find('}').unwrap_or(after_colon.len());
            return after_colon[..end].trim().to_string();
        }
        "serde_json::Value".to_string()
    }

    fn to_rust_type_name(&self, ts_name: &str) -> String {
        // Remove "model." prefix if present
        let cleaned = ts_name.replace("model.", "").replace("import(", "").replace(")", "");

        // If it's already PascalCase, keep it
        if cleaned.chars().next().map_or(false, |c| c.is_uppercase()) {
            cleaned
        } else {
            cleaned.to_pascal_case()
        }
    }

    pub fn to_snake_case(&self, name: &str) -> String {
        name.to_snake_case()
    }

    pub fn is_string_type(&self, rust_type: &str) -> bool {
        rust_type == "String"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        let mapper = TypeMapper::new();

        let (t, h, d) = mapper.map_type("string", false);
        assert_eq!(t, "String");
        assert!(!h && !d);

        let (t, h, d) = mapper.map_type("boolean", true);
        assert_eq!(t, "Option<bool>");
        assert!(!h && !d);

        let (t, h, d) = mapper.map_type("number", false);
        assert_eq!(t, "i64");
        assert!(!h && !d);
    }

    #[test]
    fn test_array_type() {
        let mapper = TypeMapper::new();

        let (t, h, d) = mapper.map_type("Array<string>", false);
        assert_eq!(t, "Vec<String>");

        let (t, h, d) = mapper.map_type("Array<Instance>", true);
        assert_eq!(t, "Option<Vec<Instance>>");
    }

    #[test]
    fn test_hashmap_type() {
        let mapper = TypeMapper::new();

        let (t, h, d) = mapper.map_type("{ [key: string]: string }", false);
        assert_eq!(t, "std::collections::HashMap<String, String>");
        assert!(h);
    }

    #[test]
    fn test_custom_type() {
        let mapper = TypeMapper::new();

        let (t, h, d) = mapper.map_type("CreateVnicDetails", false);
        assert_eq!(t, "CreateVnicDetails");

        let (t, _, _) = mapper.map_type("model.Instance", true);
        assert_eq!(t, "Option<Instance>");
    }
}
