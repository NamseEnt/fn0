pub const MAX_PROJECT_NAME_LEN: usize = 100;

pub fn is_valid_project_name(s: &str) -> bool {
    let len = s.chars().count();
    if len == 0 || len > MAX_PROJECT_NAME_LEN {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}
