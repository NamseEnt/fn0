#[allow(clippy::too_many_arguments)]
pub async fn merge_loop(
    registry: Arc<PartRegistry>,
    remote_cache: Option<Arc<RemoteCache>>,
    config: Arc<Config>,
    tenant_policy: Arc<TenantPolicy>,
    delete_requests: Arc<crate::delete_requests::DeleteRequests>,
    healthy: Arc<AtomicBool>,
    metrics: Arc<RuntimeMetrics>,
    mut drain_rx: watch::Receiver<bool>,
) {
    healthy.store(true, Ordering::Release);
    let mut ticker = interval(config.merge_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = wait_for_drain(&mut drain_rx) => return,
        }
        // Taken before the tick, and the status resolved after it, so a
        // request accepted mid-tick is applied by the next one rather than
        // being reported as processed by a rewrite that never saw it.
        let masks = delete_requests.masks();
        match merge_once(
            &registry,
            remote_cache.as_deref(),
            &config,
            &tenant_policy,
            &masks,
            Some(&drain_rx),
            &metrics,
            crate::clock::Clock::system().now_ns(),
        )
        .await
        {
            Ok(()) => {
                if !masks.is_empty() {
                    let metas: Vec<crate::part::PartMeta> = registry
                        .snapshot()
                        .into_iter()
                        .map(|reader| reader.meta().clone())
                        .collect();
                    delete_requests.mark_processed(&metas);
                }
                metrics.merge_success.fetch_add(1, Ordering::Relaxed);
                healthy.store(true, Ordering::Release)
            }
            Err(e) => {
                metrics.merge_errors.fetch_add(1, Ordering::Relaxed);
                healthy.store(false, Ordering::Release);
                tracing::error!(error = %e, "merge iteration failed");
            }
        }
    }
}

/// Merge with per-tenant retention switched off, which is what every test
/// that is not specifically about retention wants.
#[cfg(test)]
async fn merge_once_without_retention(
    registry: &PartRegistry,
    remote_cache: Option<&RemoteCache>,
    config: &Config,
    now_ns: i64,
) -> Result<(), String> {
    merge_once(
        registry,
        remote_cache,
        config,
        &TenantPolicy::disabled(),
        &crate::delete_requests::DeleteMasks::default(),
        None,
        &RuntimeMetrics::new(),
        now_ns,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn merge_once(
    registry: &PartRegistry,
    remote_cache: Option<&RemoteCache>,
    config: &Config,
    tenant_policy: &TenantPolicy,
    deletes: &crate::delete_requests::DeleteMasks,
    drain: Option<&watch::Receiver<bool>>,
    metrics: &RuntimeMetrics,
    now_ns: i64,
) -> Result<(), String> {
    let readers = registry.snapshot();
    if readers.is_empty() {
        return Ok(());
    }

    // Retention never writes a part itself. When a part carries expired rows
    // but no ordinary merge would pick it up, it becomes a valid group of one
    // here, and merge's existing transaction, tombstone and manifest CAS do
    // the replacement — one commit path, one crash-safety story.
    let cutoffs = tenant_policy.cutoffs_now();

    // Published here rather than on scrape: this tick already has the snapshot
    // and the cutoffs, and it runs whether or not the previous one succeeded,
    // so a stalled merge still shows its backlog growing.
    metrics.merge_debt_parts.store(
        merge_debt_part_count(registry, config, cutoffs.as_ref(), deletes, now_ns) as u64,
        Ordering::Relaxed,
    );

    let mut by_partition: HashMap<String, Vec<Arc<PartReader>>> = HashMap::new();
    for r in readers {
        by_partition
            .entry(r.meta().partition.clone())
            .or_default()
            .push(r);
    }

    let parts_root = config.data_dir.join("parts");
    let mut errors = Vec::new();
    let mut groups_processed = 0usize;

    'partitions: for (partition, mut parts) in by_partition {
        parts.sort_by_key(|r| r.meta().row_count);

        // Exclude an oversized single part (it is already large enough).
        // Group small parts for merging. Simplification: within a partition, group from the smallest
        // parts until merge_target_part_rows is reached once there are at least merge_min_part_count.
        let groups = select_groups(&parts, config, cutoffs.as_ref(), deletes, now_ns);
        for MergeGroup {
            parts: group,
            retention_only,
        } in groups
        {
            if groups_processed >= config.merge_max_groups_per_tick {
                break 'partitions;
            }
            // A tick may hold many groups and each one can take a minute. The
            // loop's own select only sees the drain between ticks, so a
            // shutdown that arrived mid-tick used to wait out every remaining
            // group: measured at 118 seconds from SIGTERM to exit, of which
            // 117 were merges started *after* the signal. The group already
            // running still finishes — abandoning it would throw away the work
            // and leave the inputs to be merged again — but no new one starts.
            if drain.is_some_and(|receiver| *receiver.borrow()) {
                tracing::info!(groups_processed, "merge stopping early to drain");
                break 'partitions;
            }
            groups_processed += 1;
            let old_ids: Vec<String> = group.iter().map(|r| r.meta().id.clone()).collect();
            let old_dirs: Vec<std::path::PathBuf> =
                group.iter().map(|r| r.part().dir.clone()).collect();

            // Keep input directories alive while the potentially expensive
            // read/write preparation runs. The deletion lock, not the
            // operation lock: the rewrite runs for as long as the group takes,
            // and holding the fair operation lock's read half for that long
            // meant one queued flush turned the whole rewrite into query tail
            // latency. The exclusive operation lock is acquired only for the
            // registry replacement below.
            let part_guard = registry.deletion_lock().read_owned().await;
            if let Some(cache) = remote_cache {
                let required: std::collections::HashSet<String> = old_ids.iter().cloned().collect();
                let missing = registry.missing_data_ids(&required);
                if !missing.is_empty() {
                    let restore = tokio::time::timeout(
                        config.max_restore_runtime,
                        cache.storage.restore_parts(&cache.parts_root, &missing),
                    )
                    .await;
                    match restore {
                        Ok(Ok(())) => cache.record_remote_success(),
                        Ok(Err(error)) => {
                            cache.record_remote_failure();
                            return Err(error);
                        }
                        Err(_) => {
                            cache.record_remote_failure();
                            return Err("object store restore timed out".to_string());
                        }
                    }
                }
            }

            if retention_only
                && let Some(cutoffs) = &cutoffs
                && group
                    .iter()
                    .all(|reader| cutoffs.expired_log_rows(reader.meta()) == 0)
            {
                // The group only existed to reclaim expired rows and another
                // tick already reclaimed them. Checked against `meta.json`
                // before reading anything, so the wasted work is a map lookup
                // rather than a whole rewrite.
                continue;
            }

            // Reading, filtering and writing happen together in one blocking
            // task: an oversized group is processed in pieces, and staging the
            // pieces would cost the memory the split exists to save.
            let merge_result = tokio::task::spawn_blocking({
                let group = group.clone();
                let parts_root = parts_root.clone();
                let old_dirs = old_dirs.clone();
                let cutoffs = cutoffs.clone();
                let deletes = deletes.clone();
                let row_group_size = config.row_group_size;
                let merge_max_memory_bytes = config.merge_max_memory_bytes;
                move || {
                    let _arena = crate::memprof::enter(crate::memprof::Arena::Merge);
                    rewrite_group(
                        &group,
                        cutoffs.as_ref(),
                        &deletes,
                        &parts_root,
                        row_group_size,
                        merge_max_memory_bytes,
                        &old_dirs,
                    )
                }
            })
            .await
            .map_err(|e| format!("merge write task join failed: {}", e))?;
            drop(part_guard);

            let rewrite = match merge_result {
                Ok(rewrite) => rewrite,
                Err(error) => {
                    // Retention deletes whole parts between this tick's group
                    // selection and the rewrite's reads, and a group whose
                    // input vanished mid-rewrite is not an incident: the next
                    // tick re-lists and never selects it again. The soak hit
                    // this four times in twenty minutes at a 5 m retention and
                    // each one wore an ERROR (todo.md). Distinguished from
                    // real I/O trouble by looking, not by matching the error
                    // text — and by the file only retention removes: cache
                    // eviction legitimately reclaims `data.parquet`, so a
                    // missing body alone must stay loud, while a missing
                    // `index.bin` means the whole part was deleted.
                    let input_vanished = group
                        .iter()
                        .any(|reader| !reader.part().index_path().exists());
                    if input_vanished {
                        tracing::debug!(
                            partition = %partition,
                            "merge input deleted mid-rewrite (retention); the next tick re-lists"
                        );
                        continue;
                    }
                    tracing::warn!(%error, partition = %partition, "merge rewrite failed, skipping group");
                    if retention_only {
                        // A rewrite streams now, so it cannot fail for want
                        // of memory at any part size; reaching here means I/O
                        // or a corrupt input. Counted rather than reported:
                        // pinning merge_healthy low would hide every other
                        // merge failure behind one group.
                        metrics
                            .retention_rewrite_skipped
                            .fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    errors.push(format!("merge rewrite failed for partition {partition}: {error}"));
                    continue;
                }
            };
            let dropped_rows = rewrite.dropped_rows;
            if rewrite.kept_rows == 0 {
                // Every row expired. Leave the inputs to retention's free
                // whole-part deletion rather than committing an empty merge.
                tracing::debug!(partition = %partition, "merge group is entirely expired; leaving it to retention");
                continue;
            }
            if dropped_rows == 0 && retention_only {
                // The meta-based pre-check said there was work; the rows say
                // otherwise. Discard the copy rather than commit a part onto
                // itself.
                let written: Vec<std::path::PathBuf> =
                    rewrite.new_parts.iter().map(|new| new.dir.clone()).collect();
                if let Err(cleanup_error) = part::remove_part_dirs(&written) {
                    tracing::warn!(%cleanup_error, "failed to remove a redundant retention rewrite");
                }
                continue;
            }

            let new_parts = rewrite.new_parts;
            let new_n = new_parts.len();
            let new_part_dirs: Vec<std::path::PathBuf> =
                new_parts.iter().map(|p| p.dir.clone()).collect();
            let cleanup_old_dirs = match verify_merge_tombstones(
                &new_part_dirs,
                &parts_root,
                &old_dirs,
            ) {
                Ok(tombstoned_old_dirs) => tombstoned_old_dirs,
                Err(error) => {
                    // The tombstone is the durable description of the
                    // replacement transaction. Never publish a merged
                    // part or delete its inputs unless that description
                    // can be read back and matches the intended inputs.
                    tracing::error!(
                        error = %error,
                        partition = %partition,
                        "merged part tombstone verification failed; keeping old parts"
                    );
                    if let Err(cleanup_error) = part::remove_part_dirs(&new_part_dirs) {
                        tracing::warn!(
                            error = %cleanup_error,
                            "failed to remove merged parts with invalid tombstones"
                        );
                    }
                    errors.push(format!(
                        "merged part tombstone verification failed for partition {partition}: {error}"
                    ));
                    continue;
                }
            };
            let part_guard = registry.deletion_lock().read_owned().await;
            let active_ids = registry.part_ids();
            if old_ids.iter().any(|id| !active_ids.contains(id)) {
                drop(part_guard);
                if let Err(cleanup_error) = part::remove_part_dirs(&new_part_dirs) {
                    tracing::warn!(%cleanup_error, "failed to clean merge output after input replacement");
                }
                tracing::debug!(partition = %partition, "merge inputs changed while preparing output");
                continue;
            }
            let mut manifest_committed = false;
            if let Some(cache) = remote_cache {
                if let Err(error) = cache.storage.publish(&new_parts, &old_ids).await {
                    if let Err(cleanup_error) = part::remove_part_dirs(&new_part_dirs) {
                        tracing::warn!(%cleanup_error, "failed to remove unpublished merged parts");
                    }
                    // Inputs replaced under us is the same benign
                    // situation the pre-publish check catches, only
                    // noticed later. The store answered correctly and
                    // nothing was written, so neither it nor merge is
                    // unhealthy: the next tick sees the new state.
                    if crate::object_storage::is_inputs_changed_error(&error) {
                        metrics.merge_inputs_changed.fetch_add(1, Ordering::Relaxed);
                        tracing::debug!(
                            partition = %partition,
                            "merge inputs were replaced while publishing"
                        );
                        continue;
                    }
                    cache.record_remote_failure();
                    tracing::error!(
                        %error,
                        partition = %partition,
                        "merged parts could not be published; keeping old parts"
                    );
                    errors.push(format!(
                        "object-store merge publish failed for partition {partition}: {error}"
                    ));
                    continue;
                }
                cache.record_remote_success();
                manifest_committed = true;
            }
            // Open the replacements while still under the deletion guard —
            // eviction could otherwise remove an unregistered directory —
            // and before the exclusive operation lock: opening validates
            // checksums over the whole merged part, and paying that under
            // the fair lock made every queued query wait for it.
            let opened_new = match PartRegistry::open_parts(new_parts) {
                Ok(opened) => opened,
                Err(e) => {
                    drop(part_guard);
                    // The tombstone is part of each new directory, so remove
                    // those directories as one failed transaction. Old
                    // registry entries and old data remain intact.
                    tracing::error!(
                        error = %e,
                        partition = %partition,
                        "merged part validation failed; keeping old parts"
                    );
                    if remote_cache.is_none()
                        && let Err(cleanup_error) = part::remove_part_dirs(&new_part_dirs)
                    {
                        tracing::warn!(
                            error = %cleanup_error,
                            "failed to remove invalid merged parts"
                        );
                    }
                    errors.push(format!(
                        "merged part validation failed for partition {partition}: {e}"
                    ));
                    continue;
                }
            };
            drop(part_guard);
            let _visibility_guard =
                crate::part_registry::PartRegistry::write_without_convoy(registry.operation_lock())
                    .await;
            // Only re-check while the replacement is still abandonable.
            // A successful publish is the commit point: the manifest
            // already lists the output and no longer lists the inputs,
            // and it only accepted the CAS because every input was
            // still present, so no other writer replaced them. Once
            // past it, discarding the output would leave the registry
            // serving inputs the manifest has dropped — whose objects
            // the orphan collector deletes after the grace period.
            // `replace` ignores ids that are already gone, so
            // retention retiring an input concurrently is harmless.
            if !manifest_committed {
                let active_ids = registry.part_ids();
                if old_ids.iter().any(|id| !active_ids.contains(id)) {
                    if let Err(cleanup_error) = part::remove_part_dirs(&new_part_dirs) {
                        tracing::warn!(%cleanup_error, "failed to clean merge output after final input replacement");
                    }
                    continue;
                }
            }
            registry.replace_opened(&old_ids, opened_new);
            if let Err(error) = part::remove_part_dirs(&cleanup_old_dirs) {
                tracing::warn!(
                    error = %error,
                    "old part cleanup incomplete; retaining merge tombstones"
                );
            } else {
                for new_dir in &new_part_dirs {
                    if let Err(e) = part::remove_merge_tombstone(new_dir) {
                        tracing::warn!(
                            error = %e,
                            ?new_dir,
                            "failed to remove merge tombstone (will be cleaned on next discover)"
                        );
                    }
                }
            }
            if dropped_rows > 0 {
                metrics
                    .retention_expired_rows_dropped
                    .fetch_add(dropped_rows as u64, Ordering::Relaxed);
            }
            // Only a group that nothing but retention would
            // have selected is a rewrite retention caused. A
            // size-driven merge that happened to drop expired
            // rows is already counted by the row counter, and
            // counting it here would hide how much extra I/O
            // retention is actually paying for.
            if retention_only {
                metrics
                    .retention_parts_rewritten
                    .fetch_add(old_ids.len() as u64, Ordering::Relaxed);
            }
            tracing::info!(
                partition = %partition,
                merged = old_ids.len(),
                produced = new_n,
                dropped_rows,
                "merge completed"
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
