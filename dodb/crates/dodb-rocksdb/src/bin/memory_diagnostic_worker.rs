use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use dodb_core::{DocumentKey, Error, Revision, TenantId, TransactionMutation};
use dodb_protocol::ProtocolLimits;
use dodb_rocksdb::raw_research::RawRocksDb;
use dodb_rocksdb::{
    ConditionKeyLockMode, RevisionAllocatorMode, RocksDbService, RocksDbServiceConfig,
    engine_health_disabled_report,
};
use dodb_server::{
    BlinkTenantService, DodbServer, DodbServerConfig, LocalTenantServiceConfig, ServerTlsConfig,
};
use dodb_service::{
    Document, DodbService, ExecutionBudget, Request, Response, ServiceFuture, ShutdownFuture,
    TransactionOutcome,
};
use dodb_storage::{CoordinatorConfig, DatabaseConfig};
use serde_json::json;
use tokio::sync::mpsc;

type WorkerError = Box<dyn std::error::Error + Send + Sync>;
type WorkerResult<T> = Result<T, WorkerError>;

struct Arguments {
    backend: String,
    data_dir: PathBuf,
    certificate_path: PathBuf,
    private_key_path: PathBuf,
    health_path: PathBuf,
    source_revision: String,
    worker_binary_sha256: String,
    parent_binary_sha256: String,
    response_budget: usize,
    max_mutations: usize,
    revision_allocator: RevisionAllocatorMode,
    condition_key_lock_mode: ConditionKeyLockMode,
    blocking_slots: usize,
    blocking_admission: dodb_rocksdb::BlockingAdmissionMode,
    engine_health_interval_ms: u64,
    tenant_count: usize,
    core_tenant_subdirectories: bool,
    blink_max_open_shards: usize,
    blink_cache_pages: Option<usize>,
}

enum BackendService {
    Blink(BlinkTenantService),
    RocksDb(RocksDbService),
    Core(BTreeMap<TenantId, Arc<dodb_rocksdb::research::SynchronousRocksDbTenant>>),
    Raw(Arc<RawRocksDb>),
}

impl BackendService {
    fn open(arguments: &Arguments) -> WorkerResult<Self> {
        match arguments.backend.as_str() {
            "blink" => {
                let config = LocalTenantServiceConfig {
                    data_dir: arguments.data_dir.clone(),
                    max_open_shards: arguments.blink_max_open_shards,
                    database_config: arguments
                        .blink_cache_pages
                        .map_or_else(DatabaseConfig::default, |capacity| {
                            DatabaseConfig::default().with_cache_capacity(capacity)
                        }),
                    coordinator_config: CoordinatorConfig::default(),
                    ..LocalTenantServiceConfig::default()
                };
                Ok(Self::Blink(BlinkTenantService::new(config)?))
            }
            "rocksdb-service" => {
                let mut config = RocksDbServiceConfig::new(&arguments.data_dir);
                config.revision_allocator = arguments.revision_allocator;
                config.condition_key_lock_mode = arguments.condition_key_lock_mode;
                config.max_blocking_operations = arguments.blocking_slots;
                config.blocking_admission = arguments.blocking_admission;
                config.detailed_write_metrics = false;
                Ok(Self::RocksDb(RocksDbService::new(config)?))
            }
            "core" => {
                let mut tenants = BTreeMap::new();
                for tenant_number in 1..=arguments.tenant_count {
                    let tenant = TenantId::new(tenant_number as u64);
                    let path = if arguments.core_tenant_subdirectories {
                        arguments.data_dir.join(format!("tenant-{tenant_number}"))
                    } else {
                        arguments.data_dir.clone()
                    };
                    let database = dodb_rocksdb::research::SynchronousRocksDbTenant::open_with_condition_key_lock_mode(
                        &path,
                        arguments.revision_allocator,
                        arguments.condition_key_lock_mode,
                    )?;
                    tenants.insert(tenant, Arc::new(database));
                }
                Ok(Self::Core(tenants))
            }
            "raw" => Ok(Self::Raw(Arc::new(RawRocksDb::open(&arguments.data_dir)?))),
            backend => Err(format!("unsupported diagnostic worker backend {backend}").into()),
        }
    }

    fn engine_health_observer(&self) -> Option<dodb_rocksdb::EngineHealthObserver> {
        match self {
            Self::Blink(_) => None,
            Self::RocksDb(service) => Some(service.engine_health_observer()),
            Self::Core(tenants) => Some(dodb_rocksdb::EngineHealthObserver::combine(
                tenants.iter().map(|(tenant, database)| {
                    database.engine_health_observer_named(format!("tenant_{tenant}"))
                }),
            )),
            Self::Raw(database) => Some(database.engine_health_observer()),
        }
    }

    fn execute_raw(database: &RawRocksDb, request: Request) -> Result<Response, Error> {
        match request {
            Request::Transact { request } if request.conditions.is_empty() => {
                let entries = request
                    .mutations
                    .iter()
                    .map(|mutation| match mutation {
                        TransactionMutation::Put { key, value } => {
                            Ok((physical_key(key), raw_envelope(value)))
                        }
                        TransactionMutation::Delete { .. } => Err(Error::invalid_request(
                            "raw unconditional diagnostic supports Put mutations only",
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                database.write_batch(&entries)?;
                Ok(Response::Transact(
                    TransactionOutcome::conditions_satisfied(),
                ))
            }
            Request::Scan {
                exclusive_after_key,
                limit,
            } => {
                let cursor = exclusive_after_key.as_ref().map(physical_key);
                let rows = database.scan_page(cursor.as_deref(), limit)?;
                let documents = rows
                    .into_iter()
                    .map(|(key_bytes, value_bytes)| decode_raw_row(&key_bytes, &value_bytes))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Response::Scan(documents))
            }
            _ => Err(Error::invalid_request(
                "raw diagnostic implements only unconditional Put batches and bounded Scan",
            )),
        }
    }
}

impl DodbService for BackendService {
    fn execute<'service>(
        &'service self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move {
            match self {
                Self::Blink(service) => service.execute(tenant, request, budget).await,
                Self::RocksDb(service) => service.execute(tenant, request, budget).await,
                Self::Core(tenants) => {
                    let service = Arc::clone(tenants.get(&tenant).ok_or_else(|| {
                        Error::invalid_request(format!("tenant {tenant} was not preopened"))
                    })?);
                    tokio::task::spawn_blocking(move || service.execute(request, budget))
                        .await
                        .map_err(|error| Error::invariant(error.to_string()))?
                }
                Self::Raw(database) => {
                    let database = Arc::clone(database);
                    tokio::task::spawn_blocking(move || Self::execute_raw(&database, request))
                        .await
                        .map_err(|error| Error::invariant(error.to_string()))?
                }
            }
        })
    }

    fn shutdown<'service>(&'service self) -> ShutdownFuture<'service> {
        Box::pin(async move {
            match self {
                Self::Blink(service) => service.shutdown().await,
                Self::RocksDb(service) => service.shutdown().await,
                Self::Core(_) | Self::Raw(_) => {}
            }
        })
    }
}

#[tokio::main]
async fn main() -> WorkerResult<()> {
    let _ = quinn::rustls::crypto::aws_lc_rs::default_provider().install_default();
    let arguments = parse_arguments()?;
    let backend = Arc::new(BackendService::open(&arguments)?);
    let protocol_limits = ProtocolLimits {
        max_response_frame_size: arguments.response_budget,
        max_mutations: arguments.max_mutations,
        ..ProtocolLimits::default()
    };
    let certificate = std::fs::read(&arguments.certificate_path)?;
    let private_key = std::fs::read(&arguments.private_key_path)?;
    let tls = ServerTlsConfig::from_der(vec![certificate], private_key)?;
    let server = Arc::new(
        DodbServer::bind(
            Arc::clone(&backend),
            DodbServerConfig {
                listen_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
                tls,
                protocol_limits,
                max_connections: 4,
                max_concurrent_streams: 1_024,
                max_concurrent_requests: 1_024,
            },
        )
        .map_err(|error| Box::new(error) as WorkerError)?,
    );
    let listen_address = server.local_addr()?;
    let mut server_task = tokio::spawn({
        let server = Arc::clone(&server);
        async move { server.run().await }
    });
    tokio::task::yield_now().await;
    let readiness = dodb_rocksdb::memory_diagnostic::worker_readiness(
        listen_address,
        arguments.source_revision.clone(),
        arguments.worker_binary_sha256.clone(),
    )?;
    println!(
        "{}",
        serde_json::to_string(&json!({
            "ready": readiness.ready,
            "listen_address": readiness.listen_address,
            "pid": readiness.pid,
            "process_group_id": readiness.process_group_id,
            "started_unix_ms": readiness.started_unix_ms,
            "process_start_time_ticks": readiness.process_start_time_ticks,
            "source_revision": readiness.source_revision,
            "worker_binary_sha256": readiness.worker_binary_sha256,
            "parent_binary_sha256": arguments.parent_binary_sha256,
            "backend": arguments.backend,
        }))?
    );
    std::io::stdout().flush()?;
    let (command_sender, mut command_receiver) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        for command_result in std::io::stdin().lock().lines() {
            match command_result {
                Ok(command) => {
                    if command_sender.send(command).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    let mut health_sampler = None;
    let mut health_report = engine_health_disabled_report();
    while let Some(command) = command_receiver.recv().await {
        match command.as_str() {
            "start-measurement" => {
                health_sampler = if arguments.engine_health_interval_ms == 0 {
                    None
                } else {
                    backend
                        .engine_health_observer()
                        .map(|observer| {
                            observer.start_sampling(
                                Duration::from_millis(arguments.engine_health_interval_ms),
                                Duration::from_secs(86_400),
                            )
                        })
                        .transpose()?
                };
            }
            "stop-measurement" => {
                health_report = match health_sampler.take() {
                    Some(sampler) => sampler.finish()?,
                    None if arguments.engine_health_interval_ms == 0 => {
                        engine_health_disabled_report()
                    }
                    None => json!({"status": "unsupported"}),
                };
            }
            "shutdown" => break,
            other => return Err(format!("unknown diagnostic worker command {other}").into()),
        }
    }
    if let Some(sampler) = health_sampler.take() {
        health_report = sampler.finish()?;
    }
    let shutdown = tokio::time::timeout(Duration::from_secs(15), async {
        server.shutdown().await;
        (&mut server_task).await
    })
    .await;
    match shutdown {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(error))) => return Err(Box::new(error) as WorkerError),
        Ok(Err(error)) => return Err(Box::new(error) as WorkerError),
        Err(_) => {
            server_task.abort();
            return Err("diagnostic worker shutdown exceeded 15 seconds".into());
        }
    }
    std::fs::write(
        &arguments.health_path,
        serde_json::to_vec_pretty(&json!({
            "backend": arguments.backend,
            "response_budget_bytes": arguments.response_budget,
            "max_mutations": arguments.max_mutations,
            "engine_health_interval_ms": arguments.engine_health_interval_ms,
            "engine_health": health_report,
            "shutdown_complete": true,
        }))?,
    )?;
    Ok(())
}

fn parse_arguments() -> WorkerResult<Arguments> {
    let values = std::env::args().skip(1).collect::<Vec<_>>();
    let backend = argument_value(&values, "--backend")?;
    let data_dir = PathBuf::from(argument_value(&values, "--memory-data-dir")?);
    let certificate_path = PathBuf::from(argument_value(&values, "--memory-certificate")?);
    let private_key_path = PathBuf::from(argument_value(&values, "--memory-private-key")?);
    let health_path = PathBuf::from(argument_value(&values, "--memory-health-report")?);
    let source_revision = argument_value(&values, "--memory-source-revision")?;
    let worker_binary_sha256 = argument_value(&values, "--memory-worker-binary-sha256")?;
    let parent_binary_sha256 = argument_value(&values, "--memory-parent-binary-sha256")?;
    let response_budget = argument_value(&values, "--memory-response-budget")?.parse()?;
    let max_mutations = argument_value(&values, "--memory-max-mutations")?.parse()?;
    let blocking_slots = option_value(&values, "--shared-blocking-slots")
        .unwrap_or_else(|| "8".to_owned())
        .parse()?;
    let blocking_admission = match option_value(&values, "--blocking-admission")
        .unwrap_or_else(|| "shared".to_owned())
        .as_str()
    {
        "shared" => dodb_rocksdb::BlockingAdmissionMode::Shared,
        "split-read-write" => dodb_rocksdb::BlockingAdmissionMode::SplitReadWrite,
        "split-read-2-write-8" => dodb_rocksdb::BlockingAdmissionMode::SplitRead2Write8,
        other => return Err(format!("unknown blocking admission {other}").into()),
    };
    let blocking_slots = blocking_admission
        .split_slot_limits()
        .map_or(blocking_slots, |(read_slots, write_slots)| {
            read_slots + write_slots
        });
    let engine_health_interval_ms = option_value(&values, "--engine-health-interval-ms")
        .unwrap_or_else(|| "0".to_owned())
        .parse()?;
    let tenant_count = option_value(&values, "--memory-tenant-count")
        .unwrap_or_else(|| "1".to_owned())
        .parse()?;
    let core_tenant_subdirectories = option_value(&values, "--memory-core-tenant-subdirectories")
        .is_some_and(|value| value == "1");
    let blink_max_open_shards = option_value(&values, "--memory-blink-max-open-shards")
        .unwrap_or_else(|| "1024".to_owned())
        .parse()?;
    let blink_cache_pages = option_value(&values, "--memory-blink-cache-pages")
        .map(|value| value.parse())
        .transpose()?;
    if tenant_count == 0 {
        return Err("memory tenant count must be nonzero".into());
    }
    let revision_allocator = match option_value(&values, "--revision-allocator")
        .unwrap_or_else(|| "serialized".to_owned())
        .as_str()
    {
        "serialized" => RevisionAllocatorMode::Serialized,
        "reserved-ranges" => RevisionAllocatorMode::ReservedRanges,
        other => return Err(format!("unknown revision allocator {other}").into()),
    };
    let condition_key_lock_mode = match option_value(&values, "--condition-key-lock-mode")
        .unwrap_or_else(|| "exclusive".to_owned())
        .as_str()
    {
        "exclusive" => ConditionKeyLockMode::Exclusive,
        "shared" => ConditionKeyLockMode::Shared,
        other => return Err(format!("unknown condition key lock mode {other}").into()),
    };
    Ok(Arguments {
        backend,
        data_dir,
        certificate_path,
        private_key_path,
        health_path,
        source_revision,
        worker_binary_sha256,
        parent_binary_sha256,
        response_budget,
        max_mutations,
        revision_allocator,
        condition_key_lock_mode,
        blocking_slots,
        blocking_admission,
        engine_health_interval_ms,
        tenant_count,
        core_tenant_subdirectories,
        blink_max_open_shards,
        blink_cache_pages,
    })
}

fn argument_value(arguments: &[String], name: &str) -> WorkerResult<String> {
    option_value(arguments, name).ok_or_else(|| format!("missing argument {name}").into())
}

fn option_value(arguments: &[String], name: &str) -> Option<String> {
    let position = arguments.iter().position(|argument| argument == name)?;
    arguments.get(position + 1).cloned()
}

fn physical_key(key: &DocumentKey) -> Vec<u8> {
    let encoded = key.encode();
    let mut physical = Vec::with_capacity(encoded.len() + 1);
    physical.push(1);
    physical.extend_from_slice(&encoded);
    physical
}

fn raw_envelope(payload: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(payload.len() + 10);
    encoded.extend_from_slice(&[1, 1]);
    encoded.extend_from_slice(&0_u64.to_be_bytes());
    encoded.extend_from_slice(payload);
    encoded
}

fn decode_raw_row(key_bytes: &[u8], value_bytes: &[u8]) -> Result<Document, Error> {
    if key_bytes.first() != Some(&1)
        || value_bytes.len() < 10
        || value_bytes[0] != 1
        || value_bytes[1] != 1
    {
        return Err(Error::corruption("raw row has an unexpected encoding"));
    }
    let key = DocumentKey::decode(&key_bytes[1..])
        .map_err(|error| Error::corruption(error.to_string()))?;
    Ok(Document {
        key,
        value: value_bytes[10..].to_vec(),
        revision: Revision::ZERO,
    })
}
