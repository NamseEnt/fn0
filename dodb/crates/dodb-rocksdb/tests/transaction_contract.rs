mod common;

use common::check_transaction_contract;
use dodb_rocksdb::{RocksDbService, RocksDbServiceConfig};
use dodb_server::{BlinkTenantService, LocalTenantService, LocalTenantServiceConfig};
use tempfile::TempDir;

fn temporary_db_directory() -> TempDir {
    match std::env::var_os("DODB_TEST_DATA_DIR") {
        Some(path) => TempDir::new_in(path).unwrap(),
        None => TempDir::new().unwrap(),
    }
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
            ..LocalTenantServiceConfig::default()
        })
        .unwrap()
    };
    check_transaction_contract(blink_service(), blink_service).await;

    let rocksdb_directory = temporary_db_directory();
    let rocksdb_path = rocksdb_directory.path().to_path_buf();
    let rocksdb_service = || RocksDbService::new(RocksDbServiceConfig::new(&rocksdb_path)).unwrap();
    check_transaction_contract(rocksdb_service(), rocksdb_service).await;
}
