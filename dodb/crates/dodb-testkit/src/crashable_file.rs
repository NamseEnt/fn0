use std::collections::BTreeMap;
use std::io::ErrorKind;

use dodb_core::{Error, Result};
use dodb_storage::DurableFile;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FileOperation {
    ReadAt,
    WriteAt,
    SetLen,
    SyncData,
    SyncAll,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FaultAction {
    ShortWrite {
        max_bytes: usize,
    },
    ShortRead {
        max_bytes: usize,
    },
    Io {
        kind: ErrorKind,
        message: String,
    },
    SyncPersistAllThenIo {
        kind: ErrorKind,
        message: String,
    },
    SyncPersistPrefixThenIo {
        length: usize,
        kind: ErrorKind,
        message: String,
    },
}

impl FaultAction {
    pub fn short_write(max_bytes: usize) -> Self {
        Self::ShortWrite { max_bytes }
    }

    pub fn short_read(max_bytes: usize) -> Self {
        Self::ShortRead { max_bytes }
    }

    pub fn io(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self::Io {
            kind,
            message: message.into(),
        }
    }
}

/// Operation-indexed fault schedule. The operation index starts at zero and
/// increments once for every DurableFile method invocation.
#[derive(Clone, Debug, Default)]
pub struct FaultPlan {
    faults: BTreeMap<u64, (FileOperation, FaultAction)>,
    next_matching: BTreeMap<FileOperation, (u64, FaultAction)>,
    matching_hits: BTreeMap<FileOperation, u64>,
    next_operation: u64,
}

impl FaultPlan {
    pub fn at(
        mut self,
        operation_index: u64,
        operation: FileOperation,
        action: FaultAction,
    ) -> Self {
        self.faults.insert(operation_index, (operation, action));
        self
    }

    pub fn insert(&mut self, operation_index: u64, operation: FileOperation, action: FaultAction) {
        self.faults.insert(operation_index, (operation, action));
    }

    /// Schedules one fault on the next invocation of a matching file
    /// operation, regardless of preceding reads or metadata calls.
    pub fn on_next(self, operation: FileOperation, action: FaultAction) -> Self {
        self.on_nth(operation, 1, action)
    }

    /// Schedules one fault on the Nth invocation of a matching file
    /// operation, independent of other operation kinds.
    pub fn on_nth(mut self, operation: FileOperation, nth: u64, action: FaultAction) -> Self {
        self.next_matching.insert(operation, (nth.max(1), action));
        self
    }

    pub fn operation_count(&self) -> u64 {
        self.next_operation
    }

    fn next_fault(&mut self, operation: FileOperation) -> Option<FaultAction> {
        let index = self.next_operation;
        self.next_operation = self.next_operation.saturating_add(1);
        match self.faults.remove(&index) {
            Some((expected_operation, action)) if expected_operation == operation => Some(action),
            Some((expected_operation, action)) => {
                self.faults.insert(index, (expected_operation, action));
                None
            }
            None => {
                let hits = self.matching_hits.entry(operation).or_insert(0);
                *hits = hits.saturating_add(1);
                if self
                    .next_matching
                    .get(&operation)
                    .is_some_and(|(nth, _)| *nth == *hits)
                {
                    self.next_matching
                        .remove(&operation)
                        .map(|(_, action)| action)
                } else {
                    None
                }
            }
        }
    }
}

/// In-memory file with explicitly separate volatile and durable bytes.
pub struct CrashableFile {
    volatile: Vec<u8>,
    durable: Vec<u8>,
    fault_plan: FaultPlan,
}

impl CrashableFile {
    pub fn new() -> Self {
        Self::from_durable(Vec::new())
    }

    pub fn from_durable(bytes: impl Into<Vec<u8>>) -> Self {
        let durable = bytes.into();
        Self {
            volatile: durable.clone(),
            durable,
            fault_plan: FaultPlan::default(),
        }
    }

    pub fn with_fault_plan(mut self, fault_plan: FaultPlan) -> Self {
        self.fault_plan = fault_plan;
        self
    }

    pub fn set_fault_plan(&mut self, fault_plan: FaultPlan) {
        self.fault_plan = fault_plan;
    }

    pub fn volatile_bytes(&self) -> &[u8] {
        &self.volatile
    }

    pub fn durable_bytes(&self) -> &[u8] {
        &self.durable
    }

    /// Flips one durable byte and mirrors the corruption into the volatile
    /// view, modeling a damaged on-disk byte before the next open.
    pub fn corrupt_durable_byte(&mut self, offset: usize) -> Result<()> {
        let byte = self
            .durable
            .get_mut(offset)
            .ok_or_else(|| Error::invalid_input("corruption offset is outside the durable file"))?;
        *byte ^= 1;
        if let Some(volatile_byte) = self.volatile.get_mut(offset) {
            *volatile_byte = *byte;
        }
        Ok(())
    }

    /// Simulates process loss: unsynced volatile state is discarded.
    pub fn crash(&mut self) {
        self.volatile = self.durable.clone();
    }

    pub fn crash_with_persisted_prefix(&mut self, length: usize) {
        let persisted_length = length.min(self.volatile.len());
        self.durable = self.volatile[..persisted_length].to_vec();
        self.volatile = self.durable.clone();
    }

    fn fault(&mut self, operation: FileOperation) -> Result<Option<FaultAction>> {
        Ok(self.fault_plan.next_fault(operation))
    }

    fn io_error(action: FaultAction) -> Result<Option<FaultAction>> {
        match action {
            FaultAction::Io { kind, message } => Err(Error::Io(std::io::Error::new(kind, message))),
            FaultAction::SyncPersistAllThenIo { .. }
            | FaultAction::SyncPersistPrefixThenIo { .. } => Err(Error::invalid_input(
                "sync persistence fault used for a non-sync operation",
            )),
            other => Ok(Some(other)),
        }
    }

    fn checked_range(offset: u64, length: usize) -> Result<std::ops::Range<usize>> {
        let start = usize::try_from(offset)
            .map_err(|_| Error::invalid_input("file offset does not fit platform usize"))?;
        let end = start
            .checked_add(length)
            .ok_or_else(|| Error::invalid_input("file range overflows platform usize"))?;
        Ok(start..end)
    }
}

impl Default for CrashableFile {
    fn default() -> Self {
        Self::new()
    }
}

impl DurableFile for CrashableFile {
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        let action = self.fault(FileOperation::ReadAt)?;
        let action = action.map(Self::io_error).transpose()?.flatten();
        let start = usize::try_from(offset)
            .map_err(|_| Error::invalid_input("file offset does not fit platform usize"))?;
        if start >= self.volatile.len() || buffer.is_empty() {
            return Ok(0);
        }
        let available = (self.volatile.len() - start).min(buffer.len());
        let count = match action {
            Some(FaultAction::ShortRead { max_bytes }) => available.min(max_bytes),
            Some(FaultAction::ShortWrite { .. }) => available,
            None => available,
            Some(FaultAction::Io { .. })
            | Some(FaultAction::SyncPersistAllThenIo { .. })
            | Some(FaultAction::SyncPersistPrefixThenIo { .. }) => {
                unreachable!("I/O faults are returned above")
            }
        };
        buffer[..count].copy_from_slice(&self.volatile[start..start + count]);
        Ok(count)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
        let action = self.fault(FileOperation::WriteAt)?;
        let action = action.map(Self::io_error).transpose()?.flatten();
        let count = match action {
            Some(FaultAction::ShortWrite { max_bytes }) => bytes.len().min(max_bytes),
            Some(FaultAction::ShortRead { .. }) | None => bytes.len(),
            Some(FaultAction::Io { .. })
            | Some(FaultAction::SyncPersistAllThenIo { .. })
            | Some(FaultAction::SyncPersistPrefixThenIo { .. }) => {
                unreachable!("I/O faults are returned above")
            }
        };
        let range = Self::checked_range(offset, count)?;
        if self.volatile.len() < range.end {
            self.volatile.resize(range.end, 0);
        }
        self.volatile[range].copy_from_slice(&bytes[..count]);
        Ok(count)
    }

    fn len(&self) -> Result<u64> {
        u64::try_from(self.volatile.len())
            .map_err(|_| Error::invariant("volatile file length does not fit u64"))
    }

    fn set_len(&mut self, length: u64) -> Result<()> {
        let action = self.fault(FileOperation::SetLen)?;
        if let Some(FaultAction::Io { kind, message }) = action {
            return Err(Error::Io(std::io::Error::new(kind, message)));
        }
        if matches!(
            action,
            Some(
                FaultAction::SyncPersistAllThenIo { .. }
                    | FaultAction::SyncPersistPrefixThenIo { .. }
            )
        ) {
            return Err(Error::invalid_input(
                "sync persistence fault used for set_len",
            ));
        }
        let length = usize::try_from(length)
            .map_err(|_| Error::invalid_input("file length does not fit platform usize"))?;
        self.volatile.resize(length, 0);
        Ok(())
    }

    fn sync_data(&mut self) -> Result<()> {
        let action = self.fault(FileOperation::SyncData)?;
        match action {
            Some(FaultAction::Io { kind, message }) => {
                return Err(Error::Io(std::io::Error::new(kind, message)));
            }
            Some(FaultAction::SyncPersistAllThenIo { kind, message }) => {
                self.durable = self.volatile.clone();
                return Err(Error::Io(std::io::Error::new(kind, message)));
            }
            Some(FaultAction::SyncPersistPrefixThenIo {
                length,
                kind,
                message,
            }) => {
                if length > self.volatile.len() {
                    return Err(Error::invalid_input(format!(
                        "sync persistence prefix {length} exceeds volatile file length {}",
                        self.volatile.len()
                    )));
                }
                self.durable = self.volatile[..length].to_vec();
                return Err(Error::Io(std::io::Error::new(kind, message)));
            }
            Some(FaultAction::ShortRead { .. } | FaultAction::ShortWrite { .. }) | None => {}
        }
        self.durable = self.volatile.clone();
        Ok(())
    }

    fn sync_all(&mut self) -> Result<()> {
        let action = self.fault(FileOperation::SyncAll)?;
        match action {
            Some(FaultAction::Io { kind, message }) => {
                return Err(Error::Io(std::io::Error::new(kind, message)));
            }
            Some(FaultAction::SyncPersistAllThenIo { kind, message }) => {
                self.durable = self.volatile.clone();
                return Err(Error::Io(std::io::Error::new(kind, message)));
            }
            Some(FaultAction::SyncPersistPrefixThenIo {
                length,
                kind,
                message,
            }) => {
                if length > self.volatile.len() {
                    return Err(Error::invalid_input(format!(
                        "sync persistence prefix {length} exceeds volatile file length {}",
                        self.volatile.len()
                    )));
                }
                self.durable = self.volatile[..length].to_vec();
                return Err(Error::Io(std::io::Error::new(kind, message)));
            }
            Some(FaultAction::ShortRead { .. } | FaultAction::ShortWrite { .. }) | None => {}
        }
        self.durable = self.volatile.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crash_discards_unsynced_writes() {
        let mut file = CrashableFile::new();
        assert_eq!(file.write_at(0, b"new").unwrap(), 3);
        file.crash();
        assert!(file.volatile_bytes().is_empty());
    }

    #[test]
    fn sync_makes_writes_survive_crash() {
        let mut file = CrashableFile::new();
        file.write_at(0, b"old").unwrap();
        file.sync_data().unwrap();
        file.write_at(0, b"new").unwrap();
        file.crash();
        assert_eq!(file.volatile_bytes(), b"old");
        assert_eq!(file.durable_bytes(), b"old");
    }

    #[test]
    fn crash_can_persist_a_prefix_of_unsynced_writes() {
        let mut file = CrashableFile::new();
        file.write_at(0, b"abcdef").unwrap();
        file.crash_with_persisted_prefix(3);
        assert_eq!(file.durable_bytes(), b"abc");
        assert_eq!(file.volatile_bytes(), b"abc");
    }

    #[test]
    fn deterministic_short_write_and_sync_failure() {
        let plan = FaultPlan::default()
            .at(0, FileOperation::WriteAt, FaultAction::short_write(2))
            .at(
                1,
                FileOperation::SyncData,
                FaultAction::io(ErrorKind::Other, "fsync"),
            );
        let mut file = CrashableFile::new().with_fault_plan(plan);
        assert_eq!(file.write_at(0, b"abcd").unwrap(), 2);
        assert!(matches!(file.sync_data(), Err(Error::Io(_))));
    }

    #[test]
    fn legacy_shadow_reset_window_loses_state_when_main_file_is_stale() {
        let mut main = CrashableFile::new();
        let mut shadow = CrashableFile::new();
        let mut wal = CrashableFile::new();

        main.write_at(0, b"old").unwrap();
        main.sync_data().unwrap();
        wal.write_at(0, b"committed-page-image").unwrap();
        wal.sync_data().unwrap();
        shadow.write_at(0, b"new").unwrap();
        shadow.sync_data().unwrap();

        // This is the legacy reset point: the shadow is durable, but the main
        // file has not been copied and synced yet.
        wal.set_len(0).unwrap();
        wal.sync_data().unwrap();
        main.crash();
        shadow.crash();
        wal.crash();

        assert_eq!(main.durable_bytes(), b"old");
        assert!(wal.durable_bytes().is_empty());
        assert_eq!(shadow.durable_bytes(), b"new");
    }

    #[test]
    fn sync_error_can_persist_all_bytes() {
        let plan = FaultPlan::default().on_next(
            FileOperation::SyncData,
            FaultAction::SyncPersistAllThenIo {
                kind: ErrorKind::Other,
                message: "fsync after persistence".into(),
            },
        );
        let mut file = CrashableFile::new().with_fault_plan(plan);
        file.write_at(0, b"whole image").unwrap();
        assert!(file.sync_data().is_err());
        assert_eq!(file.durable_bytes(), b"whole image");
    }

    #[test]
    fn sync_error_can_persist_a_prefix() {
        let plan = FaultPlan::default().on_next(
            FileOperation::SyncData,
            FaultAction::SyncPersistPrefixThenIo {
                length: 4,
                kind: ErrorKind::Other,
                message: "fsync after prefix persistence".into(),
            },
        );
        let mut file = CrashableFile::new().with_fault_plan(plan);
        file.write_at(0, b"whole image").unwrap();
        assert!(file.sync_data().is_err());
        assert_eq!(file.durable_bytes(), b"whol");
    }

    #[test]
    fn sync_persistence_faults_reject_invalid_targets_and_lengths() {
        let invalid_write_fault = FaultPlan::default().on_next(
            FileOperation::WriteAt,
            FaultAction::SyncPersistAllThenIo {
                kind: ErrorKind::Other,
                message: "misconfigured target".into(),
            },
        );
        let mut write_file = CrashableFile::new().with_fault_plan(invalid_write_fault);
        assert!(matches!(
            write_file.write_at(0, b"bytes"),
            Err(Error::InvalidInput(_))
        ));

        let oversized_prefix_fault = FaultPlan::default().on_next(
            FileOperation::SyncData,
            FaultAction::SyncPersistPrefixThenIo {
                length: 6,
                kind: ErrorKind::Other,
                message: "prefix exceeds image".into(),
            },
        );
        let mut sync_file = CrashableFile::new().with_fault_plan(oversized_prefix_fault);
        sync_file.write_at(0, b"short").unwrap();
        assert!(matches!(sync_file.sync_data(), Err(Error::InvalidInput(_))));
        assert!(sync_file.durable_bytes().is_empty());
    }
}
