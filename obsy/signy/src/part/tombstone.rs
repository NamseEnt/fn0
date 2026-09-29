pub fn write_merge_tombstone(
    part_dir: &Path,
    parts_root: &Path,
    old_dirs: &[PathBuf],
) -> io::Result<()> {
    let canonical_parts_root = fs::canonicalize(parts_root)?;
    let relative_old_dirs: Vec<PathBuf> = old_dirs
        .iter()
        .map(|old_dir| {
            let canonical_old_dir = fs::canonicalize(old_dir)?;
            canonical_old_dir
                .strip_prefix(&canonical_parts_root)
                .map(Path::to_path_buf)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
        })
        .collect::<io::Result<Vec<_>>>()?;
    let tomb = MergeTombstone {
        old_dirs: relative_old_dirs,
    };
    let s = serde_json::to_string(&tomb).map_err(io::Error::other)?;
    let path = part_dir.join(MERGE_TOMBSTONE_FILE);
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &s)?;
    sync_file(&tmp)?;
    fs::rename(&tmp, &path)?;
    sync_file(&path)?;
    fsync_dir(part_dir)?;
    Ok(())
}

pub fn read_merge_tombstone(part_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let path = part_dir.join(MERGE_TOMBSTONE_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let tomb: MergeTombstone = serde_json::from_str(&s).map_err(|e| e.to_string())?;
    for old_dir in &tomb.old_dirs {
        validate_tombstone_part_path(old_dir)?;
    }
    Ok(tomb.old_dirs)
}

pub fn read_merge_tombstone_dirs(
    part_dir: &Path,
    parts_root: &Path,
) -> Result<Vec<PathBuf>, String> {
    let relative_dirs = read_merge_tombstone(part_dir)?;
    let canonical_root = fs::canonicalize(parts_root).map_err(|e| {
        format!(
            "failed to canonicalize parts root {}: {e}",
            parts_root.display()
        )
    })?;

    relative_dirs
        .into_iter()
        .map(|relative_dir| {
            let dir = parts_root.join(&relative_dir);
            let parent = dir
                .parent()
                .ok_or_else(|| format!("part directory has no parent: {}", dir.display()))?;
            let canonical_parent = fs::canonicalize(parent).map_err(|e| {
                format!(
                    "failed to canonicalize tombstone target parent {}: {e}",
                    parent.display()
                )
            })?;
            canonical_parent
                .strip_prefix(&canonical_root)
                .map_err(|_| {
                    format!(
                        "merge tombstone target escapes parts root: {}",
                        dir.display()
                    )
                })?;

            match fs::canonicalize(&dir) {
                Ok(canonical_dir) => {
                    canonical_dir.strip_prefix(&canonical_root).map_err(|_| {
                        format!(
                            "merge tombstone target escapes parts root: {}",
                            dir.display()
                        )
                    })?;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!(
                        "failed to canonicalize tombstone target {}: {error}",
                        dir.display()
                    ));
                }
            }
            Ok(dir)
        })
        .collect()
}

fn validate_tombstone_part_path(path: &Path) -> Result<(), String> {
    let components: Vec<_> = path.components().collect();
    if components.len() != 2
        || components
            .iter()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(format!(
            "invalid merge tombstone part path: {}",
            path.display()
        ));
    }
    Ok(())
}

pub fn remove_merge_tombstone(part_dir: &Path) -> io::Result<()> {
    let path = part_dir.join(MERGE_TOMBSTONE_FILE);
    fs::remove_file(&path)?;
    fsync_dir(part_dir)?;
    Ok(())
}

fn sync_file(path: &Path) -> io::Result<()> {
    let f = fs::File::open(path)?;
    f.sync_all()?;
    let dir = fs::File::open(path.parent().unwrap_or(Path::new(".")))?;
    dir.sync_all()?;
    Ok(())
}

fn canonical_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn fsync_dir(path: &Path) -> io::Result<()> {
    let dir = fs::File::open(path)?;
    dir.sync_all()?;
    Ok(())
}

pub fn remove_part_dirs(dirs: &[PathBuf]) -> Result<(), String> {
    let mut parents = std::collections::BTreeSet::new();
    let mut first_error = None;

    for dir in dirs {
        match dir.parent() {
            Some(parent) => {
                parents.insert(parent.to_path_buf());
            }
            None => {
                first_error.get_or_insert_with(|| {
                    format!("part directory has no parent: {}", dir.display())
                });
                continue;
            }
        }
        match fs::symlink_metadata(dir) {
            Ok(_) => {
                if let Err(error) = fs::remove_dir_all(dir) {
                    first_error.get_or_insert_with(|| {
                        format!("failed to remove part directory {}: {error}", dir.display())
                    });
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                first_error.get_or_insert_with(|| {
                    format!(
                        "failed to inspect part directory {}: {error}",
                        dir.display()
                    )
                });
            }
        }
    }

    for parent in parents {
        if let Err(error) = fsync_dir(&parent) {
            first_error.get_or_insert_with(|| {
                format!("failed to fsync part parent {}: {error}", parent.display())
            });
        }
    }

    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

