fn canonical_dir_set(dirs: &[PathBuf]) -> Result<std::collections::BTreeSet<PathBuf>, String> {
    dirs.iter()
        .map(|dir| {
            std::fs::canonicalize(dir)
                .map_err(|error| format!("failed to canonicalize {}: {error}", dir.display()))
        })
        .collect()
}

fn verify_merge_tombstones(
    new_part_dirs: &[PathBuf],
    parts_root: &Path,
    expected_old_dirs: &[PathBuf],
) -> Result<Vec<PathBuf>, String> {
    if new_part_dirs.is_empty() {
        return Err("merge produced no replacement parts".to_string());
    }
    if expected_old_dirs.is_empty() {
        return Err("merge has no input parts".to_string());
    }

    let expected = canonical_dir_set(expected_old_dirs)?;
    let mut cleanup_old_dirs = None;
    for new_dir in new_part_dirs {
        let tombstoned_old_dirs = part::read_merge_tombstone_dirs(new_dir, parts_root)
            .map_err(|error| format!("failed to read {}: {error}", new_dir.display()))?;
        if tombstoned_old_dirs.is_empty() {
            return Err(format!(
                "merge tombstone in {} contains no input parts",
                new_dir.display()
            ));
        }
        let actual = canonical_dir_set(&tombstoned_old_dirs)?;
        if actual != expected {
            return Err(format!(
                "merge tombstone in {} does not match the intended input parts",
                new_dir.display()
            ));
        }
        if cleanup_old_dirs.is_none() {
            cleanup_old_dirs = Some(tombstoned_old_dirs);
        }
    }

    cleanup_old_dirs.ok_or_else(|| "merge produced no replacement parts".to_string())
}


