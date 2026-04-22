use base64::prelude::*;

const SOURCE_MAP_PREFIX: &[u8] =
    b"//# sourceMappingURL=data:application/json;charset=utf-8;base64,";

pub fn extract_source_map(code: &[u8]) -> Option<Vec<u8>> {
    let range = find_source_map_range(code)?;
    let source_map_comment = &code[range];
    let base64_data = source_map_comment.split_at(SOURCE_MAP_PREFIX.len()).1;
    BASE64_STANDARD.decode(base64_data).ok()
}

fn find_source_map_range(code: &[u8]) -> Option<std::ops::Range<usize>> {
    fn last_non_blank_line_range(code: &[u8]) -> Option<std::ops::Range<usize>> {
        let mut hit_non_whitespace = false;
        let mut end = code.len();
        for i in (0..code.len()).rev() {
            let is_whitespace =
                code[i] == b' ' || code[i] == b'\t' || code[i] == b'\r' || code[i] == b'\n';
            if !hit_non_whitespace && is_whitespace {
                end = i;
            }
            hit_non_whitespace = hit_non_whitespace || !is_whitespace;
            if hit_non_whitespace && code[i] == b'\n' {
                return Some(i + 1..end);
            }
        }
        if hit_non_whitespace {
            return Some(0..end);
        }
        None
    }

    let range = last_non_blank_line_range(code)?;
    if code[range.clone()].starts_with(SOURCE_MAP_PREFIX) {
        Some(range)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_source_map() {
        let code = b"console.log('hello');\n//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozfQ==";
        let result = extract_source_map(code);
        assert!(result.is_some());
        let decoded = result.unwrap();
        assert_eq!(decoded, b"{\"version\":3}");
    }

    #[test]
    fn test_no_source_map() {
        let code = b"console.log('hello');";
        let result = extract_source_map(code);
        assert!(result.is_none());
    }

    #[test]
    fn test_with_trailing_newline() {
        let code = b"console.log('hello');\n//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozfQ==\n";
        let result = extract_source_map(code);
        assert!(result.is_some());
    }
}
