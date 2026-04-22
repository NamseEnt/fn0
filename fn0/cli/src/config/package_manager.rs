use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Bun,
}

impl PackageManager {
    pub fn command(&self) -> &str {
        match self {
            PackageManager::Bun => "bun",
        }
    }

    pub fn install_args(&self) -> Vec<&str> {
        match self {
            PackageManager::Bun => vec!["add"],
        }
    }

    pub fn install_dev_args(&self) -> Vec<&str> {
        match self {
            PackageManager::Bun => vec!["add", "-D"],
        }
    }
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManager::Bun => write!(f, "bun"),
        }
    }
}
