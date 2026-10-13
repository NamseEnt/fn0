mod common;

use common::check_transaction_contract;
use dodb_rocksdb::{RocksDbService, RocksDbServiceConfig};
use dodb_server::{BlinkTenantService, LocalTenantService, LocalTenantServiceConfig};
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn btree_blink_and_rocksdb_share_transaction_contract() {
    let native_directory = TempDir::new().unwrap();
    let native_path = native_directory.path().to_path_buf();
    let native_service = || {
        LocalTenantService::new(LocalTenantServiceConfig {
            data_dir: native_path.clone(),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap()
    };
    check_transaction_contract(native_service(), native_service).await;

    let blink_directory = TempDir::new().unwrap();
    let blink_path = blink_directory.path().to_path_buf();
    let blink_service = || {
        BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: blink_path.clone(),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap()
    };
    check_transaction_contract(blink_service(), blink_service).await;

    let rocksdb_directory = TempDir::new().unwrap();
    let rocksdb_path = rocksdb_directory.path().to_path_buf();
    let rocksdb_service = || RocksDbService::new(RocksDbServiceConfig::new(&rocksdb_path)).unwrap();
    check_transaction_contract(rocksdb_service(), rocksdb_service).await;
}
