//! The on-disk form of a histogram series' observations.
//!
//! A histogram used to reach storage as `bounds + 3` scalar series, each a
//! Gorilla stream. Holding the instrument whole removes sixty-six identities
//! from the index and the catalogs — but only if the bytes do not come back
//! the other way. Counts written plainly cost eight bytes a bucket a point,
//! which at sixty-four buckets is five times what the fan-out paid, so the
//! encoding is the point of this module rather than an afterthought.
//!
//! Two properties do the work. Cumulative-by-boundary counts rise slowly and
//! monotonically, so a delta against the previous point is a small number; and
//! a series' boundary schema repeats, so it is written once for a run of
//! points rather than once per point. A chunk is a sequence of such runs,
//! because an exponential histogram that rescales keeps its series and changes
//! its schema — the case that used to mint a fresh set of sixty-seven series.

use crate::series::HistogramPoint;

/// LEB128, the encoding every varint format converges on.
fn put_uvarint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn get_uvarint(bytes: &[u8], at: &mut usize) -> Result<u64, String> {
    let mut value = 0u64;
    let mut shift = 0u32;
    loop {
        let byte = *bytes.get(*at).ok_or("histogram chunk is truncated")?;
        *at += 1;
        if shift >= 64 {
            return Err("histogram chunk has an overlong varint".to_string());
        }
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
    }
}

/// Zigzag, so a small decrease costs as little as a small increase. Counts
/// only fall on a reset, but they do fall.
fn zigzag(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

fn unzigzag(value: u64) -> i64 {
    ((value >> 1) as i64) ^ -((value & 1) as i64)
}

fn put_ivarint(out: &mut Vec<u8>, value: i64) {
    put_uvarint(out, zigzag(value));
}

fn get_ivarint(bytes: &[u8], at: &mut usize) -> Result<i64, String> {
    Ok(unzigzag(get_uvarint(bytes, at)?))
}

fn take<'a>(bytes: &'a [u8], at: &mut usize, len: usize) -> Result<&'a [u8], String> {
    let end = at
        .checked_add(len)
        .filter(|end| *end <= bytes.len())
        .ok_or("histogram chunk is truncated")?;
    let slice = &bytes[*at..end];
    *at = end;
    Ok(slice)
}

/// Encode a series' points, in the order they arrived.
///
/// Points are grouped into runs of one boundary schema. Within a run every
/// number is a delta against the point before it, which is where the size
/// comes from: a bucket that saw nothing since the last scrape costs one byte.
pub fn encode(points: &[(i64, HistogramPoint)]) -> Vec<u8> {
    let mut runs: Vec<&[(i64, HistogramPoint)]> = Vec::new();
    let mut start = 0usize;
    for index in 1..points.len() {
        if points[index].1.bounds != points[start].1.bounds {
            runs.push(&points[start..index]);
            start = index;
        }
    }
    if !points.is_empty() {
        runs.push(&points[start..]);
    }

    let mut out = Vec::new();
    put_uvarint(&mut out, runs.len() as u64);
    for run in runs {
        let bounds = &run[0].1.bounds;
        put_uvarint(&mut out, bounds.len() as u64);
        for bound in bounds.iter() {
            out.extend_from_slice(&bound.to_le_bytes());
        }
        put_uvarint(&mut out, run.len() as u64);

        let mut previous_ts = 0i64;
        let mut previous_count = 0i64;
        let mut previous_cumulative = vec![0i64; bounds.len()];
        for (ts_ns, point) in run {
            put_ivarint(&mut out, ts_ns - previous_ts);
            previous_ts = *ts_ns;
            put_ivarint(&mut out, point.count as i64 - previous_count);
            previous_count = point.count as i64;
            match point.sum {
                Some(sum) => {
                    out.push(1);
                    out.extend_from_slice(&sum.to_le_bytes());
                }
                None => out.push(0),
            }
            for (index, value) in point.cumulative.iter().enumerate() {
                let previous = previous_cumulative.get(index).copied().unwrap_or(0);
                put_ivarint(&mut out, *value as i64 - previous);
                if let Some(slot) = previous_cumulative.get_mut(index) {
                    *slot = *value as i64;
                }
            }
        }
    }
    out
}

/// Decode what [`encode`] wrote. A chunk crossed a checksum, so a failure
/// here is this module disagreeing with itself rather than a corrupt file.
pub fn decode(bytes: &[u8]) -> Result<Vec<(i64, HistogramPoint)>, String> {
    let mut at = 0usize;
    let runs = get_uvarint(bytes, &mut at)?;
    let mut points = Vec::new();
    for _ in 0..runs {
        let bound_count = usize::try_from(get_uvarint(bytes, &mut at)?)
            .map_err(|_| "histogram chunk declares more bounds than fit in memory")?;
        let mut bounds = Vec::with_capacity(bound_count);
        for _ in 0..bound_count {
            let raw = take(bytes, &mut at, 8)?;
            bounds.push(f64::from_le_bytes(raw.try_into().expect("eight bytes")));
        }
        let bounds: std::sync::Arc<[f64]> = bounds.into();
        let point_count = usize::try_from(get_uvarint(bytes, &mut at)?)
            .map_err(|_| "histogram chunk declares more points than fit in memory")?;

        let mut ts_ns = 0i64;
        let mut count = 0i64;
        let mut cumulative = vec![0i64; bound_count];
        for _ in 0..point_count {
            ts_ns += get_ivarint(bytes, &mut at)?;
            count += get_ivarint(bytes, &mut at)?;
            let sum = match *take(bytes, &mut at, 1)?.first().expect("one byte") {
                0 => None,
                1 => {
                    let raw = take(bytes, &mut at, 8)?;
                    Some(f64::from_le_bytes(raw.try_into().expect("eight bytes")))
                }
                other => return Err(format!("histogram point has sum flag {other}")),
            };
            for slot in cumulative.iter_mut() {
                *slot += get_ivarint(bytes, &mut at)?;
            }
            points.push((
                ts_ns,
                HistogramPoint {
                    bounds: bounds.clone(),
                    cumulative: cumulative.iter().map(|value| *value as u64).collect(),
                    sum,
                    count: count as u64,
                },
            ));
        }
    }
    if at != bytes.len() {
        return Err("histogram chunk has trailing bytes".to_string());
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(bounds: &std::sync::Arc<[f64]>, cumulative: &[u64], count: u64) -> HistogramPoint {
        HistogramPoint {
            bounds: bounds.clone(),
            cumulative: cumulative.to_vec(),
            sum: Some(count as f64 * 0.5),
            count,
        }
    }

    #[test]
    fn a_run_round_trips_with_its_schema_and_totals() {
        let bounds: std::sync::Arc<[f64]> = vec![0.005, 0.01, 0.025].into();
        let points = vec![
            (100, point(&bounds, &[1, 2, 3], 4)),
            (200, point(&bounds, &[3, 5, 8], 9)),
        ];
        assert_eq!(decode(&encode(&points)).unwrap(), points);
    }

    #[test]
    fn a_rescale_starts_a_new_run_inside_the_same_chunk() {
        let first: std::sync::Arc<[f64]> = vec![0.005, 0.01].into();
        let second: std::sync::Arc<[f64]> = vec![0.01, 0.05, 0.1].into();
        let points = vec![
            (100, point(&first, &[1, 2], 3)),
            (200, point(&first, &[2, 4], 5)),
            // The observed range widened: the same series, different bounds.
            (300, point(&second, &[1, 2, 3], 4)),
        ];
        let decoded = decode(&encode(&points)).unwrap();
        assert_eq!(decoded, points);
        assert_eq!(&*decoded[2].1.bounds, &[0.01, 0.05, 0.1]);
    }

    #[test]
    fn a_missing_sum_survives_the_round_trip() {
        let bounds: std::sync::Arc<[f64]> = vec![1.0].into();
        let points = vec![(
            100,
            HistogramPoint {
                bounds: bounds.clone(),
                cumulative: vec![7],
                sum: None,
                count: 7,
            },
        )];
        assert_eq!(decode(&encode(&points)).unwrap(), points);
    }

    #[test]
    fn a_counter_reset_costs_no_more_than_the_increase_did() {
        let bounds: std::sync::Arc<[f64]> = vec![1.0, 2.0].into();
        let points = vec![
            (100, point(&bounds, &[500, 900], 1000)),
            // The process restarted: every bucket falls back to nothing.
            (200, point(&bounds, &[1, 2], 3)),
        ];
        assert_eq!(decode(&encode(&points)).unwrap(), points);
    }

    #[test]
    fn a_slow_counter_costs_about_a_byte_a_bucket_a_point() {
        let bounds: std::sync::Arc<[f64]> = (0..64).map(|index| index as f64).collect();
        let mut points = Vec::new();
        let mut running = vec![0u64; 64];
        for step in 0..100i64 {
            for (index, slot) in running.iter_mut().enumerate() {
                *slot += (index as u64 % 3) + 1;
            }
            points.push((
                1_772_000_000_000_000_000 + step * 15_000_000_000,
                point(&bounds, &running, running[63] + 1),
            ));
        }
        let encoded = encode(&points);
        assert_eq!(decode(&encoded).unwrap(), points);
        // Sixty-four buckets stored plainly would be 512 bytes a point before
        // the timestamp and the totals. The fan-out this replaces paid Gorilla
        // for sixty-seven separate series.
        let per_point = (encoded.len() - 64 * 8) / points.len();
        assert!(
            per_point < 128,
            "a hundred slow scrapes cost {per_point} bytes a point"
        );
        assert!(
            per_point > 64,
            "sixty-four buckets cannot cost less than a byte each: {per_point}"
        );
    }

    #[test]
    fn a_truncated_chunk_is_an_error_and_not_a_panic() {
        let bounds: std::sync::Arc<[f64]> = vec![1.0].into();
        let encoded = encode(&[(100, point(&bounds, &[1], 1))]);
        for cut in 0..encoded.len() {
            assert!(decode(&encoded[..cut]).is_err(), "prefix of {cut} bytes");
        }
    }
}
