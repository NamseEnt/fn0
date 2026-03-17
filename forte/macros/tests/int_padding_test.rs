fn format_u8(v: u8) -> String {
    format!("{:03}", v)
}
fn format_u16(v: u16) -> String {
    format!("{:05}", v)
}
fn format_u32(v: u32) -> String {
    format!("{:010}", v)
}
fn format_u64(v: u64) -> String {
    format!("{:020}", v)
}
fn format_usize(v: usize) -> String {
    format!("{:020}", v)
}

fn format_i8(v: i8) -> String {
    format!("{:03}", (v as u8).wrapping_add(128u8))
}
fn format_i16(v: i16) -> String {
    format!("{:05}", (v as u16).wrapping_add(32768u16))
}
fn format_i32(v: i32) -> String {
    format!("{:010}", (v as u32).wrapping_add(2147483648u32))
}
fn format_i64(v: i64) -> String {
    format!("{:020}", (v as u64).wrapping_add(9223372036854775808u64))
}

#[test]
fn test_u8_padding() {
    assert_eq!(format_u8(0), "000");
    assert_eq!(format_u8(1), "001");
    assert_eq!(format_u8(42), "042");
    assert_eq!(format_u8(255), "255");
    assert!(format_u8(0) < format_u8(1));
    assert!(format_u8(1) < format_u8(255));
}

#[test]
fn test_u16_padding() {
    assert_eq!(format_u16(0), "00000");
    assert_eq!(format_u16(1), "00001");
    assert_eq!(format_u16(1000), "01000");
    assert_eq!(format_u16(65535), "65535");
    assert!(format_u16(0) < format_u16(1));
    assert!(format_u16(1) < format_u16(65535));
}

#[test]
fn test_u32_padding() {
    assert_eq!(format_u32(0), "0000000000");
    assert_eq!(format_u32(1), "0000000001");
    assert_eq!(format_u32(u32::MAX), "4294967295");
    assert!(format_u32(0) < format_u32(1));
    assert!(format_u32(1) < format_u32(u32::MAX));
}

#[test]
fn test_u64_padding() {
    assert_eq!(format_u64(0), "00000000000000000000");
    assert_eq!(format_u64(1), "00000000000000000001");
    assert_eq!(format_u64(u64::MAX), "18446744073709551615");
    assert!(format_u64(0) < format_u64(1));
    assert!(format_u64(1) < format_u64(u64::MAX));
}

#[test]
fn test_usize_padding() {
    assert_eq!(format_usize(0), "00000000000000000000");
    assert_eq!(format_usize(1), "00000000000000000001");
    assert!(format_usize(0) < format_usize(1));
}

#[test]
fn test_i8_offset_boundary() {
    assert_eq!(format_i8(i8::MIN), "000");   // -128 + 128 = 0
    assert_eq!(format_i8(-1), "127");         // -1 + 128 = 127
    assert_eq!(format_i8(0), "128");          // 0 + 128 = 128
    assert_eq!(format_i8(1), "129");          // 1 + 128 = 129
    assert_eq!(format_i8(i8::MAX), "255");    // 127 + 128 = 255
}

#[test]
fn test_i8_sort_order() {
    assert!(format_i8(i8::MIN) < format_i8(-1));
    assert!(format_i8(-1) < format_i8(0));
    assert!(format_i8(0) < format_i8(1));
    assert!(format_i8(1) < format_i8(i8::MAX));

    assert!(format_i8(-100) < format_i8(-50));
    assert!(format_i8(-50) < format_i8(-1));
    assert!(format_i8(-1) < format_i8(0));
    assert!(format_i8(0) < format_i8(50));
    assert!(format_i8(50) < format_i8(100));
}

#[test]
fn test_i16_offset_boundary() {
    assert_eq!(format_i16(i16::MIN), "00000");  // -32768 + 32768 = 0
    assert_eq!(format_i16(-1), "32767");          // -1 + 32768 = 32767
    assert_eq!(format_i16(0), "32768");           // 0 + 32768 = 32768
    assert_eq!(format_i16(1), "32769");           // 1 + 32768 = 32769
    assert_eq!(format_i16(i16::MAX), "65535");    // 32767 + 32768 = 65535
}

#[test]
fn test_i16_sort_order() {
    assert!(format_i16(i16::MIN) < format_i16(-1));
    assert!(format_i16(-1) < format_i16(0));
    assert!(format_i16(0) < format_i16(1));
    assert!(format_i16(1) < format_i16(i16::MAX));

    assert!(format_i16(-10000) < format_i16(-100));
    assert!(format_i16(-100) < format_i16(100));
    assert!(format_i16(100) < format_i16(10000));
}

#[test]
fn test_i32_offset_boundary() {
    assert_eq!(format_i32(i32::MIN), "0000000000");   // -2147483648 + 2147483648 = 0
    assert_eq!(format_i32(-1), "2147483647");
    assert_eq!(format_i32(0), "2147483648");
    assert_eq!(format_i32(1), "2147483649");
    assert_eq!(format_i32(i32::MAX), "4294967295");    // 2147483647 + 2147483648 = 4294967295
}

#[test]
fn test_i32_sort_order() {
    assert!(format_i32(i32::MIN) < format_i32(-1));
    assert!(format_i32(-1) < format_i32(0));
    assert!(format_i32(0) < format_i32(1));
    assert!(format_i32(1) < format_i32(i32::MAX));

    assert!(format_i32(-1000000) < format_i32(-1));
    assert!(format_i32(-1) < format_i32(0));
    assert!(format_i32(0) < format_i32(1000000));
}

#[test]
fn test_i64_offset_boundary() {
    assert_eq!(format_i64(i64::MIN), "00000000000000000000");
    assert_eq!(format_i64(-1), "09223372036854775807");
    assert_eq!(format_i64(0), "09223372036854775808");
    assert_eq!(format_i64(1), "09223372036854775809");
    assert_eq!(format_i64(i64::MAX), "18446744073709551615");
}

#[test]
fn test_i64_sort_order() {
    assert!(format_i64(i64::MIN) < format_i64(-1));
    assert!(format_i64(-1) < format_i64(0));
    assert!(format_i64(0) < format_i64(1));
    assert!(format_i64(1) < format_i64(i64::MAX));

    assert!(format_i64(-1000000000000) < format_i64(-1));
    assert!(format_i64(-1) < format_i64(0));
    assert!(format_i64(0) < format_i64(1000000000000));
}

#[test]
fn test_i64_max_min_width() {
    let min = format_i64(i64::MIN);
    let max = format_i64(i64::MAX);
    assert_eq!(min.len(), 20);
    assert_eq!(max.len(), 20);
}

#[test]
fn test_i32_consecutive_values() {
    for v in -100..100 {
        assert!(
            format_i32(v) < format_i32(v + 1),
            "format_i32({}) = {} should be < format_i32({}) = {}",
            v,
            format_i32(v),
            v + 1,
            format_i32(v + 1)
        );
    }
}

#[test]
fn test_i8_all_values_sorted() {
    let formatted: Vec<_> = (i8::MIN..=i8::MAX)
        .map(|v| (v, format_i8(v)))
        .collect();

    for window in formatted.windows(2) {
        let (v1, s1) = &window[0];
        let (v2, s2) = &window[1];
        assert!(
            s1 < s2,
            "i8 sort broken: format_i8({}) = {} should be < format_i8({}) = {}",
            v1, s1, v2, s2
        );
    }
}

#[test]
fn test_mixed_unsigned_field_format() {
    let pk = format!("Product/store_id={}&version={:05}", "abc", 42u16);
    assert_eq!(pk, "Product/store_id=abc&version=00042");
}

#[test]
fn test_mixed_signed_field_format() {
    let offset_val = (-5i32 as u32).wrapping_add(2147483648u32);
    let sk = format!("score={:010}", offset_val);
    assert_eq!(sk, "score=2147483643");
}
