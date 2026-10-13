use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=DODB_ROCKSDB_DIR");
    let rocksdb_dir = PathBuf::from(
        std::env::var("DODB_ROCKSDB_DIR")
            .expect("DODB_ROCKSDB_DIR must name a RocksDB 11.8.1 build"),
    );
    let source_commit = std::process::Command::new("git")
        .args([
            "-C",
            rocksdb_dir.to_str().expect("RocksDB path must be UTF-8"),
            "rev-parse",
            "HEAD",
        ])
        .output()
        .expect("git must be available to verify the RocksDB source commit");
    assert!(
        source_commit.status.success(),
        "RocksDB source directory must be a git checkout"
    );
    let source_commit = String::from_utf8(source_commit.stdout)
        .expect("git commit output must be UTF-8")
        .trim()
        .to_owned();
    assert_eq!(source_commit, "abeebd9630f11bd08c28b7bd43c7bdfc62050654");
    println!("cargo:rustc-env=DODB_ROCKSDB_COMMIT={source_commit}");
    println!("cargo:rerun-if-changed=shim/transaction_shim.cc");
    println!(
        "cargo:rerun-if-changed={}",
        rocksdb_dir.join("librocksdb.a").display()
    );
    cc::Build::new()
        .cpp(true)
        .file("shim/transaction_shim.cc")
        .include(rocksdb_dir.join("include"))
        .flag_if_supported("-std=c++20")
        .flag_if_supported("-fno-rtti")
        .warnings(true)
        .compile("dodb_rocksdb_transaction_shim");
    println!("cargo:rustc-link-search=native={}", rocksdb_dir.display());
    println!("cargo:rustc-link-lib=static=rocksdb");
    let make_config = std::fs::read_to_string(rocksdb_dir.join("make_config.mk"))
        .expect("make_config.mk must exist in the RocksDB build directory");
    for line in make_config.lines() {
        if let Some(flags) = line.strip_prefix("PLATFORM_LDFLAGS=") {
            for flag in flags.split_whitespace() {
                if let Some(library) = flag.strip_prefix("-l") {
                    println!("cargo:rustc-link-lib={library}");
                }
            }
        }
    }
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }
}
