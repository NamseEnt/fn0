fn main() {
    for name in ["CROSSDB_TURSO_TAG", "CROSSDB_TURSO_COMMIT"] {
        println!("cargo:rerun-if-env-changed={name}");
        let value = std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set"));
        println!("cargo:rustc-env={name}={value}");
    }
}
