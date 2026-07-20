use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{env, fs, path::Path};

const DEK_KEY: &str = "__dek";
const ENV_FILES: [&str; 2] = ["env.yaml", "env.local.yaml"];

pub fn generate_env() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_dir = Path::new(&manifest_dir).parent().unwrap();
    let output_path = Path::new(&manifest_dir).join("src/env_generated.rs");

    for file in ENV_FILES {
        println!("cargo:rerun-if-changed=../{file}");
    }

    write_env_module(project_dir, &output_path);
}

fn write_env_module(project_dir: &Path, output_path: &Path) {
    let keys = collect_keys(project_dir);
    let tokens = generate_env_code(&keys);

    let syntax_tree = syn::parse2::<syn::File>(tokens).expect("Failed to parse generated env code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // Comparing against `Option` rather than a defaulted String so that a
    // missing file is still written when the generated content is empty.
    if fs::read_to_string(output_path).ok().as_deref() != Some(formatted.as_str()) {
        fs::write(output_path, formatted).unwrap();
    }
}

fn collect_keys(project_dir: &Path) -> Vec<String> {
    let mut keys: Vec<String> = ENV_FILES
        .iter()
        .flat_map(|file| read_keys(&project_dir.join(file)))
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

fn read_keys(path: &Path) -> Vec<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    if content.trim().is_empty() {
        return Vec::new();
    }
    let value: serde_yaml::Value =
        serde_yaml::from_str(&content).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let serde_yaml::Value::Mapping(mapping) = value else {
        panic!("{} must contain a mapping", path.display());
    };
    mapping
        .keys()
        .map(|key| {
            key.as_str()
                .unwrap_or_else(|| panic!("{}: key is not a string", path.display()))
        })
        .filter(|key| *key != DEK_KEY)
        .map(str::to_string)
        .collect()
}

fn to_snake_case(s: &str) -> String {
    s.to_lowercase()
}

fn generate_env_code(keys: &[String]) -> TokenStream {
    if keys.is_empty() {
        return quote! {};
    }

    let fn_declarations: Vec<TokenStream> = keys
        .iter()
        .map(|key| {
            let fn_name = format_ident!("{}", to_snake_case(key));
            let static_name = format_ident!("{}", key.to_uppercase());

            quote! {
                pub fn #fn_name() -> &'static str {
                    static #static_name: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
                        std::env::var(#key).unwrap()
                    });
                    &#static_name
                }
            }
        })
        .collect();

    quote! {
        #![allow(dead_code)]

        #(#fn_declarations)*
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn merges_keys_from_both_files_skipping_dek() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("env.yaml"),
            "__dek:\n  encrypted: ct\nSHARED: value\nSECRET_KEY:\n  secret: ct\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("env.local.yaml"),
            "SECRET_KEY: local\nLOCAL_ONLY: value\n",
        )
        .unwrap();

        assert_eq!(
            collect_keys(dir.path()),
            vec!["LOCAL_ONLY", "SECRET_KEY", "SHARED"]
        );
    }

    #[test]
    fn missing_and_empty_files_yield_no_keys() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("env.yaml"), "").unwrap();
        assert!(collect_keys(dir.path()).is_empty());
    }

    #[test]
    fn module_is_written_even_when_there_are_no_keys() {
        let dir = TempDir::new().unwrap();
        let output_path = dir.path().join("env_generated.rs");
        write_env_module(dir.path(), &output_path);
        assert_eq!(fs::read_to_string(&output_path).unwrap(), "");
    }

    #[test]
    fn module_declares_an_accessor_per_key() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("env.yaml"), "COOKIE_SECRET: abc\n").unwrap();
        let output_path = dir.path().join("env_generated.rs");
        write_env_module(dir.path(), &output_path);

        let generated = fs::read_to_string(&output_path).unwrap();
        assert!(generated.contains("pub fn cookie_secret() -> &'static str"));
        assert!(generated.contains("std::env::var(\"COOKIE_SECRET\")"));
    }

    #[test]
    #[should_panic(expected = "must contain a mapping")]
    fn non_mapping_yaml_panics() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("env.yaml"), "- just\n- a\n- list\n").unwrap();
        collect_keys(dir.path());
    }
}
