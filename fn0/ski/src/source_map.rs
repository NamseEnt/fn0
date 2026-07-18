use base64::prelude::*;

const SOURCE_MAP_MARKER: &[u8] = b"//# sourceMappingURL=";

pub fn extract_source_map(code: &[u8]) -> Option<Vec<u8>> {
    let range = last_non_blank_line_range(code)?;
    let base64_data = base64_payload(&code[range])?;
    BASE64_STANDARD.decode(base64_data).ok()
}

fn base64_payload(source_map_comment: &[u8]) -> Option<&[u8]> {
    let url = source_map_comment.strip_prefix(SOURCE_MAP_MARKER)?;
    let rest = strip_prefix_ignore_ascii_case(url, b"data:application/json")?;
    let rest = strip_prefix_ignore_ascii_case(rest, b";charset=utf-8").unwrap_or(rest);
    strip_prefix_ignore_ascii_case(rest, b";base64,")
}

fn strip_prefix_ignore_ascii_case<'a>(data: &'a [u8], prefix: &[u8]) -> Option<&'a [u8]> {
    let (head, tail) = data.split_at_checked(prefix.len())?;
    head.eq_ignore_ascii_case(prefix).then_some(tail)
}

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
    fn test_with_charset() {
        let code = b"console.log('hello');\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozfQ==";
        let result = extract_source_map(code);
        assert_eq!(result.unwrap(), b"{\"version\":3}");
    }

    #[test]
    fn test_media_type_case_insensitive() {
        let code = b"console.log('hello');\n//# sourceMappingURL=data:Application/JSON;Charset=UTF-8;Base64,eyJ2ZXJzaW9uIjozfQ==";
        let result = extract_source_map(code);
        assert_eq!(result.unwrap(), b"{\"version\":3}");
    }

    #[test]
    fn test_no_source_map() {
        let code = b"console.log('hello');";
        let result = extract_source_map(code);
        assert!(result.is_none());
    }

    #[test]
    fn test_other_media_type() {
        let code =
            b"console.log('hello');\n//# sourceMappingURL=data:text/plain;base64,eyJ2ZXJzaW9uIjozfQ==";
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
