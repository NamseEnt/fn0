fn validate_cache_root(root: &Path) -> Result<PathBuf, String> {
    let metadata = std::fs::symlink_metadata(root).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("refusing unsafe cache root {}", root.display()));
    }
    std::fs::canonicalize(root).map_err(|error| error.to_string())
}

/// Walk `components` under `root`, refusing anything that is a symlink, is not
/// a directory, or escapes the root.
///
/// `create` is what separates a caller that is about to write from one that is
/// only asking where a part would live. Creating on the read side is what left
/// an empty part directory behind every restore that could not fetch its
/// files, and startup reconciliation refuses a part directory with no
/// metadata.
fn ensure_safe_directory_chain(
    root: &Path,
    components: &[&str],
    create: bool,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(root).map_err(|error| error.to_string())?;
    let canonical_root = validate_cache_root(root)?;
    let mut current = root.to_path_buf();
    // Nothing below a directory that is absent can exist, so the walk keeps
    // appending components to answer "where would this live" without touching
    // the filesystem again.
    let mut absent = false;
    for component in components {
        if !is_safe_path_component(component) {
            return Err(format!("unsafe cache path component {component:?}"));
        }
        current.push(component);
        if absent {
            continue;
        }
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing symlinked cache directory {}",
                    current.display()
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(format!(
                    "cache path is not a directory: {}",
                    current.display()
                ));
            }
            Ok(_) => {}
            // A path that does not exist is the answer to "where would this
            // part live", not a directory to bring into being. Materialising
            // it here is what left an empty part directory behind every
            // restore that could not fetch its files, and startup
            // reconciliation refuses a part directory with no metadata.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if !create {
                    absent = true;
                    continue;
                }
                std::fs::create_dir(&current).map_err(|error| {
                    format!(
                        "failed to create cache directory {}: {error}",
                        current.display()
                    )
                })?;
            }
            Err(error) => return Err(error.to_string()),
        }
        let canonical = std::fs::canonicalize(&current).map_err(|error| error.to_string())?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "cache directory escapes root: {}",
                current.display()
            ));
        }
    }
    Ok(current)
}

fn ensure_existing_cache_dir(canonical_root: &Path, path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "refusing unsafe cache directory {}",
            path.display()
        ));
    }
    let canonical = std::fs::canonicalize(path).map_err(|error| error.to_string())?;
    if !canonical.starts_with(canonical_root) {
        return Err(format!("cache directory escapes root: {}", path.display()));
    }
    Ok(())
}

fn cache_part_dir(parts_root: &Path, descriptor: &ManifestPart) -> Result<PathBuf, String> {
    let dir =
        ensure_safe_directory_chain(parts_root, &[&descriptor.partition, &descriptor.id], false)?;
    for file in [
        DATA_FILE,
        INDEX_FILE,
        META_FILE,
        part::MERGE_TOMBSTONE_FILE,
        UPLOAD_MARKER_FILE,
        ".access",
    ] {
        match std::fs::symlink_metadata(dir.join(file)) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing symlinked cache file {}",
                    dir.join(file).display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(dir)
}

fn validate_cache_tree_no_symlinks(parts_root: &Path) -> Result<(), String> {
    std::fs::create_dir_all(parts_root).map_err(|error| error.to_string())?;
    let canonical_root = validate_cache_root(parts_root)?;
    for partition in std::fs::read_dir(parts_root).map_err(|error| error.to_string())? {
        let partition = partition.map_err(|error| error.to_string())?;
        let metadata =
            std::fs::symlink_metadata(partition.path()).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "refusing symlinked cache partition {}",
                partition.path().display()
            ));
        }
        if !metadata.is_dir() {
            continue;
        }
        ensure_existing_cache_dir(&canonical_root, &partition.path())?;
        for entry in std::fs::read_dir(partition.path()).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let metadata =
                std::fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "refusing symlinked cache directory {}",
                    entry.path().display()
                ));
            }
            if metadata.is_dir() {
                ensure_existing_cache_dir(&canonical_root, &entry.path())?;
                for child in std::fs::read_dir(entry.path()).map_err(|error| error.to_string())? {
                    let child = child.map_err(|error| error.to_string())?;
                    let child_metadata = std::fs::symlink_metadata(child.path())
                        .map_err(|error| error.to_string())?;
                    if child_metadata.file_type().is_symlink() {
                        return Err(format!(
                            "refusing symlinked cache file {}",
                            child.path().display()
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn open_manifest_part(
    dir: &Path,
    descriptor: &ManifestPart,
    require_data: bool,
) -> Result<crate::part::PartReader, String> {
    let part = part::load_part(dir)?;
    if part.meta.id != descriptor.id || part.meta.partition != descriptor.partition {
        return Err(format!(
            "cached part metadata does not match manifest descriptor {}/{}",
            descriptor.partition, descriptor.id
        ));
    }
    if require_data {
        crate::part::PartReader::open(part)
    } else {
        crate::part::PartReader::open_cached(part)
    }
}

