const CATALOG_COMMITS_PREFIX: &str = "catalog/commits";
const CATALOG_SNAPSHOTS_PREFIX: &str = "catalog/snapshots";
const CATALOG_SNAPSHOT_INTERVAL: u64 = 256;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct CatalogState {
    generation: u64,
    writer_epoch: u64,
    last_digest: String,
    last_transaction_id: String,
    manifest: Manifest,
    trace_manifest: TraceManifest,
    metric_manifest: MetricManifest,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct CatalogMutation {
    transaction_id: String,
    writer_epoch: u64,
    claim_writer: bool,
    log_added: Vec<ManifestPart>,
    log_removed: Vec<String>,
    trace_added: Vec<TraceManifestPart>,
    trace_removed: Vec<String>,
    metric_added: Vec<MetricManifestPart>,
    metric_removed: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct CatalogCommit {
    generation: u64,
    parent_generation: u64,
    parent_digest: String,
    mutation: CatalogMutation,
    digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct CatalogSnapshot {
    generation: u64,
    through_digest: String,
    state: CatalogState,
    digest: String,
}

#[derive(Clone, Debug)]
struct CatalogReplica<T> {
    value: T,
    bytes: Vec<u8>,
}

fn catalog_generation_key(generation: u64) -> String {
    format!("{generation:020}.json")
}

fn catalog_generation_from_path(path: &ObjectPath) -> Option<u64> {
    path.as_ref()
        .rsplit('/')
        .next()?
        .strip_suffix(".json")?
        .parse()
        .ok()
}

fn catalog_digest<T: Serialize>(value: &T) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn seal_catalog_commit(mut commit: CatalogCommit) -> Result<CatalogCommit, String> {
    commit.digest.clear();
    commit.digest = catalog_digest(&commit)?;
    Ok(commit)
}

fn verify_catalog_commit(commit: &CatalogCommit) -> Result<(), String> {
    let expected = seal_catalog_commit(commit.clone())?.digest;
    if expected != commit.digest {
        return Err(format!(
            "catalog commit {} has digest {}, expected {}",
            commit.generation, commit.digest, expected
        ));
    }
    Ok(())
}

fn seal_catalog_snapshot(mut snapshot: CatalogSnapshot) -> Result<CatalogSnapshot, String> {
    snapshot.digest.clear();
    snapshot.digest = catalog_digest(&snapshot)?;
    Ok(snapshot)
}

fn verify_catalog_snapshot(snapshot: &CatalogSnapshot) -> Result<(), String> {
    if snapshot.state.generation != snapshot.generation {
        return Err(format!(
            "catalog snapshot generation {} contains state generation {}",
            snapshot.generation, snapshot.state.generation
        ));
    }
    if snapshot.state.last_digest != snapshot.through_digest {
        return Err(format!(
            "catalog snapshot {} has a mismatched through digest",
            snapshot.generation
        ));
    }
    let expected = seal_catalog_snapshot(snapshot.clone())?.digest;
    if expected != snapshot.digest {
        return Err(format!(
            "catalog snapshot {} has digest {}, expected {}",
            snapshot.generation, snapshot.digest, expected
        ));
    }
    Ok(())
}

impl ObjectStorage {
    pub async fn verify_catalog_listing(&self) -> Result<(), String> {
        use futures_util::StreamExt;

        let state = self.load_catalog_state().await?;
        for replica in ["a", "b"] {
            let prefix = self.path(&format!("{CATALOG_COMMITS_PREFIX}/{replica}"));
            let expected = self.catalog_commit_path(replica, state.generation);
            let mut stream = self.store.list(Some(&prefix));
            let mut found = false;
            while let Some(item) = stream.next().await {
                let meta = item.map_err(|error| {
                    format!("failed to list catalog replica {replica} during startup: {error}")
                })?;
                if meta.location == expected {
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(format!(
                    "catalog generation {} replica {replica} was not immediately visible in listing",
                    state.generation
                ));
            }
        }
        Ok(())
    }

    fn catalog_replica_path(&self, kind: &str, replica: &str, generation: u64) -> ObjectPath {
        self.path(&format!(
            "{kind}/{replica}/{}",
            catalog_generation_key(generation)
        ))
    }

    fn catalog_commit_path(&self, replica: &str, generation: u64) -> ObjectPath {
        self.catalog_replica_path(CATALOG_COMMITS_PREFIX, replica, generation)
    }

    fn catalog_snapshot_path(&self, replica: &str, generation: u64) -> ObjectPath {
        self.catalog_replica_path(CATALOG_SNAPSHOTS_PREFIX, replica, generation)
    }

    async fn catalog_get_bytes(&self, path: &ObjectPath) -> Result<Option<Vec<u8>>, String> {
        match self.store.get(path).await {
            Ok(result) => result
                .bytes()
                .await
                .map(|bytes| Some(bytes.to_vec()))
                .map_err(|error| format!("failed to read catalog object {path}: {error}")),
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(error) => Err(format!("failed to read catalog object {path}: {error}")),
        }
    }

    async fn catalog_put_create(&self, path: &ObjectPath, bytes: &[u8]) -> Result<bool, String> {
        match self
            .store
            .put_opts(
                path,
                bytes.to_vec().into(),
                PutOptions {
                    mode: PutMode::Create,
                    ..Default::default()
                },
            )
            .await
        {
            Ok(_) => Ok(true),
            Err(object_store::Error::AlreadyExists { .. })
            | Err(object_store::Error::Precondition { .. }) => Ok(false),
            Err(error) => Err(format!("failed to create catalog object {path}: {error}")),
        }
    }

    async fn catalog_generations(
        &self,
        kind: &str,
        offset: Option<u64>,
    ) -> Result<Vec<u64>, String> {
        use futures_util::StreamExt;

        let mut generations = std::collections::BTreeSet::new();
        for replica in ["a", "b"] {
            let prefix = self.path(&format!("{kind}/{replica}"));
            let offset_path =
                offset.map(|generation| self.catalog_replica_path(kind, replica, generation));
            let mut stream = match offset_path.as_ref() {
                Some(path) => self.store.list_with_offset(Some(&prefix), path),
                None => self.store.list(Some(&prefix)),
            };
            while let Some(item) = stream.next().await {
                let meta = item.map_err(|error| {
                    format!("failed to list catalog {kind} replica {replica}: {error}")
                })?;
                if let Some(generation) = catalog_generation_from_path(&meta.location) {
                    generations.insert(generation);
                }
            }
        }
        Ok(generations.into_iter().collect())
    }

    async fn load_catalog_commit_replica(
        &self,
        replica: &str,
        generation: u64,
    ) -> Result<Option<CatalogReplica<CatalogCommit>>, String> {
        let path = self.catalog_commit_path(replica, generation);
        let Some(bytes) = self.catalog_get_bytes(&path).await? else {
            return Ok(None);
        };
        let commit: CatalogCommit = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid catalog commit {path}: {error}"))?;
        verify_catalog_commit(&commit)?;
        if commit.generation != generation {
            return Err(format!(
                "catalog commit {path} contains generation {}",
                commit.generation
            ));
        }
        Ok(Some(CatalogReplica {
            value: commit,
            bytes,
        }))
    }

    async fn load_catalog_snapshot_replica(
        &self,
        replica: &str,
        generation: u64,
    ) -> Result<Option<CatalogReplica<CatalogSnapshot>>, String> {
        let path = self.catalog_snapshot_path(replica, generation);
        let Some(bytes) = self.catalog_get_bytes(&path).await? else {
            return Ok(None);
        };
        let snapshot: CatalogSnapshot = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid catalog snapshot {path}: {error}"))?;
        verify_catalog_snapshot(&snapshot)?;
        validate_manifest(&snapshot.state.manifest)?;
        validate_trace_manifest(&snapshot.state.trace_manifest)?;
        validate_metric_manifest(&snapshot.state.metric_manifest)?;
        if snapshot.generation != generation {
            return Err(format!(
                "catalog snapshot {path} contains generation {}",
                snapshot.generation
            ));
        }
        Ok(Some(CatalogReplica {
            value: snapshot,
            bytes,
        }))
    }

    async fn load_catalog_commit(&self, generation: u64) -> Result<CatalogCommit, String> {
        let first = self.load_catalog_commit_replica("a", generation).await;
        let second = self.load_catalog_commit_replica("b", generation).await;
        match (first, second) {
            (Ok(Some(first)), Ok(Some(second))) => {
                if first.value.digest != second.value.digest {
                    return Err(format!(
                        "catalog generation {generation} has divergent replicas"
                    ));
                }
                Ok(first.value)
            }
            (Ok(Some(first)), Ok(None)) => {
                let path = self.catalog_commit_path("b", generation);
                let _ = self.catalog_put_create(&path, &first.bytes).await?;
                Ok(first.value)
            }
            (Ok(None), Ok(Some(second))) => {
                let path = self.catalog_commit_path("a", generation);
                let _ = self.catalog_put_create(&path, &second.bytes).await?;
                Ok(second.value)
            }
            (Err(first_error), Ok(Some(second))) => {
                tracing::warn!(
                    generation,
                    %first_error,
                    "ignoring invalid catalog commit replica"
                );
                Ok(second.value)
            }
            (Ok(Some(first)), Err(second_error)) => {
                tracing::warn!(
                    generation,
                    %second_error,
                    "ignoring invalid catalog commit replica"
                );
                Ok(first.value)
            }
            (Err(first_error), Err(second_error)) => Err(format!(
                "both catalog replicas for generation {generation} are invalid: {first_error}; {second_error}"
            )),
            (Err(error), Ok(None)) | (Ok(None), Err(error)) => Err(format!(
                "catalog generation {generation} is invalid: {error}"
            )),
            (Ok(None), Ok(None)) => Err(format!("catalog generation {generation} is missing")),
        }
    }

    async fn load_catalog_snapshot(&self, generation: u64) -> Result<CatalogSnapshot, String> {
        let first = self.load_catalog_snapshot_replica("a", generation).await;
        let second = self.load_catalog_snapshot_replica("b", generation).await;
        match (first, second) {
            (Ok(Some(first)), Ok(Some(second))) => {
                if first.value.digest != second.value.digest {
                    return Err(format!(
                        "catalog snapshot {generation} has divergent replicas"
                    ));
                }
                Ok(first.value)
            }
            (Ok(Some(first)), Ok(None)) => {
                let path = self.catalog_snapshot_path("b", generation);
                let _ = self.catalog_put_create(&path, &first.bytes).await?;
                Ok(first.value)
            }
            (Ok(None), Ok(Some(second))) => {
                let path = self.catalog_snapshot_path("a", generation);
                let _ = self.catalog_put_create(&path, &second.bytes).await?;
                Ok(second.value)
            }
            (Err(first_error), Ok(Some(second))) => {
                tracing::warn!(
                    generation,
                    %first_error,
                    "ignoring invalid catalog snapshot replica"
                );
                Ok(second.value)
            }
            (Ok(Some(first)), Err(second_error)) => {
                tracing::warn!(
                    generation,
                    %second_error,
                    "ignoring invalid catalog snapshot replica"
                );
                Ok(first.value)
            }
            (Err(first_error), Err(second_error)) => Err(format!(
                "both catalog snapshot replicas for generation {generation} are invalid: {first_error}; {second_error}"
            )),
            (Err(error), Ok(None)) | (Ok(None), Err(error)) => Err(format!(
                "catalog snapshot generation {generation} is invalid: {error}"
            )),
            (Ok(None), Ok(None)) => Err(format!(
                "catalog snapshot generation {generation} is missing"
            )),
        }
    }

    fn apply_log_parts(
        parts: &mut Vec<ManifestPart>,
        added: &[ManifestPart],
        removed: &[String],
    ) -> Result<(), String> {
        let removed: HashSet<&str> = removed.iter().map(String::as_str).collect();
        parts.retain(|part| !removed.contains(part.id.as_str()));
        for part in added {
            if let Some(existing) = parts.iter().find(|item| item.id == part.id) {
                if existing != part {
                    return Err(format!("manifest part ID collision: {}", part.id));
                }
            } else {
                parts.push(part.clone());
            }
        }
        parts
            .sort_by(|left, right| (&left.partition, &left.id).cmp(&(&right.partition, &right.id)));
        Ok(())
    }

    fn apply_trace_parts(
        parts: &mut Vec<TraceManifestPart>,
        added: &[TraceManifestPart],
        removed: &[String],
    ) -> Result<(), String> {
        let removed: HashSet<&str> = removed.iter().map(String::as_str).collect();
        parts.retain(|part| !removed.contains(part.id.as_str()));
        for part in added {
            if let Some(existing) = parts.iter().find(|item| item.id == part.id) {
                if existing != part {
                    return Err(format!("trace manifest part ID collision: {}", part.id));
                }
            } else {
                parts.push(part.clone());
            }
        }
        parts
            .sort_by(|left, right| (&left.partition, &left.id).cmp(&(&right.partition, &right.id)));
        Ok(())
    }

    fn apply_metric_parts(
        parts: &mut Vec<MetricManifestPart>,
        added: &[MetricManifestPart],
        removed: &[String],
    ) -> Result<(), String> {
        let removed: HashSet<&str> = removed.iter().map(String::as_str).collect();
        parts.retain(|part| !removed.contains(part.id.as_str()));
        for part in added {
            if let Some(existing) = parts.iter().find(|item| item.id == part.id) {
                if existing != part {
                    return Err(format!("metric manifest part ID collision: {}", part.id));
                }
            } else {
                parts.push(part.clone());
            }
        }
        parts
            .sort_by(|left, right| (&left.partition, &left.id).cmp(&(&right.partition, &right.id)));
        Ok(())
    }

    fn apply_catalog_mutation(
        state: &CatalogState,
        commit: &CatalogCommit,
    ) -> Result<CatalogState, String> {
        if commit.parent_generation != state.generation {
            return Err(format!(
                "catalog commit {} points at generation {}, current is {}",
                commit.generation, commit.parent_generation, state.generation
            ));
        }
        if commit.parent_digest != state.last_digest {
            return Err(format!(
                "catalog commit {} has a mismatched parent digest",
                commit.generation
            ));
        }
        let expected_generation = state
            .generation
            .checked_add(1)
            .ok_or_else(|| "catalog generation overflow".to_string())?;
        if commit.generation != expected_generation {
            return Err(format!(
                "catalog generation must advance from {} to {}, got {}",
                state.generation, expected_generation, commit.generation
            ));
        }
        if commit.mutation.claim_writer {
            let expected_epoch = state
                .writer_epoch
                .checked_add(1)
                .ok_or_else(|| "writer epoch overflow".to_string())?;
            if commit.mutation.writer_epoch != expected_epoch {
                return Err(format!(
                    "catalog writer epoch must advance from {} to {}, got {}",
                    state.writer_epoch, expected_epoch, commit.mutation.writer_epoch
                ));
            }
        } else if commit.mutation.writer_epoch != state.writer_epoch {
            return Err(format!(
                "catalog commit {} changes writer epoch without a claim",
                commit.generation
            ));
        }

        let mut next = state.clone();
        Self::apply_log_parts(
            &mut next.manifest.parts,
            &commit.mutation.log_added,
            &commit.mutation.log_removed,
        )?;
        Self::apply_trace_parts(
            &mut next.trace_manifest.parts,
            &commit.mutation.trace_added,
            &commit.mutation.trace_removed,
        )?;
        Self::apply_metric_parts(
            &mut next.metric_manifest.parts,
            &commit.mutation.metric_added,
            &commit.mutation.metric_removed,
        )?;
        next.generation = commit.generation;
        next.writer_epoch = commit.mutation.writer_epoch;
        next.last_digest = commit.digest.clone();
        next.last_transaction_id = commit.mutation.transaction_id.clone();
        next.manifest.generation = next.generation;
        next.manifest.writer_epoch = next.writer_epoch;
        next.trace_manifest.generation = next.generation;
        next.trace_manifest.writer_epoch = next.writer_epoch;
        next.metric_manifest.generation = next.generation;
        next.metric_manifest.writer_epoch = next.writer_epoch;
        validate_manifest(&next.manifest)?;
        validate_trace_manifest(&next.trace_manifest)?;
        validate_metric_manifest(&next.metric_manifest)?;
        Ok(next)
    }

    async fn load_latest_catalog_snapshot(&self) -> Result<Option<CatalogSnapshot>, String> {
        let generations = self
            .catalog_generations(CATALOG_SNAPSHOTS_PREFIX, None)
            .await?;
        for generation in generations.into_iter().rev() {
            match self.load_catalog_snapshot(generation).await {
                Ok(snapshot) => return Ok(Some(snapshot)),
                Err(error) => {
                    tracing::warn!(generation, %error, "ignoring invalid catalog snapshot");
                }
            }
        }
        Ok(None)
    }

    pub(crate) async fn load_catalog_state(&self) -> Result<CatalogState, String> {
        if let Some(state) = self.catalog_state.lock().await.clone() {
            return Ok(state);
        }
        let snapshot = self.load_latest_catalog_snapshot().await?;
        let mut state = snapshot.map(|snapshot| snapshot.state).unwrap_or_default();
        let generations = self
            .catalog_generations(CATALOG_COMMITS_PREFIX, Some(state.generation))
            .await?;
        for legacy_path in [
            self.manifest_path(),
            self.trace_manifest_path(),
            self.metric_manifest_path(),
        ] {
            if self.catalog_get_bytes(&legacy_path).await?.is_some() {
                return Err(format!(
                    "legacy mutable manifest {legacy_path} is present; catalog migration is required before startup"
                ));
            }
        }
        for generation in generations {
            let expected_generation = state
                .generation
                .checked_add(1)
                .ok_or_else(|| "catalog generation overflow".to_string())?;
            if generation != expected_generation {
                return Err(format!(
                    "catalog generation gap after {}: found {}",
                    state.generation, generation
                ));
            }
            let commit = self.load_catalog_commit(generation).await?;
            verify_catalog_commit(&commit)?;
            state = Self::apply_catalog_mutation(&state, &commit)?;
        }
        if state.generation > 0 {
            *self.catalog_state.lock().await = Some(state.clone());
        }
        Ok(state)
    }

    async fn write_catalog_snapshot(&self, state: &CatalogState) -> Result<(), String> {
        let snapshot = seal_catalog_snapshot(CatalogSnapshot {
            generation: state.generation,
            through_digest: state.last_digest.clone(),
            state: state.clone(),
            digest: String::new(),
        })?;
        let bytes = serde_json::to_vec(&snapshot).map_err(|error| error.to_string())?;
        for replica in ["a", "b"] {
            let path = self.catalog_snapshot_path(replica, state.generation);
            if self.catalog_put_create(&path, &bytes).await? {
                continue;
            }
            let existing = self
                .catalog_get_bytes(&path)
                .await?
                .ok_or_else(|| format!("catalog snapshot disappeared: {path}"))?;
            if existing != bytes {
                return Err(format!("catalog snapshot replica differs: {path}"));
            }
        }
        Ok(())
    }

    /// Delete catalog objects that no startup can read any more.
    ///
    /// A startup reads the newest snapshot that verifies and replays the
    /// commits after it, falling back to an older snapshot when the newest one
    /// does not verify. So two snapshots are kept that verify on both replicas
    /// and whose digest matches the commit at their generation, together with
    /// that commit and every commit after it; a single damaged snapshot then
    /// still leaves a readable catalog. Everything older goes, but only once it
    /// is older than `min_age`: the Bucket Lock rule on `catalog/` refuses to
    /// delete anything younger, and `min_age` has to be set above that rule.
    pub async fn prune_catalog(&self, min_age: std::time::Duration) -> Result<usize, String> {
        self.prune_catalog_at(min_age, chrono::Utc::now()).await
    }

    pub(crate) async fn prune_catalog_at(
        &self,
        min_age: std::time::Duration,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<usize, String> {
        use futures_util::StreamExt;

        let snapshot_generations = self
            .catalog_generations(CATALOG_SNAPSHOTS_PREFIX, None)
            .await?;
        let mut verified_snapshot_generations = Vec::new();
        for generation in snapshot_generations.into_iter().rev() {
            if self.catalog_snapshot_is_intact(generation).await? {
                verified_snapshot_generations.push(generation);
                if verified_snapshot_generations.len() == 2 {
                    break;
                }
            }
        }
        let [_, oldest_kept_snapshot] = verified_snapshot_generations[..] else {
            return Ok(0);
        };
        let cutoff = now
            - chrono::Duration::from_std(min_age)
                .map_err(|error| format!("invalid catalog prune age: {error}"))?;

        let mut removed = 0;
        for kind in [CATALOG_SNAPSHOTS_PREFIX, CATALOG_COMMITS_PREFIX] {
            for replica in ["a", "b"] {
                let prefix = self.path(&format!("{kind}/{replica}"));
                let mut superseded = Vec::new();
                let mut stream = self.store.list(Some(&prefix));
                while let Some(item) = stream.next().await {
                    let meta = item.map_err(|error| {
                        format!("failed to list catalog {kind} replica {replica}: {error}")
                    })?;
                    let Some(generation) = catalog_generation_from_path(&meta.location) else {
                        continue;
                    };
                    if generation < oldest_kept_snapshot && meta.last_modified < cutoff {
                        superseded.push(meta.location);
                    }
                }
                drop(stream);
                for location in superseded {
                    match self.store.delete(&location).await {
                        Ok(()) | Err(object_store::Error::NotFound { .. }) => removed += 1,
                        Err(error) => {
                            return Err(format!(
                                "failed to delete superseded catalog object {location}: {error}"
                            ));
                        }
                    }
                }
            }
        }
        Ok(removed)
    }

    /// Whether both replicas of a snapshot read back identical and valid, and
    /// the snapshot closes over the commit that is really at its generation.
    async fn catalog_snapshot_is_intact(&self, generation: u64) -> Result<bool, String> {
        let first = self.load_catalog_snapshot_replica("a", generation).await;
        let second = self.load_catalog_snapshot_replica("b", generation).await;
        let (Ok(Some(first)), Ok(Some(second))) = (first, second) else {
            return Ok(false);
        };
        if first.bytes != second.bytes {
            return Ok(false);
        }
        match self.load_catalog_commit(generation).await {
            Ok(commit) => Ok(commit.digest == first.value.through_digest),
            Err(_) => Ok(false),
        }
    }

    pub(crate) async fn commit_catalog_mutation(
        &self,
        mut mutation: CatalogMutation,
    ) -> Result<CatalogState, String> {
        if mutation.transaction_id.is_empty() {
            mutation.transaction_id = uuid::Uuid::new_v4().to_string();
        }
        let _guard = self.manifest_update.lock().await;
        for _ in 0..MAX_CAS_ATTEMPTS {
            let state = self.load_catalog_state().await?;
            self.check_epoch(state.writer_epoch)?;
            if state.last_transaction_id == mutation.transaction_id {
                return Ok(state);
            }
            if mutation.claim_writer {
                let expected_epoch = state
                    .writer_epoch
                    .checked_add(1)
                    .ok_or_else(|| "writer epoch overflow".to_string())?;
                if mutation.writer_epoch != expected_epoch {
                    continue;
                }
            } else if mutation.writer_epoch != state.writer_epoch {
                return Err(format!(
                    "catalog mutation writer epoch {} does not match {}",
                    mutation.writer_epoch, state.writer_epoch
                ));
            }
            let commit = seal_catalog_commit(CatalogCommit {
                generation: state
                    .generation
                    .checked_add(1)
                    .ok_or_else(|| "catalog generation overflow".to_string())?,
                parent_generation: state.generation,
                parent_digest: state.last_digest.clone(),
                mutation: mutation.clone(),
                digest: String::new(),
            })?;
            let next = Self::apply_catalog_mutation(&state, &commit)?;
            if !mutation.claim_writer
                && next.manifest.parts == state.manifest.parts
                && next.trace_manifest.parts == state.trace_manifest.parts
                && next.metric_manifest.parts == state.metric_manifest.parts
            {
                *self.catalog_state.lock().await = Some(state.clone());
                return Ok(state);
            }
            let bytes = serde_json::to_vec(&commit).map_err(|error| error.to_string())?;
            let first_path = self.catalog_commit_path("a", commit.generation);
            if !self.catalog_put_create(&first_path, &bytes).await? {
                *self.catalog_state.lock().await = None;
                continue;
            }
            let second_path = self.catalog_commit_path("b", commit.generation);
            if !self.catalog_put_create(&second_path, &bytes).await? {
                let existing = self
                    .catalog_get_bytes(&second_path)
                    .await?
                    .ok_or_else(|| format!("catalog replica disappeared: {second_path}"))?;
                if existing != bytes {
                    return Err(format!("catalog replica differs: {second_path}"));
                }
            }
            *self.catalog_state.lock().await = Some(next.clone());
            if next.generation % CATALOG_SNAPSHOT_INTERVAL == 0
                && let Err(error) = self.write_catalog_snapshot(&next).await
            {
                tracing::warn!(
                    generation = next.generation,
                    %error,
                    "catalog snapshot deferred"
                );
            }
            return Ok(next);
        }
        Err("catalog compare-and-create retry limit exceeded".to_string())
    }
}
