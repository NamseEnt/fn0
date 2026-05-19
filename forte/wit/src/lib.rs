//! WASI wit definitions shared by forte-sdk (its own bindings) and
//! forte-codegen (which routes them to user crates via build scripts).
//!
//! The wit tree is embedded at compile time via `include_dir!`, so any
//! downstream crate can call [`extract_wit`] from a build script to
//! materialize the same wit files into an arbitrary directory.

use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};

static WIT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/wit");

pub fn extract_wit(target_dir: &Path) -> PathBuf {
    if target_dir.exists() {
        std::fs::remove_dir_all(target_dir).ok();
    }
    std::fs::create_dir_all(target_dir).expect("failed to create wit target dir");
    WIT_DIR
        .extract(target_dir)
        .expect("failed to extract bundled wit files");
    target_dir.to_path_buf()
}
