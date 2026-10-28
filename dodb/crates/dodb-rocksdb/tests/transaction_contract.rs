mod common;

use common::check_transaction_contract;
use dodb_rocksdb::{
    ConditionKeyLockMode, RevisionAllocatorMode, RocksDbService, RocksDbServiceConfig,
    minimal_native::{MinimalNativeConfig, MinimalNativeService},
};
use dodb_server::{
    BlinkCheckpointPolicy, BlinkTenantService, LocalTenantService, LocalTenantServiceConfig,
};
use tempfile::TempDir;

fn temporary_db_directory() -> TempDir {
    match std::env::var_os("DODB_TEST_DATA_DIR") {
        Some(path) => TempDir::new_in(path).unwrap(),
        None => TempDir::new().unwrap(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn minimal_native_transaction_path_satisfies_common_contract() {
    let directory = temporary_db_directory();
    let path = directory.path().to_path_buf();
    let service = || MinimalNativeService::new(MinimalNativeConfig::new(path.clone())).unwrap();
    check_transaction_contract(service(), service).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn shared_minimal_native_satisfies_transaction_and_reopen_contract() {
    let directory = temporary_db_directory();
    let service = || {
        let mut config = MinimalNativeConfig::new(directory.path());
        config.shared_resources = Some(dodb_rocksdb::shared_resources::SharedResourceConfig {
            cache_bytes: 16 * 1024 * 1024,
            wbm_bytes: 4 * 1024 * 1024,
            write_buffer_bytes: 256 * 1024,
            flush_threads: 1,
            compaction_threads: 1,
            max_open_files: 64,
        });
        MinimalNativeService::new(config).unwrap()
    };
    check_transaction_contract(service(), service).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn btree_blink_and_rocksdb_share_transaction_contract() {
    let native_directory = temporary_db_directory();
    let native_path = native_directory.path().to_path_buf();
    let native_service = || {
        LocalTenantService::new(LocalTenantServiceConfig {
            data_dir: native_path.clone(),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap()
    };
    check_transaction_contract(native_service(), native_service).await;

    let blink_directory = temporary_db_directory();
    let blink_path = blink_directory.path().to_path_buf();
    let blink_service = || {
        BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: blink_path.clone(),
            blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                threshold_bytes: 1,
            },
            ..LocalTenantServiceConfig::default()
        })
        .unwrap()
    };
    check_transaction_contract(blink_service(), blink_service).await;

    for condition_key_lock_mode in [
        ConditionKeyLockMode::Exclusive,
        ConditionKeyLockMode::Shared,
    ] {
        for revision_allocator in [
            RevisionAllocatorMode::Serialized,
            RevisionAllocatorMode::ReservedRanges,
        ] {
            let rocksdb_directory = temporary_db_directory();
            let rocksdb_path = rocksdb_directory.path().to_path_buf();
            let rocksdb_service = || {
                let mut config = RocksDbServiceConfig::new(&rocksdb_path);
                config.revision_allocator = revision_allocator;
                config.condition_key_lock_mode = condition_key_lock_mode;
                RocksDbService::new(config).unwrap()
            };
            check_transaction_contract(rocksdb_service(), rocksdb_service).await;
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn segmented_sync_and_background_satisfy_common_transaction_contract() {
    for execution in [
        dodb_server::BlinkCheckpointExecution::Sync,
        dodb_server::BlinkCheckpointExecution::Background,
    ] {
        let directory = temporary_db_directory();
        let service = || {
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                blink_wal_layout: dodb_server::BlinkWalLayout::Segmented {
                    max_segment_bytes: 1,
                },
                blink_checkpoint_execution: execution,
                blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                    threshold_bytes: 1,
                },
                ..LocalTenantServiceConfig::default()
            })
            .unwrap()
        };
        check_transaction_contract(service(), service).await;
    }
}
