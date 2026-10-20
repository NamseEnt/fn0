use std::ffi::{CStr, CString, c_char, c_int, c_uchar, c_void};
use std::path::Path;
use std::ptr;
use std::ptr::NonNull;
use std::sync::Arc;

use dodb_core::Error;

unsafe extern "C" {
    fn dodb_raw_rocksdb_open(path: *const c_char, error: *mut *mut c_char) -> *mut c_void;
    fn dodb_raw_rocksdb_close(database: *mut c_void);
    fn dodb_raw_rocksdb_write_batch(
        database: *mut c_void,
        keys: *const *const c_uchar,
        key_lengths: *const usize,
        values: *const *const c_uchar,
        value_lengths: *const usize,
        count: usize,
        error: *mut *mut c_char,
    ) -> c_int;
    fn dodb_raw_rocksdb_iterator_new(database: *mut c_void, error: *mut *mut c_char)
    -> *mut c_void;
    fn dodb_raw_rocksdb_iterator_seek(
        iterator: *mut c_void,
        key: *const c_uchar,
        key_length: usize,
    );
    fn dodb_raw_rocksdb_iterator_valid(iterator: *mut c_void) -> c_int;
    fn dodb_raw_rocksdb_iterator_key(
        iterator: *mut c_void,
        key_length: *mut usize,
    ) -> *const c_uchar;
    fn dodb_raw_rocksdb_iterator_value(
        iterator: *mut c_void,
        value_length: *mut usize,
    ) -> *const c_uchar;
    fn dodb_raw_rocksdb_iterator_status(iterator: *mut c_void, error: *mut *mut c_char) -> c_int;
    fn dodb_raw_rocksdb_iterator_next(iterator: *mut c_void);
    fn dodb_raw_rocksdb_iterator_destroy(iterator: *mut c_void);
    fn dodb_raw_rocksdb_free_error(error: *mut c_char);
}

pub struct RawRocksDb {
    database: *mut c_void,
    health_handle: Arc<RawRocksDbHealthHandle>,
}

struct RawRocksDbHealthHandle(NonNull<c_void>);

unsafe impl Send for RawRocksDbHealthHandle {}
unsafe impl Sync for RawRocksDbHealthHandle {}

impl super::engine_health::EngineHealthDatabase for RawRocksDbHealthHandle {
    fn database_pointer(&self) -> *mut c_void {
        self.0.as_ptr()
    }

    fn database_kind(&self) -> super::engine_health::EngineHealthDatabaseKind {
        super::engine_health::EngineHealthDatabaseKind::Db
    }
}

impl Drop for RawRocksDbHealthHandle {
    fn drop(&mut self) {
        unsafe { dodb_raw_rocksdb_close(self.0.as_ptr()) };
    }
}

unsafe impl Send for RawRocksDb {}
unsafe impl Sync for RawRocksDb {}

impl RawRocksDb {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let path = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|_| Error::invalid_input("raw RocksDB path contains a zero byte"))?;
        let mut error = ptr::null_mut();
        let database = unsafe { dodb_raw_rocksdb_open(path.as_ptr(), &mut error) };
        if database.is_null() {
            return Err(take_error(error));
        }
        let health_handle = NonNull::new(database)
            .ok_or_else(|| Error::invariant("raw RocksDB open returned a null database"))?;
        Ok(Self {
            database,
            health_handle: Arc::new(RawRocksDbHealthHandle(health_handle)),
        })
    }

    pub fn engine_health_observer(&self) -> super::EngineHealthObserver {
        let mut observer = super::EngineHealthObserver::new();
        observer.add_database("raw_database".to_owned(), &self.health_handle);
        observer
    }

    pub fn write_batch(&self, entries: &[(Vec<u8>, Vec<u8>)]) -> Result<(), Error> {
        let keys = entries
            .iter()
            .map(|(key, _)| key.as_ptr())
            .collect::<Vec<_>>();
        let key_lengths = entries.iter().map(|(key, _)| key.len()).collect::<Vec<_>>();
        let values = entries
            .iter()
            .map(|(_, value)| value.as_ptr())
            .collect::<Vec<_>>();
        let value_lengths = entries
            .iter()
            .map(|(_, value)| value.len())
            .collect::<Vec<_>>();
        let mut error = ptr::null_mut();
        let status = unsafe {
            dodb_raw_rocksdb_write_batch(
                self.database,
                keys.as_ptr(),
                key_lengths.as_ptr(),
                values.as_ptr(),
                value_lengths.as_ptr(),
                entries.len(),
                &mut error,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(take_error(error))
        }
    }

    pub fn scan(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error> {
        let mut error = ptr::null_mut();
        let iterator = unsafe { dodb_raw_rocksdb_iterator_new(self.database, &mut error) };
        if iterator.is_null() {
            return Err(take_error(error));
        }
        let result = unsafe { self.scan_iterator(iterator) };
        unsafe { dodb_raw_rocksdb_iterator_destroy(iterator) };
        result
    }

    pub fn scan_page(
        &self,
        exclusive_after_key: Option<&[u8]>,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut error = ptr::null_mut();
        let iterator = unsafe { dodb_raw_rocksdb_iterator_new(self.database, &mut error) };
        if iterator.is_null() {
            return Err(take_error(error));
        }
        if let Some(cursor_key) = exclusive_after_key {
            unsafe {
                dodb_raw_rocksdb_iterator_seek(iterator, cursor_key.as_ptr(), cursor_key.len())
            };
        }
        let result = unsafe { self.scan_iterator_page(iterator, exclusive_after_key, limit) };
        unsafe { dodb_raw_rocksdb_iterator_destroy(iterator) };
        result
    }

    unsafe fn scan_iterator_page(
        &self,
        iterator: *mut c_void,
        exclusive_after_key: Option<&[u8]>,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error> {
        let mut entries = Vec::with_capacity(limit.min(1_024));
        while unsafe { dodb_raw_rocksdb_iterator_valid(iterator) } != 0 {
            let mut key_length = 0;
            let key_pointer = unsafe { dodb_raw_rocksdb_iterator_key(iterator, &mut key_length) };
            let key = unsafe { copy_iterator_bytes(key_pointer, key_length) }?;
            if exclusive_after_key.is_some_and(|cursor_key| key.as_slice() <= cursor_key) {
                unsafe { dodb_raw_rocksdb_iterator_next(iterator) };
                continue;
            }
            let mut value_length = 0;
            let value_pointer =
                unsafe { dodb_raw_rocksdb_iterator_value(iterator, &mut value_length) };
            let value = unsafe { copy_iterator_bytes(value_pointer, value_length) }?;
            entries.push((key, value));
            if entries.len() == limit {
                return Ok(entries);
            }
            unsafe { dodb_raw_rocksdb_iterator_next(iterator) };
        }
        let mut error = ptr::null_mut();
        if unsafe { dodb_raw_rocksdb_iterator_status(iterator, &mut error) } == 0 {
            Ok(entries)
        } else {
            Err(take_error(error))
        }
    }

    unsafe fn scan_iterator(
        &self,
        iterator: *mut c_void,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error> {
        let mut entries = Vec::new();
        while unsafe { dodb_raw_rocksdb_iterator_valid(iterator) } != 0 {
            let mut key_length = 0;
            let key_pointer = unsafe { dodb_raw_rocksdb_iterator_key(iterator, &mut key_length) };
            let mut value_length = 0;
            let value_pointer =
                unsafe { dodb_raw_rocksdb_iterator_value(iterator, &mut value_length) };
            let key = unsafe { copy_iterator_bytes(key_pointer, key_length) }?;
            let value = unsafe { copy_iterator_bytes(value_pointer, value_length) }?;
            entries.push((key, value));
            unsafe { dodb_raw_rocksdb_iterator_next(iterator) };
        }
        let mut error = ptr::null_mut();
        if unsafe { dodb_raw_rocksdb_iterator_status(iterator, &mut error) } == 0 {
            Ok(entries)
        } else {
            Err(take_error(error))
        }
    }
}

unsafe fn copy_iterator_bytes(pointer: *const c_uchar, length: usize) -> Result<Vec<u8>, Error> {
    if length == 0 {
        return Ok(Vec::new());
    }
    if pointer.is_null() {
        return Err(Error::invariant(
            "raw RocksDB iterator returned a null nonempty slice",
        ));
    }
    Ok(unsafe { std::slice::from_raw_parts(pointer, length) }.to_vec())
}

fn take_error(error: *mut c_char) -> Error {
    if error.is_null() {
        return Error::invariant("raw RocksDB shim returned no error text");
    }
    let message = unsafe { CStr::from_ptr(error) }
        .to_string_lossy()
        .into_owned();
    unsafe { dodb_raw_rocksdb_free_error(error) };
    Error::durability(message)
}
