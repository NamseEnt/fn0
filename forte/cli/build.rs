use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

fn main() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let target_binary = PathBuf::from(&manifest_dir)
        .join("target")
        .join(&profile)
        .join("forte");

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

    println!("cargo:rerun-if-changed=build.rs");
}
