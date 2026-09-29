use std::{env, fs, path::Path};

fn main() {
    forte_codegen::generate_routes();
    forte_codegen::generate_env();
    let rust_manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let project_dir = Path::new(&rust_manifest_dir)
        .parent()
        .expect("Rust manifest directory has no project parent");
    for generated_file in [
        Path::new(&rust_manifest_dir).join("src/route_generated.rs"),
        project_dir.join("fe/src/paths.generated.ts"),
    ] {
        let generated = fs::read_to_string(&generated_file).expect("generated routes are missing");
        let normalized = generated.replace("/api/dodb_write", "/api/dodb-write");
        if generated != normalized {
            fs::write(generated_file, normalized)
                .expect("could not normalize the DODB write route");
        }
    }
}
