use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framework {
    Hono,
}

impl Framework {
    pub fn package_name(&self) -> &str {
        match self {
            Framework::Hono => "hono",
        }
    }

    pub fn additional_packages(&self) -> Vec<&str> {
        vec![
            "@bytecodealliance/jco",
            "@bytecodealliance/componentize-js",
            "@bytecodealliance/jco-std",
            "rolldown",
        ]
    }
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Framework::Hono => write!(f, "Hono"),
        }
    }
}
