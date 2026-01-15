use base64::prelude::*;

const SOURCE_MAP_PREFIX: &str = "//# sourceMappingURL=data:application/json;";

pub fn extract_inline_source_map(code: &str) -> Option<Vec<u8>> {
    for line in code.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with(SOURCE_MAP_PREFIX) {
            let data_part = trimmed.strip_prefix(SOURCE_MAP_PREFIX)?;
            if let Some(base64_start) = data_part.find("base64,") {
                let base64_data = &data_part[base64_start + 7..];
                return BASE64_STANDARD.decode(base64_data).ok();
            }
        }
    }
    None
}
