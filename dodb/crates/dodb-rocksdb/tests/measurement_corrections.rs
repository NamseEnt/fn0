use std::path::Path;

#[test]
fn wal_file_classification_includes_numeric_rocksdb_logs_only() {
    for filename in ["tenant-1/000039.log", "1.log", "blink/tenant-1-shard-1.wal"] {
        assert!(
            dodb_rocksdb::research::is_wal_file(Path::new(filename)),
            "{filename}"
        );
    }
    for filename in [
        "LOG",
        "LOG.old.1792482916081967",
        "workload.stdout.log",
        "wal-statistics.txt",
        "123x.log",
        ".log",
        "１２３.log",
        "000039.sst",
        "wal-directory/CURRENT",
    ] {
        assert!(
            !dodb_rocksdb::research::is_wal_file(Path::new(filename)),
            "{filename}"
        );
    }
}
