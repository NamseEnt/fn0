use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use dodb_core::DocumentKey;
use dodb_storage::{BTreeStore, DatabaseConfig, FaultInjector, ProductionFile};

struct BlockingFault {
    point: String,
    marker_path: PathBuf,
}

impl FaultInjector for BlockingFault {
    fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
        if point == self.point {
            std::fs::write(&self.marker_path, point.as_bytes()).map_err(dodb_core::Error::from)?;
            loop {
                thread::sleep(Duration::from_millis(50));
            }
        }
        Ok(())
    }
}

fn main() {
    let mut arguments = env::args_os().skip(1);
    let data_path = arguments.next().expect("data path is required");
    let wal_path = arguments.next().expect("WAL path is required");
    let marker_path = arguments.next().expect("marker path is required");
    let checkpoint_point = arguments.next();

    let mut store = BTreeStore::open_with_wal(
        ProductionFile::open(data_path).expect("data file should open"),
        ProductionFile::open(wal_path).expect("WAL file should open"),
        DatabaseConfig::default(),
    )
    .expect("database should open");
    store
        .put(
            DocumentKey::new(b"subprocess".to_vec(), b"key".to_vec()),
            b"value".to_vec(),
        )
        .expect("WAL-backed put should succeed");
    if let Some(checkpoint_point) = checkpoint_point {
        store.set_fault_injector(BlockingFault {
            point: checkpoint_point.to_string_lossy().into_owned(),
            marker_path: marker_path.clone().into(),
        });
        let _ = store.checkpoint();
    } else {
        std::fs::write(marker_path, b"success-returned").expect("marker should be written");
    }

    loop {
        thread::sleep(Duration::from_millis(50));
    }
}
