use std::io::{self, Read};
use std::path::Path;

const IDENTITY_FILE: &str = "identity";
const SENDER_ID_BYTES: usize = 16;
const IDENTITY_BYTES: usize = SENDER_ID_BYTES + 4;

/// Which collecty this queue belongs to.
///
/// Random, and made when the queue directory is, so a lost volume produces a
/// new one. An operator-set name would survive the volume it names: the queue
/// would restart its segment numbering while signy still remembered the old
/// high-water mark, and everything under it would be dropped as already-seen.
///
/// This is the whole of what the queue writes down about itself. How far signy
/// has got is not recorded, because the segment files say it: one that has
/// been answered for is unlinked, so whatever is still on disk is still owed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SenderId([u8; SENDER_ID_BYTES]);

impl SenderId {
    pub fn generate() -> io::Result<SenderId> {
        let mut bytes = [0u8; SENDER_ID_BYTES];
        std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
        Ok(SenderId(bytes))
    }
}

impl std::fmt::Display for SenderId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for SenderId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self}")
    }
}

pub fn load(dir: &Path) -> io::Result<Option<SenderId>> {
    let mut file = match std::fs::File::open(dir.join(IDENTITY_FILE)) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::with_capacity(IDENTITY_BYTES);
    file.read_to_end(&mut bytes)?;
    if bytes.len() != IDENTITY_BYTES {
        return Ok(None);
    }
    let stored = u32::from_le_bytes(bytes[SENDER_ID_BYTES..].try_into().expect("four bytes"));
    if crc32fast::hash(&bytes[0..SENDER_ID_BYTES]) != stored {
        return Ok(None);
    }
    let mut id = [0u8; SENDER_ID_BYTES];
    id.copy_from_slice(&bytes[0..SENDER_ID_BYTES]);
    Ok(Some(SenderId(id)))
}

pub fn store(dir: &Path, sender: SenderId) -> io::Result<()> {
    let mut bytes = [0u8; IDENTITY_BYTES];
    bytes[0..SENDER_ID_BYTES].copy_from_slice(&sender.0);
    let crc = crc32fast::hash(&bytes[0..SENDER_ID_BYTES]);
    bytes[SENDER_ID_BYTES..].copy_from_slice(&crc.to_le_bytes());

    let path = dir.join(IDENTITY_FILE);
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::File::open(&tmp)?.sync_all()?;
    std::fs::rename(&tmp, &path)?;
    std::fs::File::open(dir)?.sync_all()
}
