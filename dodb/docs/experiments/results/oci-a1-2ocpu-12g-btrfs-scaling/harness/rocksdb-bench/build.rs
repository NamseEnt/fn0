use std::path::PathBuf;

fn main() {
    for name in ["CROSSDB_ROCKSDB_TAG", "CROSSDB_ROCKSDB_COMMIT", "CROSSDB_ROCKSDB_DIR"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let tag = std::env::var("CROSSDB_ROCKSDB_TAG").expect("CROSSDB_ROCKSDB_TAG must be set");
    let commit =
        std::env::var("CROSSDB_ROCKSDB_COMMIT").expect("CROSSDB_ROCKSDB_COMMIT must be set");
    println!("cargo:rustc-env=CROSSDB_ROCKSDB_TAG={tag}");
    println!("cargo:rustc-env=CROSSDB_ROCKSDB_COMMIT={commit}");
    let rocksdb_dir =
        PathBuf::from(std::env::var("CROSSDB_ROCKSDB_DIR").expect("CROSSDB_ROCKSDB_DIR must be set"));
    println!("cargo:rerun-if-changed=shim/rocksdb_shim.cc");
    println!("cargo:rerun-if-changed={}", rocksdb_dir.join("librocksdb.a").display());
    cc::Build::new()
        .cpp(true)
        .file("shim/rocksdb_shim.cc")
        .include(rocksdb_dir.join("include"))
        .flag("-std=c++20")
        .flag("-fno-rtti")
        .flag("-march=armv8-a+crc+crypto")
        .flag("-O2")
        .define("NDEBUG", None)
        .warnings(true)
        .compile("crossdb_rocksdb_shim");
    println!("cargo:rustc-link-search=native={}", rocksdb_dir.display());
    println!("cargo:rustc-link-lib=static=rocksdb");
    let make_config = std::fs::read_to_string(rocksdb_dir.join("make_config.mk"))
        .expect("make_config.mk should exist after the RocksDB build");
    for line in make_config.lines() {
        if let Some(flags) = line.strip_prefix("PLATFORM_LDFLAGS=") {
            for flag in flags.split_whitespace() {
                if let Some(library) = flag.strip_prefix("-l") {
                    println!("cargo:rustc-link-lib={library}");
                }
            }
        }
    }
    println!("cargo:rustc-link-lib=stdc++");
}
