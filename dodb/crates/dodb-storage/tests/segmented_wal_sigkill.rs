use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dodb_core::{DocumentKey, Revision, RevisionState};
use dodb_storage::{BlinkStore, DatabaseConfig, ProductionFile};

const CHILD_DATABASE_ENV: &str = "DODB_SEGMENTED_SIGKILL_DATABASE";
const CHILD_READY_ENV: &str = "DODB_SEGMENTED_SIGKILL_READY";
const CHILD_ACK_ENV: &str = "DODB_SEGMENTED_SIGKILL_ACK";

fn database_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let directory = std::env::temp_dir().join(format!(
        "dodb-segmented-sigkill-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory.join("tenant.db")
}

fn key(index: u64) -> DocumentKey {
    DocumentKey::new(b"sigkill".to_vec(), index.to_be_bytes().to_vec())
}

#[test]
fn segmented_child_writer() {
    let Some(path) = std::env::var_os(CHILD_DATABASE_ENV).map(PathBuf::from) else {
        return;
    };
    let ready_path = PathBuf::from(std::env::var_os(CHILD_READY_ENV).unwrap());
    let ack_path = PathBuf::from(std::env::var_os(CHILD_ACK_ENV).unwrap());
    let mut store = BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
        &path,
        DatabaseConfig::default(),
        1,
    )
    .unwrap();
    let mut acknowledgements = fs::File::create(ack_path).unwrap();
    for index in 1..=5 {
        let revision = store
            .put(key(index), format!("acknowledged-{index}").into_bytes())
            .unwrap();
        use std::io::Write;
        writeln!(acknowledgements, "{index},{revision}").unwrap();
        acknowledgements.sync_all().unwrap();
    }
    fs::write(ready_path, b"ready").unwrap();
    loop {
        thread::park();
    }
}

#[test]
fn sigkill_after_acknowledged_segment_rotations_recovers_parent_expected_state() {
    let path = database_path();
    let ready_path = path.with_extension("ready");
    let ack_path = path.with_extension("ack");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("segmented_child_writer")
        .arg("--nocapture")
        .env(CHILD_DATABASE_ENV, &path)
        .env(CHILD_READY_ENV, &ready_path)
        .env(CHILD_ACK_ENV, &ack_path)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !ready_path.exists() && Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            panic!("segmented WAL child exited before readiness: {status}");
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(ready_path.exists());
    child.kill().unwrap();
    child.wait().unwrap();
    let acknowledgements = fs::read_to_string(ack_path).unwrap();
    let expected = acknowledgements
        .lines()
        .map(|line| {
            let (index, revision) = line.split_once(',').unwrap();
            (
                index.parse::<u64>().unwrap(),
                Revision::new(revision.parse().unwrap()),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 5);

    let mut recovered = BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
        &path,
        DatabaseConfig::default(),
        1,
    )
    .unwrap();
    for (index, revision) in expected {
        assert_eq!(
            recovered.get(&key(index)).unwrap(),
            RevisionState::Present {
                value: format!("acknowledged-{index}").into_bytes(),
                revision,
            }
        );
    }
    assert!(
        fs::read_dir(path.with_file_name("tenant.db.segments"))
            .unwrap()
            .count()
            >= 5
    );
    drop(recovered);
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
