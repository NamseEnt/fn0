use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

fn main() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let target_dir = resolve_target_dir(Path::new(&manifest_dir));
    let target_binary = target_dir.join(&profile).join("forte");

    let home = env::var("HOME").expect("HOME not set");
    let cargo_bin = PathBuf::from(&home).join(".cargo").join("bin");

    if !cargo_bin.exists() {
        fs::create_dir_all(&cargo_bin).ok();
    }

    let symlink_path = cargo_bin.join("forte");

    if symlink_path.exists() || symlink_path.is_symlink() {
        fs::remove_file(&symlink_path).ok();
    }

    if let Err(e) = symlink(&target_binary, &symlink_path) {
        println!(
            "cargo:warning=Failed to create symlink: {} -> {}: {}",
            symlink_path.display(),
            target_binary.display(),
            e
        );
    } else {
        println!(
            "cargo:warning=Symlinked: {} -> {}",
            symlink_path.display(),
            target_binary.display()
        );
    }

    let workspace_root = PathBuf::from(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root from forte/cli manifest dir")
        .to_path_buf();

    let targets: &[(&str, &str, &str)] = &[
        ("FORTE_JSON_VERSION", "forte/json/Cargo.toml", "forte-json"),
        ("FORTE_SDK_VERSION", "forte/sdk/Cargo.toml", "forte-sdk"),
        (
            "FORTE_CODEGEN_VERSION",
            "forte/codegen/Cargo.toml",
            "forte-codegen",
        ),
        ("FN0_DOC_DB_VERSION", "doc-db/Cargo.toml", "fn0-doc-db"),
        (
            "FN0_OBJECT_STORAGE_VERSION",
            "object-storage/Cargo.toml",
            "fn0-object-storage",
        ),
    ];

    for (env_var, rel_path, expected_name) in targets {
        let manifest = workspace_root.join(rel_path);
        let version = read_package_version(&manifest, expected_name);
        println!("cargo:rustc-env={env_var}={version}");
        println!("cargo:rerun-if-changed={}", manifest.display());
    }

    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");
}

fn resolve_target_dir(manifest_dir: &Path) -> PathBuf {
    if let Ok(t) = env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(t);
    }
    let mut dir = manifest_dir.parent();
    while let Some(d) = dir {
        let manifest = d.join("Cargo.toml");
        if manifest.exists() {
            let content = fs::read_to_string(&manifest).unwrap_or_default();
            if content.contains("[workspace]") {
                return d.join("target");
            }
        }
        dir = d.parent();
    }
    manifest_dir.join("target")
}

fn read_package_version(path: &Path, expected_name: &str) -> String {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

    let mut in_package = false;
    let mut found_name: Option<String> = None;
    let mut found_version: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(section) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_package = section == "package";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name") {
            found_name = Some(extract_string_value(rest));
        } else if let Some(rest) = trimmed.strip_prefix("version") {
            found_version = Some(extract_string_value(rest));
        }
        if found_name.is_some() && found_version.is_some() {
            break;
        }
    }

    let name = found_name.unwrap_or_else(|| {
        panic!("no [package].name in {}", path.display());
    });
    assert_eq!(
        name,
        expected_name,
        "{} has [package].name = {name:?}, expected {expected_name:?}",
        path.display()
    );

    found_version.unwrap_or_else(|| panic!("no [package].version in {}", path.display()))
}

fn extract_string_value(rest: &str) -> String {
    let after_eq = rest
        .trim_start()
        .strip_prefix('=')
        .expect("expected `=` after key")
        .trim();
    after_eq
        .trim_matches('"')
        .to_string()
}
