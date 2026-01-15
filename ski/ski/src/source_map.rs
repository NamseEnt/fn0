use base64::prelude::*;
use regex::Regex;
use sourcemap::SourceMap;
use std::path::Path;
use std::sync::LazyLock;

const SOURCE_MAP_PREFIX: &[u8] = b"//# sourceMappingURL=data:application/json;charset=utf-8;base64,";

static STACK_LINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^ {4}at (?:(?P<func>\S.*?)\s\()?(?P<file>.+?):(?P<line>\d+)(?::(?P<col>\d+))?\)?$")
        .unwrap()
});

pub fn rewrite_stack_trace(stack: &str, script_url: &str, source_map_json: &[u8]) -> String {
    let Ok(sm) = SourceMap::from_slice(source_map_json) else {
        return stack.to_string();
    };

    let script_dir = Path::new(script_url)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    stack
        .lines()
        .filter_map(|line| rewrite_stack_line(line, script_url, &script_dir, &sm))
        .collect::<Vec<_>>()
        .join("\n")
}

fn rewrite_stack_line(line: &str, script_url: &str, script_dir: &str, sm: &SourceMap) -> Option<String> {
    let Some(caps) = STACK_LINE_REGEX.captures(line) else {
        return Some(line.to_string());
    };

    let file = caps.name("file").map(|m| m.as_str()).unwrap_or("");

    let is_bundle_file = file.contains(script_url) || file.ends_with("server.js");
    if !is_bundle_file {
        return Some(line.to_string());
    }

    let line_num: u32 = caps
        .name("line")
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);
    let col_num: u32 = caps
        .name("col")
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    if line_num == 0 {
        return None;
    }

    let token = sm.lookup_token(line_num - 1, col_num.saturating_sub(1))?;
    let source = token.get_source()?;

    let resolved_source = if source.starts_with("../") || source.starts_with("./") {
        resolve_relative_path(script_dir, source)
    } else {
        source.to_string()
    };

    let orig_line = token.get_src_line() + 1;
    let orig_col = token.get_src_col() + 1;

    let func_name = caps.name("func").map(|m| m.as_str().trim());
    let result = match func_name {
        Some(name) if !name.is_empty() && name != "eval" => {
            format!("    at {} ({}:{}:{})", name, resolved_source, orig_line, orig_col)
        }
        _ => {
            format!("    at {}:{}:{}", resolved_source, orig_line, orig_col)
        }
    };
    Some(result)
}

fn resolve_relative_path(base_dir: &str, relative: &str) -> String {
    let base = Path::new(base_dir);
    let mut result = base.to_path_buf();

    for component in Path::new(relative).components() {
        match component {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::Normal(name) => {
                result.push(name);
            }
            std::path::Component::CurDir => {}
            _ => {}
        }
    }

    result.to_string_lossy().to_string()
}

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
            let is_whitespace = code[i] == b' '
                || code[i] == b'\t'
                || code[i] == b'\r'
                || code[i] == b'\n';
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
