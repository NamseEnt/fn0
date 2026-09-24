use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use dodb_core::Result;

/// Minimal positional file contract shared by production and simulated files.
///
/// `read_at` and `write_at` may return short counts. Callers that require a
/// complete operation must loop and handle a zero count. Writes are volatile
/// until `sync_data` or `sync_all` succeeds.
pub trait DurableFile {
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize>;
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize>;
    fn len(&self) -> Result<u64>;
    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
    fn set_len(&mut self, length: u64) -> Result<()>;
    fn sync_data(&mut self) -> Result<()>;
    fn sync_all(&mut self) -> Result<()>;
}

/// Thin production adapter. Database layout and higher-level I/O are
/// intentionally absent in Phase 0.
pub struct ProductionFile {
    file: File,
}

impl ProductionFile {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn from_file(file: File) -> Self {
        Self { file }
    }

    pub fn into_file(self) -> File {
        self.file
    }
}

impl DurableFile for ProductionFile {
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.file.seek(SeekFrom::Start(offset))?;
        Ok(self.file.read(buffer)?)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
        self.file.seek(SeekFrom::Start(offset))?;
        Ok(self.file.write(bytes)?)
    }

    fn len(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }

    fn set_len(&mut self, length: u64) -> Result<()> {
        self.file.set_len(length)?;
        Ok(())
    }

    fn sync_data(&mut self) -> Result<()> {
        self.file.sync_data()?;
        Ok(())
    }

    fn sync_all(&mut self) -> Result<()> {
        self.file.sync_all()?;
        Ok(())
    }
}
