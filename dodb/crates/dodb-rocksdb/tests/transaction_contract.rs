mod common;

use common::check_transaction_contract;
use dodb_rocksdb::{RocksDbService, RocksDbServiceConfig};
use dodb_server::{LocalTenantService, LocalTenantServiceConfig};
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn native_and_rocksdb_share_transaction_contract() {
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

    let rocksdb_directory = TempDir::new().unwrap();
    let rocksdb_path = rocksdb_directory.path().to_path_buf();
    let rocksdb_service = || RocksDbService::new(RocksDbServiceConfig::new(&rocksdb_path)).unwrap();
    check_transaction_contract(rocksdb_service(), rocksdb_service).await;
}
