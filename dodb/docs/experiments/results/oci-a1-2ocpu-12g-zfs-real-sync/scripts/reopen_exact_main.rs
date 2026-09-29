use std::error::Error;
use std::fs;
use std::path::PathBuf;

use dodb_core::{DocumentKey, RevisionState, TransactionMutation, TransactionRequest};
use dodb_storage::{BTreeStore, DatabaseConfig, ProductionFile};

fn main() -> Result<(), Box<dyn Error>> {
    let database_path = PathBuf::from("/bench/zfs/db/reopen-exact-main.db");
    let wal_path = database_path.with_extension("wal");
    let _ = fs::remove_file(&database_path);
    let _ = fs::remove_file(&wal_path);
    let mut expected = Vec::new();
    let mut mutations = Vec::new();
    for mutation_index in 0..16u64 {
        let key = DocumentKey::new(b"reopen-check-pk".to_vec(), mutation_index.to_be_bytes().to_vec());
        let value = format!("exactmain-durable-value-{mutation_index}").into_bytes();
        mutations.push(TransactionMutation::Put {
            key: key.clone(),
            value: value.clone(),
        });
        expected.push((key, value));
    }
    let request = TransactionRequest::new(Vec::new(), mutations);
    {
        let mut store = BTreeStore::<ProductionFile, ProductionFile>::open_with_wal(
            ProductionFile::open(&database_path)?,
            ProductionFile::open(&wal_path)?,
            DatabaseConfig::default(),
        )?;
        store.transact(request)?;
    }
    {
        let mut store = BTreeStore::<ProductionFile, ProductionFile>::open_with_wal(
            ProductionFile::open(&database_path)?,
            ProductionFile::open(&wal_path)?,
            DatabaseConfig::default(),
        )?;
        for (key, expected_value) in &expected {
            match store.get(key)? {
                RevisionState::Present { value, .. } if &value == expected_value => {}
                state => return Err(format!("unexpected recovered value: {state:?}").into()),
            }
        }
    }
    println!("engine=ExactMain logical_transaction_width=16 clean_close_reopen=PASS recovered_values=16/16 path={}", database_path.display());
    let _ = fs::remove_file(&database_path);
    let _ = fs::remove_file(&wal_path);
    Ok(())
}
