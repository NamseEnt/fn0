use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};

static WIT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../sdk/wit");

pub fn extract_wit(out_dir: &Path) -> PathBuf {
    let target = out_dir.join("forte-wit");
    if target.exists() {
        std::fs::remove_dir_all(&target).ok();
    }
    std::fs::create_dir_all(&target).expect("failed to create forte-wit dir");
    WIT_DIR
        .extract(&target)
        .expect("failed to extract bundled wit files");
    target
}
