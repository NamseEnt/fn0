use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let wit_dir = manifest_dir.join("wit");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    let reference = out_dir.join("forte-wit-reference");

    forte_wit::extract_wit(&reference);

    let mut drift = Vec::new();
    compare_dirs(&reference, &wit_dir, Path::new(""), &mut drift);

    if !drift.is_empty() {
        panic!(
            "forte/sdk/wit is out of sync with forte-wit (source of truth).\n\
             Drift: {drift:?}\n\
             Resync with: rm -rf forte/sdk/wit && cp -r forte/wit/wit forte/sdk/wit"
        );
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn compare_dirs(reference: &Path, sdk: &Path, rel: &Path, drift: &mut Vec<String>) {
    let r_path = reference.join(rel);
    let s_path = sdk.join(rel);

    if r_path.is_dir() {
        if !s_path.is_dir() {
            drift.push(format!("{} (missing in sdk)", rel.display()));
            return;
        }
        for entry in fs::read_dir(&r_path)
            .unwrap_or_else(|e| panic!("read_dir {}: {e}", r_path.display()))
            .flatten()
        {
            compare_dirs(reference, sdk, &rel.join(entry.file_name()), drift);
        }
    } else if r_path.is_file() {
        let r = fs::read(&r_path).unwrap_or_default();
        let s = fs::read(&s_path).unwrap_or_default();
        if r != s {
            drift.push(rel.display().to_string());
        }
    }
}
