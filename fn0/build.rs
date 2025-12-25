fn main() {
    println!("cargo:rerun-if-changed=js");
    println!("cargo:rerun-if-changed=build.rs");

    // For now, we'll read the deno JS files at runtime from the cargo registry
    // In production, you'd want to bundle them at build time
}
