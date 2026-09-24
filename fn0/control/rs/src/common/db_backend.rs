#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DbBackend {
    Turso,
    Dodb,
}

pub fn configured() -> Result<DbBackend, String> {
    match std::env::var("FN0_DB_BACKEND") {
        Ok(value) => from_value(&value),
        Err(std::env::VarError::NotPresent) => Ok(DbBackend::Turso),
        Err(std::env::VarError::NotUnicode(_)) => {
            Err("FN0_DB_BACKEND is not valid Unicode".to_string())
        }
    }
}

fn from_value(value: &str) -> Result<DbBackend, String> {
    match value {
        "turso" => Ok(DbBackend::Turso),
        "dodb" => Ok(DbBackend::Dodb),
        _ => Err(format!("unknown FN0_DB_BACKEND value {value:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{DbBackend, from_value};

    #[test]
    fn supports_turso_and_dodb_and_rejects_unknown_modes() {
        assert_eq!(from_value("turso"), Ok(DbBackend::Turso));
        assert_eq!(from_value("dodb"), Ok(DbBackend::Dodb));
        assert!(from_value("").is_err());
        assert!(from_value("unknown").is_err());
    }
}
