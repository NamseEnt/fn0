pub const MAX_PROJECT_NAME_LEN: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong { len: usize },
    InvalidChar { ch: char },
}

impl std::fmt::Display for NameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameError::Empty => write!(f, "name cannot be empty"),
            NameError::TooLong { len } => {
                write!(f, "name too long: {len} chars (max {MAX_PROJECT_NAME_LEN})")
            }
            NameError::InvalidChar { ch } => write!(
                f,
                "name contains invalid character {ch:?}; allowed: letters, digits, '.', '_', '-'"
            ),
        }
    }
}

impl std::error::Error for NameError {}

pub fn validate_project_name(s: &str) -> std::result::Result<(), NameError> {
    let len = s.chars().count();
    if len == 0 {
        return Err(NameError::Empty);
    }
    if len > MAX_PROJECT_NAME_LEN {
        return Err(NameError::TooLong { len });
    }
    if let Some(ch) = s
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '.' && *c != '_' && *c != '-')
    {
        return Err(NameError::InvalidChar { ch });
    }
    Ok(())
}

pub fn is_valid_project_name(s: &str) -> bool {
    validate_project_name(s).is_ok()
}
