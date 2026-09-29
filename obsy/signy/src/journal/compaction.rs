async fn compact_wal(
    file: &mut tokio::fs::File,
    wal_path: &Path,
    ckpt_path: &Path,
    offset: u64,
    good_len: &mut u64,
) -> Result<(), IoError> {
    let state_path = wal_path.with_file_name(COMPACTION_STATE_FILE);
    let compaction_state = read_compaction_state(&state_path)?;
    if let Some(state) = compaction_state {
        {
            let wal_len = std::fs::metadata(wal_path)?.len();
            let tmp_path = wal_path.with_extension("wal.compact.tmp");
            if tmp_path.exists() {
                // Rename did not happen. The checkpoint may already be zero,
                // so restore the old WAL/checkpoint pair before retrying. New
                // appends are included in the next source length.
                if wal_len < state.source_len {
                    return Err(IoError::new(
                        std::io::ErrorKind::InvalidData,
                        "source WAL is shorter than its recorded compaction length",
                    ));
                }
                write_checkpoint(ckpt_path, state.offset)?;
                std::fs::remove_file(&tmp_path)?;
                std::fs::remove_file(&state_path)?;
                if let Some(parent) = wal_path.parent() {
                    std::fs::File::open(parent)?.sync_all()?;
                }
                *good_len = wal_len;
            } else {
                // Rename happened; only the directory durability step (or a
                // later state update) failed. Never apply the old offset to
                // this replacement WAL a second time.
                if wal_len < state.retained_len {
                    return Err(IoError::new(
                        std::io::ErrorKind::InvalidData,
                        "replacement WAL is shorter than its recorded suffix",
                    ));
                }
                sync_wal_parent(wal_path)?;
                remove_compaction_state(&state_path, wal_path)?;
                *file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(wal_path)
                    .await?;
                *good_len = file.metadata().await?.len();
                return Ok(());
            }
        }
    }
    if offset > *good_len {
        return Err(IoError::new(
            std::io::ErrorKind::InvalidInput,
            format!("WAL compaction offset {offset} exceeds durable length {good_len}"),
        ));
    }
    file.sync_all().await?;
    let retained_len = *good_len - offset;
    let tmp_path = wal_path.with_extension("wal.compact.tmp");
    let mut source = OpenOptions::new().read(true).open(wal_path).await?;
    source.seek(SeekFrom::Start(offset)).await?;
    let mut tmp = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&tmp_path)
        .await?;
    let copied = tokio::io::copy(&mut source.take(retained_len), &mut tmp).await?;
    if copied != retained_len {
        return Err(IoError::new(
            std::io::ErrorKind::UnexpectedEof,
            format!("copied {copied} of {retained_len} WAL bytes during compaction"),
        ));
    }
    tmp.flush().await?;
    tmp.sync_all().await?;
    drop(tmp);

    // This durable intent record makes compaction idempotent across the
    // rename/fsync boundary. A retry observes it and keeps the current WAL
    // intact instead of interpreting the suffix length as the old offset.
    let state = CompactionState {
        offset,
        source_len: *good_len,
        retained_len,
    };
    write_compaction_state(&state_path, &state)?;

    // Resetting the checkpoint before replacing the WAL makes every crash
    // point at-least-once safe: a crash before rename replays the old WAL,
    // while a crash after rename replays only the retained suffix.
    write_checkpoint(ckpt_path, 0)?;
    tokio::fs::rename(&tmp_path, wal_path).await?;
    sync_wal_parent(wal_path)?;
    *file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(wal_path)
        .await?;
    *good_len = retained_len;
    // Retire the intent durably. Leaving it behind would make the next
    // compaction compare its offset against this one's, and the two are in
    // different coordinate systems from the moment the checkpoint reset above
    // landed.
    remove_compaction_state(&state_path, wal_path)?;
    Ok(())
}

/// Remove the intent record and make the removal durable.
///
/// The phase-2 marker it replaces existed only to say "the rename committed",
/// which is exactly what the record's absence says once the removal is durable.
fn remove_compaction_state(state_path: &Path, wal_path: &Path) -> Result<(), IoError> {
    #[cfg(test)]
    if take_compaction_fault(wal_path, CompactionFault::BeforeStateRemoval) {
        return Err(IoError::other("injected compaction state removal failure"));
    }
    match std::fs::remove_file(state_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    }
    if let Some(parent) = wal_path.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct CompactionState {
    offset: u64,
    source_len: u64,
    retained_len: u64,
}

fn read_compaction_state(path: &Path) -> Result<Option<CompactionState>, IoError> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    match bytes.as_slice() {
        offset if offset.len() == 24 => Ok(Some(CompactionState {
            offset: u64::from_le_bytes(offset[0..8].try_into().unwrap()),
            source_len: u64::from_le_bytes(offset[8..16].try_into().unwrap()),
            retained_len: u64::from_le_bytes(offset[16..24].try_into().unwrap()),
        })),
        _ => Err(IoError::new(
            std::io::ErrorKind::InvalidData,
            "invalid WAL compaction state",
        )),
    }
}

/// The record's presence means a compaction is in flight; its absence means
/// none is. There is no completion record — a finished compaction removes this
/// one, and comparing an offset written before the checkpoint reset against one
/// written after it is what wedged the flush loop when there was.
fn write_compaction_state(path: &Path, state: &CompactionState) -> Result<(), IoError> {
    let mut bytes = Vec::with_capacity(24);
    bytes.extend_from_slice(&state.offset.to_le_bytes());
    bytes.extend_from_slice(&state.source_len.to_le_bytes());
    bytes.extend_from_slice(&state.retained_len.to_le_bytes());
    let tmp = path.with_extension("compact.state.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::File::open(&tmp)?.sync_all()?;
    std::fs::rename(&tmp, path)?;
    if let Some(parent) = path.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn sync_wal_parent(wal_path: &Path) -> Result<(), IoError> {
    #[cfg(test)]
    if take_compaction_fault(wal_path, CompactionFault::AfterRename) {
        return Err(IoError::other("injected WAL directory fsync failure"));
    }
    if let Some(parent) = wal_path.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}


