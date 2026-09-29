use std::hint::black_box;
use std::time::Instant;

const PAYLOAD_SIZE: usize = 4104;
const ITERATIONS: usize = 250_000;
const REPETITIONS: usize = 7;

#[inline(never)]
fn measure_default(payload: &[u8]) -> (f64, u32) {
    let started = Instant::now();
    let mut result = 0u32;
    for iteration in 0..ITERATIONS {
        result ^= crc32c::crc32c(black_box(payload)).rotate_left((iteration & 31) as u32);
    }
    black_box(result);
    (started.elapsed().as_nanos() as f64 / ITERATIONS as f64, result)
}

#[target_feature(enable = "crc")]
#[inline(never)]
unsafe fn measure_specialized(payload: &[u8]) -> (f64, u32) {
    let started = Instant::now();
    let mut result = 0u32;
    for iteration in 0..ITERATIONS {
        result ^= crc32c::crc32c(black_box(payload)).rotate_left((iteration & 31) as u32);
    }
    black_box(result);
    (started.elapsed().as_nanos() as f64 / ITERATIONS as f64, result)
}

#[inline(never)]
fn measure_portable(payload: &[u8]) -> (f64, u32) {
    if std::arch::is_aarch64_feature_detected!("crc") {
        unsafe { measure_specialized(payload) }
    } else {
        measure_default(payload)
    }
}

#[inline(never)]
fn measure_global(payload: &[u8]) -> (f64, u32) {
    measure_default(payload)
}

fn report(name: &str, payload: &[u8], measure: fn(&[u8]) -> (f64, u32)) -> u32 {
    let mut values = Vec::with_capacity(REPETITIONS);
    let mut expected = None;
    for _ in 0..REPETITIONS {
        let (nanos, checksum) = measure(payload);
        if let Some(expected) = expected {
            assert_eq!(checksum, expected);
        } else {
            expected = Some(checksum);
        }
        values.push(nanos);
    }
    values.sort_by(f64::total_cmp);
    let median = values[values.len() / 2];
    let gib_per_second = PAYLOAD_SIZE as f64 / median * 1_000_000_000.0 / 1024.0_f64.powi(3);
    println!("{name}: {median:.1} ns/payload {gib_per_second:.3} GiB/s checksum={:#010x} samples={values:?}", expected.unwrap());
    expected.unwrap()
}

fn main() {
    let payload: Vec<u8> = (0..PAYLOAD_SIZE)
        .map(|offset| (offset.wrapping_mul(131).wrapping_add(17) & 0xff) as u8)
        .collect();
    println!("crc_feature={}", std::arch::is_aarch64_feature_detected!("crc"));
    if cfg!(target_feature = "crc") {
        report("global_plus_crc", &payload, measure_global);
    } else {
        let generic = report("default", &payload, measure_default);
        let portable = report("portable_specialized", &payload, measure_portable);
        assert_eq!(generic, portable);
    }
}
