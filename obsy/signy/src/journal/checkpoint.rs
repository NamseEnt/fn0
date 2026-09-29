pub fn read_checkpoint(ckpt_path: &Path) -> Result<u64, IoError> {
    match std::fs::read(ckpt_path) {
        Ok(bytes) => {
            if bytes.len() != 8 {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "journal checkpoint must be exactly 8 bytes, got {}",
                        bytes.len()
                    ),
                ));
            }
            Ok(u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(e),
    }
}

pub fn write_checkpoint(ckpt_path: &Path, offset: u64) -> Result<(), IoError> {
    let tmp = ckpt_path.with_extension("ckpt.tmp");
    std::fs::write(&tmp, offset.to_le_bytes())?;
    let tmp_file = std::fs::File::open(&tmp)?;
    tmp_file.sync_all()?;
    std::fs::rename(&tmp, ckpt_path)?;
    if let Some(parent) = ckpt_path.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}
