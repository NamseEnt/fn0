impl ObjectStorage {
    async fn upload_part(&self, part: &Part) -> Result<(), String> {
        let descriptor = ManifestPart::from(part);
        for file in PART_FILES {
            let local_path = part.dir.join(file);
            let metadata = std::fs::symlink_metadata(&local_path).map_err(|error| {
                format!(
                    "failed to inspect part file {}: {error}",
                    local_path.display()
                )
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(format!(
                    "refusing unsafe part file {}",
                    local_path.display()
                ));
            }
            self.upload_object(
                &local_path,
                metadata.len(),
                &self.part_path(&descriptor, file),
                "part",
                &format!("part {} file {file}", part.meta.id),
            )
            .await?;
        }
        Ok(())
    }

    /// Repairs the manifest and local catalog after crashes, including a
    /// local-only to object-store transition. An initially empty manifest is
    /// seeded with the final locally recovered generation in one CAS. Once a
    /// remote generation exists, interrupted merge tombstones are replayed
    /// oldest-first before local cleanup.
    pub async fn reconcile_local_cache(&self, parts_root: &Path) -> Result<Manifest, String> {
        let initial = self.restore_catalog(parts_root).await?;
        validate_cache_tree_no_symlinks(parts_root)?;
        let migrating_from_local_only = initial.generation == 0 && initial.parts.is_empty();

        let tombstoned_active_ids: HashSet<String> = initial
            .parts
            .iter()
            .filter(|descriptor| {
                parts_root
                    .join(&descriptor.partition)
                    .join(&descriptor.id)
                    .join(part::MERGE_TOMBSTONE_FILE)
                    .exists()
            })
            .map(|descriptor| descriptor.id.clone())
            .collect();
        if !tombstoned_active_ids.is_empty() {
            self.restore_parts(parts_root, &tombstoned_active_ids)
                .await?;
        }

        let groups = collect_local_merge_groups(parts_root)?;
        let valid_outputs: HashSet<&str> = groups
            .iter()
            .flat_map(|group| group.added.iter().map(|part| part.meta.id.as_str()))
            .collect();
        for active_id in &tombstoned_active_ids {
            if !valid_outputs.contains(active_id.as_str()) {
                return Err(format!(
                    "active manifest part {active_id} has an invalid local merge tombstone replacement"
                ));
            }
        }

        let produced_ids: HashSet<&str> = groups
            .iter()
            .flat_map(|group| group.added.iter().map(|part| part.meta.id.as_str()))
            .collect();
        let merge_order = topological_merge_order(&groups)?;
        if migrating_from_local_only {
            // Reject overlapping roots before local cleanup: they describe
            // competing histories whose rows cannot be combined safely.
            let mut root_inputs = HashSet::new();
            for group in &groups {
                let is_root = group
                    .old_ids
                    .iter()
                    .all(|old_id| !produced_ids.contains(old_id.as_str()));
                if !is_root {
                    continue;
                }
                for old_id in &group.old_ids {
                    if !root_inputs.insert(old_id.as_str()) {
                        return Err(format!(
                            "overlapping local-only merge roots contain input part {old_id}"
                        ));
                    }
                }
            }

            // Resolve every local tombstone first, then publish the complete
            // final frontier in one CAS. This handles unbalanced trees such as
            // A+B -> M followed by M+C -> N without needing C to appear in an
            // intermediate remote generation. A crash before the CAS leaves
            // durable local outputs to retry; a crash after it leaves the
            // whole active generation visible.
            let local_parts = part::discover_parts(parts_root)?;
            self.publish_local_only_parts(&local_parts, &initial)
                .await?;
            return self.restore_catalog(parts_root).await;
        }

        // Re-read only after this loop has changed the manifest itself. It was
        // fetched once per merge group, which on a large manifest is a
        // multi-megabyte GET per group to observe a document only this loop
        // writes — the writer epoch already excludes anyone else.
        let mut manifest = self.load_manifest().await?;
        let mut manifest_is_stale = false;
        for index in merge_order {
            if manifest_is_stale {
                manifest = self.load_manifest().await?;
                manifest_is_stale = false;
            }
            let active: HashSet<&str> =
                manifest.parts.iter().map(|part| part.id.as_str()).collect();
            let active_inputs = groups[index]
                .old_ids
                .iter()
                .filter(|id| active.contains(id.as_str()))
                .count();
            let outputs_active = groups[index].added.iter().all(|part| {
                let descriptor = ManifestPart::from(part);
                manifest.parts.iter().any(|active| active == &descriptor)
            });

            if outputs_active && active_inputs == 0 {
                // The manifest CAS completed before the previous process
                // stopped. The local tombstone is only pending cleanup.
                continue;
            }
            if active_inputs == groups[index].old_ids.len() {
                self.publish(&groups[index].added, &groups[index].old_ids)
                    .await?;
                manifest_is_stale = true;
                continue;
            }
            if merge_group_reaches_active_output(index, &groups, &active) {
                // A newer local merge in the same tombstone chain is already
                // active. Replaying this ancestor would temporarily resurrect
                // rows that the active descendant replaced.
                continue;
            }
            return Err(format!(
                "local merge replacement conflict: expected {} active input parts, found {active_inputs}",
                groups[index].old_ids.len()
            ));
        }

        // The manifest now durably describes every valid merge transaction,
        // so it is safe for the existing tombstone recovery to remove old
        // local generations and their markers.
        let local_parts = part::discover_parts(parts_root)?;
        let manifest = self.load_manifest().await?;
        let published = self
            .publish_local_only_parts(&local_parts, &manifest)
            .await?;
        // Only restore again if the manifest changed. The second pass exists to
        // fetch catalog files for parts this reconcile just published, and on a
        // clean restart it publishes nothing — so it was re-validating every
        // part's checksums a second time for no result. At 10,000 parts that
        // pass alone cost about eighteen seconds of a sixty-four second
        // startup.
        if published.generation == manifest.generation {
            return Ok(published);
        }
        self.restore_catalog(parts_root).await
    }

    /// Restores only the small metadata/index catalog needed to plan queries.
    /// Parquet bodies remain remote until a query or merge selects them.
    pub async fn restore_catalog(&self, parts_root: &Path) -> Result<Manifest, String> {
        let manifest = self.load_manifest().await?;
        tokio::fs::create_dir_all(parts_root)
            .await
            .map_err(|error| error.to_string())?;
        validate_cache_root(parts_root)?;
        // Deciding what is missing is local work and stays sequential; the
        // downloads are what the round trips are spent on.
        let mut missing = Vec::new();
        for descriptor in &manifest.parts {
            let final_dir = cache_part_dir(parts_root, descriptor)?;
            self.record_catalog_validation();
            if open_manifest_part(&final_dir, descriptor, false).is_ok() {
                continue;
            }
            missing.push(descriptor);
        }
        self.download_parts(missing, parts_root, false).await?;
        Ok(manifest)
    }

    /// Download a set of parts with bounded concurrency.
    ///
    /// The first error wins and the rest are abandoned. A partial restore is
    /// already the state the caller has to handle — it is what a crash mid-way
    /// leaves — so finishing the remaining downloads before reporting failure
    /// would spend round trips to reach the same outcome.
    async fn download_parts(
        &self,
        descriptors: Vec<&ManifestPart>,
        parts_root: &Path,
        include_data: bool,
    ) -> Result<(), String> {
        // Chunked rather than a sliding window: the difference is a partly
        // idle tail per chunk, and in exchange the concurrency is obvious from
        // reading it and the borrow of `self` stays a plain one.
        for chunk in descriptors.chunks(RESTORE_CONCURRENCY) {
            let downloads = chunk
                .iter()
                .map(|descriptor| self.download_part(descriptor, parts_root, include_data));
            futures_util::future::try_join_all(downloads).await?;
        }
        Ok(())
    }

    /// Restores the Parquet bodies for a selected set of parts. The caller
    /// must hold the cache's exclusive operation lock.
    pub async fn restore_parts(
        &self,
        parts_root: &Path,
        ids: &HashSet<String>,
    ) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let manifest = self.load_manifest().await?;
        let mut restored = HashSet::new();
        let mut missing = Vec::new();
        for descriptor in &manifest.parts {
            if !ids.contains(&descriptor.id) {
                continue;
            }
            let final_dir = cache_part_dir(parts_root, descriptor)?;
            if open_manifest_part(&final_dir, descriptor, true).is_err() {
                missing.push(descriptor);
            }
            restored.insert(descriptor.id.clone());
        }
        self.download_parts(missing, parts_root, true).await?;
        let missing: Vec<_> = ids.difference(&restored).cloned().collect();
        if !missing.is_empty() {
            return Err(format!(
                "parts are no longer present in the object-store manifest: {}",
                missing.join(", ")
            ));
        }
        Ok(())
    }

    async fn download_part(
        &self,
        descriptor: &ManifestPart,
        parts_root: &Path,
        include_data: bool,
    ) -> Result<(), String> {
        let final_dir = cache_part_dir(parts_root, descriptor)?;
        // Eviction takes the Parquet body and leaves the catalog beside it, so a
        // body-only restore usually finds `index.bin` and `meta.json` already
        // local and already checksummed against this descriptor. Rebuilding the
        // whole directory would spend two more GETs to arrive at bytes that are
        // on disk, and would delete a healthy catalog in order to do it.
        if include_data && open_manifest_part(&final_dir, descriptor, false).is_ok() {
            return self
                .download_part_body(descriptor, parts_root, &final_dir)
                .await;
        }
        let local_tombstone = std::fs::read(final_dir.join(part::MERGE_TOMBSTONE_FILE)).ok();
        let staging = StagingDir::create(parts_root, &descriptor.partition, &descriptor.id)?;
        let tmp_dir = staging.path();
        let files: &[&str] = if include_data {
            &PART_FILES
        } else {
            &CATALOG_FILES
        };
        for &file in files {
            self.fetch_part_file(descriptor, file, tmp_dir).await?;
        }
        let downloaded =
            open_manifest_part(tmp_dir, descriptor, include_data).map_err(|error| {
                format!(
                    "downloaded part {} failed validation: {error}",
                    descriptor.id
                )
            })?;
        drop(downloaded);

        if let Some(tombstone) = local_tombstone {
            let marker = tmp_dir.join(part::MERGE_TOMBSTONE_FILE);
            std::fs::write(&marker, tombstone).map_err(|error| error.to_string())?;
            std::fs::File::open(&marker)
                .and_then(|file| file.sync_all())
                .map_err(|error| error.to_string())?;
            part::fsync_dir(tmp_dir).map_err(|error| error.to_string())?;
        }

        match std::fs::remove_dir_all(&final_dir) {
            Ok(()) => {}
            // Another restore of the same part reached the commit first. Gone is
            // the state this line wanted, so it is not an error to find it that
            // way — and a part is immutable, so the copy that beat us here holds
            // the same bytes ours does.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
        // The partition directory, not the part's own: a part directory that
        // exists before its files do is exactly what this commit replaces.
        if let Some(parent) = final_dir.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::rename(tmp_dir, &final_dir)
            .map_err(|error| format!("failed to commit cached part {}: {error}", descriptor.id))?;
        // A catalog-only download restores no body, so it buys no later scan
        // and is not what the reuse figure is about.
        if include_data {
            crate::restore_meter::global().note_restore(&final_dir);
        }
        Ok(())
    }

    /// Restores only the Parquet body into a part directory whose catalog is
    /// already local and already valid for this descriptor.
    ///
    /// The body lands by renaming one file into place rather than by replacing
    /// the directory, so the catalog and any merge tombstone beside it are never
    /// deleted and never rewritten. A body that fails validation is removed
    /// again: every reader checksums the body it opens, so nothing trusts the
    /// file in the window before this check, and removing it keeps the retry on
    /// the one-GET path instead of demoting it to the three-GET one.
    async fn download_part_body(
        &self,
        descriptor: &ManifestPart,
        parts_root: &Path,
        final_dir: &Path,
    ) -> Result<(), String> {
        let staging = StagingDir::create(parts_root, &descriptor.partition, &descriptor.id)?;
        self.fetch_part_file(descriptor, DATA_FILE, staging.path())
            .await?;
        let data_path = final_dir.join(DATA_FILE);
        std::fs::rename(staging.path().join(DATA_FILE), &data_path)
            .map_err(|error| format!("failed to commit cached part {}: {error}", descriptor.id))?;
        match open_manifest_part(final_dir, descriptor, true) {
            Ok(downloaded) => {
                drop(downloaded);
                Ok(())
            }
            Err(error) => {
                let _ = std::fs::remove_file(&data_path);
                Err(format!(
                    "downloaded part {} failed validation: {error}",
                    descriptor.id
                ))
            }
        }
    }

    async fn fetch_part_file(
        &self,
        descriptor: &ManifestPart,
        file: &str,
        dest_dir: &Path,
    ) -> Result<(), String> {
        let result = self
            .store
            .get(&self.part_path(descriptor, file))
            .await
            .map_err(|error| {
                format!(
                    "failed to download part {} file {file}: {error}",
                    descriptor.id
                )
            })?;
        let bytes = result.bytes().await.map_err(|error| {
            format!("failed to read part {} file {file}: {error}", descriptor.id)
        })?;
        tokio::fs::write(dest_dir.join(file), bytes)
            .await
            .map_err(|error| {
                format!(
                    "failed to cache part {} file {file}: {error}",
                    descriptor.id
                )
            })
    }

    /// Evict Parquet bodies until the cache fits `max_bytes`, oldest first.
    ///
    /// Driven by the part directories the registry holds rather than by walking
    /// the tree. The walk was two levels of `read_dir` plus a
    /// `symlink_metadata` per entry, over every directory on disk, to arrive at
    /// exactly the set the registry already knows — everything not in it was
    /// skipped anyway, because absence from the registry does not prove a
    /// directory is disposable. It may be local-only data from a period when
    /// object storage was disabled, or an interrupted transaction awaiting
    /// operator recovery. So the walk cost scaled with the disk while the
    /// result scaled with the registry.
    ///
    /// The safety checks stay per directory: a path taken from the registry is
    /// still checked for symlinks and for containment in the cache root, since
    /// what those guard against is a cache tree that has been tampered with,
    /// not a caller that passed the wrong list.
    pub fn evict_cache(
        &self,
        parts_root: &Path,
        max_bytes: u64,
        part_dirs: &[PathBuf],
    ) -> Result<u64, String> {
        self.evict_bodies(parts_root, max_bytes, part_dirs, DATA_FILE, "cache")
    }

    /// `domain` only names the cache in error messages. An operator reading
    /// "refusing symlinked trace cache file" knows which tree to look at;
    /// collapsing both into one wording would take that away.
    fn evict_bodies(
        &self,
        cache_root: &Path,
        max_bytes: u64,
        part_dirs: &[PathBuf],
        data_file: &str,
        domain: &str,
    ) -> Result<u64, String> {
        let canonical_root = match std::fs::symlink_metadata(cache_root) {
            Ok(_) => validate_cache_root(cache_root)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(error) => return Err(error.to_string()),
        };

        let mut cached = Vec::new();
        let mut total = 0u64;
        for dir in part_dirs {
            match std::fs::symlink_metadata(dir) {
                Ok(_) => {}
                // A registered part whose directory is gone is the eviction's
                // own past work, or a restore that has not run yet. Neither is
                // an error here.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.to_string()),
            }
            ensure_existing_cache_dir(&canonical_root, dir)?;

            // The bound applies to evictable Parquet bodies. Metadata, bloom
            // filters, and stream indexes form the small persistent catalog
            // required to plan selective restores.
            let data_path = dir.join(data_file);
            let bytes = match std::fs::symlink_metadata(&data_path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(format!(
                        "refusing symlinked {domain} file {}",
                        data_path.display()
                    ));
                }
                Ok(metadata) if metadata.is_file() => metadata.len(),
                Ok(_) => {
                    return Err(format!(
                        "{domain} data path is not a file: {}",
                        data_path.display()
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
                Err(error) => return Err(error.to_string()),
            };
            let access_path = dir.join(".access");
            let accessed = match std::fs::symlink_metadata(&access_path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(format!(
                        "refusing symlinked {domain} access marker {}",
                        access_path.display()
                    ));
                }
                Ok(metadata) => metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    std::fs::symlink_metadata(dir)
                        .map_err(|error| error.to_string())?
                        .modified()
                        .unwrap_or(std::time::UNIX_EPOCH)
                }
                Err(error) => return Err(error.to_string()),
            };
            total = total.saturating_add(bytes);
            if bytes > 0 {
                cached.push((accessed, bytes, dir.clone()));
            }
        }

        cached.sort_by_key(|(accessed, _, _)| *accessed);
        for (_, bytes, dir) in cached {
            if total <= max_bytes {
                break;
            }
            let data_path = dir.join(data_file);
            match std::fs::remove_file(&data_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.to_string()),
            }
            crate::restore_meter::global().note_evict(&dir);
            total = total.saturating_sub(bytes);
        }
        Ok(total)
    }

    /// The trace side of `evict_cache`, driven the same way and for the same
    /// reason.
    pub fn evict_trace_cache(
        &self,
        traces_root: &Path,
        max_bytes: u64,
        part_dirs: &[PathBuf],
    ) -> Result<u64, String> {
        self.evict_bodies(
            traces_root,
            max_bytes,
            part_dirs,
            TRACE_DATA_FILE,
            "trace cache",
        )
    }

    /// The metric side, same discipline: bodies leave, the catalog stays so
    /// selection keeps planning without a restore.
    pub fn evict_metric_cache(
        &self,
        metrics_root: &Path,
        max_bytes: u64,
        part_dirs: &[PathBuf],
    ) -> Result<u64, String> {
        self.evict_bodies(
            metrics_root,
            max_bytes,
            part_dirs,
            SERIES_DATA_FILE,
            "metric cache",
        )
    }
}
