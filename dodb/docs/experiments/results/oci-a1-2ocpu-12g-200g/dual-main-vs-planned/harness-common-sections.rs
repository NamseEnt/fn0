===== WorkloadGenerator =====
#[derive(Clone, Debug)]
struct WorkloadGenerator {
    config: WorkloadConfig,
    worker_id: usize,
    operation: u64,
    state: u64,
}

impl WorkloadGenerator {
    fn new(config: WorkloadConfig, seed: u64, worker_id: usize) -> Self {
        Self {
            config,
            worker_id,
            operation: 0,
            state: seed ^ (worker_id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
        }
    }

    fn next_transaction(&mut self) -> TransactionRequest {
        let mut keys = Vec::with_capacity(self.config.width);
        let mut seen = HashSet::with_capacity(self.config.width);
        for offset in 0..self.config.width {
            let index = self.next_index(offset);
            let key = match self.config.transaction_mode {
                TransactionMode::Unconditional => self.key_for_index(index),
                TransactionMode::InsertIfAbsent => self.key_for_index(
                    self.config
                        .working_set
                        .saturating_add(self.worker_id.saturating_mul(1_000_000))
                        .saturating_add((self.operation as usize).saturating_mul(self.config.width))
                        .saturating_add(offset),
                ),
            };
            if seen.insert(key.clone()) {
                keys.push(key);
            } else {
                let fallback = self.key_for_index(
                    self.config
                        .working_set
                        .saturating_add(self.worker_id.saturating_mul(1_000_000))
                        .saturating_add((self.operation as usize).saturating_mul(self.config.width))
                        .saturating_add(offset),
                );
                assert!(
                    seen.insert(fallback.clone()),
                    "transaction key generator duplicated a key"
                );
                keys.push(fallback);
            }
        }
        self.operation = self.operation.wrapping_add(1);
        let mutations = keys
            .iter()
            .enumerate()
            .map(|(offset, key)| TransactionMutation::Put {
                key: key.clone(),
                value: value_bytes(self.config.value_size, self.operation, offset),
            })
            .collect::<Vec<_>>();
        let conditions = match self.config.transaction_mode {
            TransactionMode::Unconditional => Vec::new(),
            TransactionMode::InsertIfAbsent => keys
                .into_iter()
                .map(|key| TransactionCondition::NotExists { key })
                .collect(),
        };
        TransactionRequest::new(conditions, mutations)
    }

    fn next_read(&mut self, kind: ReadKind) -> BatchRequest {
        let index = self.next_index(0);
        self.operation = self.operation.wrapping_add(1);
        match kind {
            ReadKind::Get => BatchRequest::Get {
                key: self.key_for_index(index),
            },
            ReadKind::Query => BatchRequest::Query {
                pk: PrimaryKey::new(query_pk(self.config.key_size)),
                exclusive_after_sk: None,
                limit: self.config.read_limit,
            },
            ReadKind::Scan => BatchRequest::Scan {
                exclusive_after_key: None,
                limit: self.config.read_limit,
            },
        }
    }

    fn next_index(&mut self, offset: usize) -> usize {
        let working_set = self.config.working_set;
        match self.config.distribution {
            Distribution::Uniform => self.random_bounded(working_set),
            Distribution::Sequential => {
                ((self.operation as usize)
                    .saturating_mul(self.config.width)
                    .saturating_add(offset))
                    % working_set
            }
            Distribution::Hotspot => {
                let hotset = (working_set / 100).max(self.config.width);
                if self.random_bounded(100) < 80 {
                    self.random_bounded(hotset.min(working_set))
                } else {
                    self.random_bounded(working_set)
                }
            }
            Distribution::SameLeafHeavy => {
                let span = working_set.min(64).max(self.config.width);
                ((self.operation as usize)
                    .saturating_mul(self.config.width)
                    .saturating_add(offset))
                    % span.min(working_set)
            }
            Distribution::DifferentLeafHeavy => {
                (self
                    .worker_id
                    .saturating_mul(1_009)
                    .saturating_add((self.operation as usize).saturating_mul(self.config.width))
                    .saturating_add(offset))
                    % working_set
            }
        }
    }

    fn random_bounded(&mut self, bound: usize) -> usize {
        assert!(bound > 0);
        self.state = splitmix64(self.state);
        (self.state as usize) % bound
    }

    fn key_for_index(&self, index: usize) -> DocumentKey {
        let (pk_len, sk_len) = key_component_lengths(self.config.key_size);
        match self.config.distribution {
            Distribution::SameLeafHeavy => DocumentKey::new(
                component_bytes(0x11, 0, pk_len),
                component_bytes(0x21, index as u64, sk_len),
            ),
            Distribution::DifferentLeafHeavy => DocumentKey::new(
                component_bytes(0x31, index as u64, pk_len),
                component_bytes(0x41, index as u64, sk_len),
            ),
            Distribution::Uniform | Distribution::Sequential | Distribution::Hotspot => {
                DocumentKey::new(
                    component_bytes(0x51, (index % 128) as u64, pk_len),
                    component_bytes(0x61, index as u64, sk_len),
                )
            }
        }
    }

    fn seed_keys(&self) -> impl Iterator<Item = DocumentKey> + '_ {
        (0..self.config.working_set).map(|index| self.key_for_index(index))
    }
}

===== writer_loop =====
async fn writer_loop(
    adapter: Arc<dyn EngineAdapter>,
    workload: WorkloadConfig,
    seed: u64,
    worker_id: usize,
    deadline: Instant,
    quota: Option<MixQuota>,
    warmup: bool,
) -> WorkerStats {
    let mut generator = WorkloadGenerator::new(workload, seed, worker_id);
    let mut stats = WorkerStats::new(seed ^ worker_id as u64);
    while Instant::now() < deadline {
        if let Some(quota) = &quota
            && !quota.claim(Role::Writer, deadline).await
        {
            break;
        }
        let request = generator.next_transaction();
        let width = request.mutations.len() as u64;
        let started = Instant::now();
        let result = adapter.execute_transaction(request).await;
        let elapsed = started.elapsed();
        if !warmup {
            stats.attempted_transactions += 1;
            stats.e2e_latency.push(elapsed);
            stats.write_latency.push(elapsed);
            match result {
                Ok(_) => {
                    stats.successful_transactions += 1;
                    stats.mutation_ops += width;
                }
                Err(Error::Conflict(_)) => stats.conflicts += 1,
                Err(Error::Overloaded(_)) => stats.overloads += 1,
                Err(_) => stats.errors += 1,
            }
        }
    }
    stats
}

===== BenchFile_and_sync_disabled =====
struct BenchFile {
    inner: ProductionFile,
    sync_mode: SyncMode,
    sync_delay: Duration,
}

impl BenchFile {
    fn open(path: &Path, sync_mode: SyncMode, sync_delay: Duration) -> Result<Self> {
        Ok(Self {
            inner: ProductionFile::open(path)?,
            sync_mode,
            sync_delay,
        })
    }
}

fn perform_benchmark_sync(
    mode: SyncMode,
    delay: Duration,
    real_sync: impl FnOnce() -> Result<()>,
) -> Result<()> {
    match mode {
        SyncMode::Real => real_sync(),
        SyncMode::Injected => {
            if !delay.is_zero() {
                std::thread::sleep(delay);
            }
            real_sync()
        }
        SyncMode::Disabled => Ok(()),
    }
}

impl DurableFile for BenchFile {
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
        self.inner.write_at(offset, bytes)
    }

    fn len(&self) -> Result<u64> {
        self.inner.len()
    }

    fn set_len(&mut self, length: u64) -> Result<()> {
        self.inner.set_len(length)
    }

    fn sync_data(&mut self) -> Result<()> {
        let mode = self.sync_mode;
        let delay = self.sync_delay;
        perform_benchmark_sync(mode, delay, || self.inner.sync_data())
    }

    fn sync_all(&mut self) -> Result<()> {
        let mode = self.sync_mode;
        let delay = self.sync_delay;
        perform_benchmark_sync(mode, delay, || self.inner.sync_all())
    }
}

===== scenario_generation =====
fn scenarios(args: &Args) -> Vec<Scenario> {
    let delay_values = [
        Duration::ZERO,
        Duration::from_micros(50),
        Duration::from_micros(100),
        Duration::from_micros(250),
        Duration::from_micros(500),
        Duration::from_millis(1),
        Duration::from_millis(2),
        Duration::from_millis(5),
    ];
    let selected_delay = args.collection_delay.unwrap_or(Duration::ZERO);
    let distributions = args.distributions();
    let widths = args.widths();
    let writers = args.writers();
    let readers = args.readers();
    let read_kinds = args.read_kinds();
    let mixes = args.mixes();
    let mut output = Vec::new();

    let include = |suite: Suite, selected: Suite| {
        args.suite == Suite::All || args.suite == selected || args.suite == suite
    };

    if include(Suite::Write, Suite::Write) {
        for writer in &writers {
            for width in &widths {
                for distribution in &distributions {
                    output.push(Scenario {
                        suite: Suite::Write,
                        workload: "100%-write",
                        writers: *writer,
                        readers: 0,
                        width: *width,
                        distribution: *distribution,
                        read_kind: None,
                        mix: None,
                        collection_delay: selected_delay,
                        sync_delay: args.sync_delay,
                    });
                }
            }
        }
    }

    if include(Suite::Read, Suite::Read) {
        for reader in &readers {
            for read_kind in &read_kinds {
                output.push(Scenario {
                    suite: Suite::Read,
                    workload: match read_kind {
                        ReadKind::Get => "100%-read-get",
                        ReadKind::Query => "100%-read-query",
                        ReadKind::Scan => "100%-read-scan",
                    },
                    writers: 0,
                    readers: *reader,
                    width: 1,
                    distribution: Distribution::Uniform,
                    read_kind: Some(*read_kind),
                    mix: None,
                    collection_delay: Duration::ZERO,
                    sync_delay: Duration::ZERO,
                });
            }
        }
    }

    if include(Suite::Mixed, Suite::Mixed) {
        let mixed_readers = args.readers.clone().unwrap_or_else(|| vec![16, 64]);
        let mixed_writers = args.writers.clone().unwrap_or_else(|| vec![16, 64]);
        for readers in &mixed_readers {
            for writers in &mixed_writers {
                for mix in &mixes {
                    output.push(Scenario {
                        suite: Suite::Mixed,
                        workload: "mixed",
                        writers: *writers,
                        readers: *readers,
                        width: 1,
                        distribution: Distribution::Uniform,
                        read_kind: Some(ReadKind::Get),
                        mix: Some(*mix),
                        collection_delay: selected_delay,
                        sync_delay: args.sync_delay,
                    });
                }
            }
        }
    }

    if args.suite == Suite::DelaySweep || args.suite == Suite::All {
        let delays = if args.collection_delay.is_some() {
            vec![selected_delay]
        } else {
            delay_values.to_vec()
        };
        for delay in delays {
            output.push(Scenario {
                suite: Suite::DelaySweep,
                workload: "delay-write-control",
                writers: 64,
                readers: 0,
                width: 1,
                distribution: Distribution::Uniform,
                read_kind: None,
                mix: None,
                collection_delay: delay,
                sync_delay: args.sync_delay,
            });
            output.push(Scenario {
                suite: Suite::DelaySweep,
                workload: "delay-mixed-control",
                writers: 64,
                readers: 16,
                width: 1,
                distribution: Distribution::Uniform,
                read_kind: Some(ReadKind::Get),
                mix: Some(Mix::BALANCED),
                collection_delay: delay,
                sync_delay: args.sync_delay,
            });
        }
    }

    if args.suite == Suite::SyncSweep || args.suite == Suite::All {
        let sync_values = [
            Duration::ZERO,
            Duration::from_micros(100),
            Duration::from_millis(1),
            Duration::from_millis(5),
            Duration::from_millis(10),
        ];
        for sync_delay in sync_values {
            output.push(Scenario {
                suite: Suite::SyncSweep,
                workload: "sync-write-control",
                writers: 64,
                readers: 0,
                width: 1,
                distribution: Distribution::Uniform,
                read_kind: None,
                mix: None,
                collection_delay: Duration::ZERO,
                sync_delay,
            });
        }
    }

    assert!(!output.is_empty(), "selected suite produced no scenarios");
    output
}

===== seed_generation =====
fn seed_store(
    store: &mut BTreeStore<BenchFile, BenchFile>,
    args: &Args,
    scenario: &Scenario,
) -> Result<usize> {
    let requests = seed_requests(args, scenario);
    let seeded = requests.iter().map(|request| request.mutations.len()).sum();
    for chunk in requests.chunks(64) {
        store.apply_transaction_group(chunk)?;
    }
    Ok(seeded)
}

fn seed_requests(args: &Args, scenario: &Scenario) -> Vec<TransactionRequest> {
    let workload = WorkloadConfig {
        distribution: scenario.distribution,
        working_set: args.working_set,
        key_size: args.key_size,
        value_size: args.value_size,
        width: 25.min(args.working_set),
        transaction_mode: TransactionMode::Unconditional,
        read_limit: args.read_limit,
    };
    let generator = WorkloadGenerator::new(workload.clone(), args.seed, 0);
    let mut requests = Vec::new();
    let mut mutations = Vec::new();
    for (index, key) in generator.seed_keys().enumerate() {
        mutations.push(TransactionMutation::Put {
            key,
            value: value_bytes(args.value_size, index as u64, 0),
        });
        if mutations.len() >= 25 {
            requests.push(TransactionRequest::new(
                Vec::new(),
                std::mem::take(&mut mutations),
            ));
        }
    }
    if !mutations.is_empty() {
        requests.push(TransactionRequest::new(Vec::new(), mutations));
    }

    // Query/scan rows use a separate PK so that every read scenario returns a
    // stable, non-empty result without changing the write key distribution.
    if scenario.read_kind.is_some() || scenario.mix.is_some() {
        let query_rows = args.working_set.min(256);
        let mut query_mutations = Vec::new();
        for index in 0..query_rows {
            query_mutations.push(TransactionMutation::Put {
                key: query_key(args.key_size, index),
                value: value_bytes(args.value_size, index as u64, 0),
            });
            if query_mutations.len() >= 25 {
                requests.push(TransactionRequest::new(
                    Vec::new(),
                    std::mem::take(&mut query_mutations),
                ));
            }
        }
        if !query_mutations.is_empty() {
            requests.push(TransactionRequest::new(Vec::new(), query_mutations));
        }
    }

    requests
}

===== benchmark_config_and_sync =====
fn benchmark_config(args: &Args, scenario: &Scenario) -> CoordinatorConfig {
    CoordinatorConfig {
        queue_capacity: args.queue_capacity,
        max_group_requests: args.max_group_requests,
        max_group_bytes: args.max_group_bytes,
        max_collection_delay: scenario.collection_delay,
    }
}

fn effective_sync(args: &Args, scenario: &Scenario) -> (SyncMode, Duration) {
    if scenario.suite == Suite::SyncSweep {
        (SyncMode::Injected, scenario.sync_delay)
    } else {
        match args.sync_mode {
            SyncMode::Real => (SyncMode::Real, Duration::ZERO),
            SyncMode::Injected => (SyncMode::Injected, scenario.sync_delay),
            SyncMode::Disabled => (SyncMode::Disabled, Duration::ZERO),
        }
    }
}

===== warmup_and_measured_control =====
async fn run_repetition(
    args: &Args,
    scenario: &Scenario,
    machine: &MachineInfo,
    repetition: usize,
    seed: u64,
    output: &mut std::fs::File,
) -> Result<()> {
    let (adapter, data_path, seeded_rows) = open_adapter(args, scenario, repetition, seed).await?;
    let warmup_stats = run_interval(
        Arc::clone(&adapter),
        args,
        scenario,
        seed ^ 0xaaaa_0000,
        args.warmup,
        true,
    )
    .await;
    let _ = warmup_stats;
    let before = adapter.snapshot();
    let cpu_start = ProcessCpuSample::capture();
    let started = Instant::now();
    let measured = run_interval(
        Arc::clone(&adapter),
        args,
        scenario,
        seed ^ 0xbbbb_0000,
        args.duration,
        false,
    )
    .await;
    let wall = started.elapsed();
    let cpu_end = ProcessCpuSample::capture();
    let after = adapter.snapshot();
    let delta = MetricDelta::from(&before, &after);
    print_run_summary(scenario, repetition, &measured, wall, &delta);
    let line = build_record(
        machine,
        args,
        scenario,
        repetition,
        seed,
        seeded_rows,
        &measured,
        &delta,
        wall,
        &cpu_start,
        &cpu_end,
    );
    use std::io::Write;
    writeln!(output, "{line}")?;
    output.flush()?;
    adapter.shutdown().await?;
    remove_database_files(&data_path);
    Ok(())
}
