use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let wit_dir = Path::new(&manifest_dir).join("wit");
    forte_wit::extract_wit(&wit_dir);
    println!("cargo:rerun-if-changed=build.rs");
}
