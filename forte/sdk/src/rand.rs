use crate::bindings::wasi::random;

pub fn fill_bytes(buf: &mut [u8]) {
    let bytes = random::random::get_random_bytes(buf.len() as u64);
    buf.copy_from_slice(&bytes);
}

pub fn get_random_bytes(len: usize) -> Vec<u8> {
    random::random::get_random_bytes(len as u64)
}

pub fn get_random_u64() -> u64 {
    random::random::get_random_u64()
}

pub fn get_insecure_random_bytes(buf: &mut [u8]) {
    let bytes = random::insecure::get_insecure_random_bytes(buf.len() as u64);
    buf.copy_from_slice(&bytes);
}

pub fn get_insecure_random_u64() -> u64 {
    random::insecure::get_insecure_random_u64()
}
