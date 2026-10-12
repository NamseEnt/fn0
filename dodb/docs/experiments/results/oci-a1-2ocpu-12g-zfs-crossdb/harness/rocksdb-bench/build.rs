use std::path::PathBuf;

fn main() {
    for name in ["CROSSDB_ROCKSDB_TAG", "CROSSDB_ROCKSDB_COMMIT", "CROSSDB_ROCKSDB_DIR", "CROSSDB_SOURCE_CLEAN"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let Ok(rocksdb_directory) = std::env::var("CROSSDB_ROCKSDB_DIR") else {
        return;
    };
    let tag = std::env::var("CROSSDB_ROCKSDB_TAG").expect("CROSSDB_ROCKSDB_TAG must be set with CROSSDB_ROCKSDB_DIR");
    let commit = std::env::var("CROSSDB_ROCKSDB_COMMIT").expect("CROSSDB_ROCKSDB_COMMIT must be set with CROSSDB_ROCKSDB_DIR");
    println!("cargo:rustc-env=CROSSDB_ROCKSDB_TAG={tag}");
    println!("cargo:rustc-env=CROSSDB_ROCKSDB_COMMIT={commit}");
    if let Ok(source_sha) = std::env::var("CROSSDB_SOURCE_SHA") {
        println!("cargo:rustc-env=CROSSDB_SOURCE_SHA={source_sha}");
    }
    if let Ok(source_clean) = std::env::var("CROSSDB_SOURCE_CLEAN") {
        println!("cargo:rustc-env=CROSSDB_SOURCE_CLEAN={source_clean}");
    }
    let rocksdb_dir = PathBuf::from(rocksdb_directory);
    println!("cargo:rerun-if-changed=shim/rocksdb_shim.cc");
    println!("cargo:rerun-if-changed={}", rocksdb_dir.join("librocksdb.a").display());
    cc::Build::new()
        .cpp(true)
        .file("shim/rocksdb_shim.cc")
        .include(rocksdb_dir.join("include"))
        .flag("-std=c++20")
        .flag("-fno-rtti")
        .flag("-march=native")
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
