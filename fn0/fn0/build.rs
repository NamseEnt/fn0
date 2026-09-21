use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let fixture_dir = manifest_dir.join("test-wasm/dibi-e2e");
    for path in watched_paths(&fixture_dir) {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed={}", manifest_dir.join("../../doc-db/src").display());
    println!("cargo:rerun-if-changed={}", manifest_dir.join("../../forte/sdk/src").display());
    println!("cargo:rerun-if-changed={}", manifest_dir.join("../../forte/sdk/wit").display());

    let manifest_path = fixture_dir.join("Cargo.toml");
    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-wasip2",
            "--manifest-path",
        ])
        .arg(&manifest_path)
        .status()
        .expect("build Dibi E2E fixture");
    assert!(status.success(), "Dibi E2E fixture build failed");

    let wasm_path = fixture_dir.join("target/wasm32-wasip2/release/dibi_e2e.wasm");
    assert!(wasm_path.is_file(), "Dibi E2E fixture output is missing");
    println!("cargo:rustc-env=FN0_DIBI_E2E_WASM={}", wasm_path.display());
}

fn watched_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![root.to_owned()];
    let Ok(entries) = fs::read_dir(root) else {
        return paths;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            paths.extend(watched_paths(&path));
        } else {
            paths.push(path);
        }
    }
    paths
}
