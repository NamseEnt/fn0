use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Framework {
    Hono,
    Astro,
}

impl Framework {
    pub fn package_name(&self) -> &str {
        match self {
            Framework::Hono => "hono",
            Framework::Astro => "astro",
        }
    }

    pub fn additional_packages(&self) -> Vec<&str> {
        match self {
            Framework::Hono => vec![
                "@bytecodealliance/jco",
                "@bytecodealliance/componentize-js",
                "@bytecodealliance/jco-std",
                "rolldown",
            ],
            Framework::Astro => vec![
                "@bytecodealliance/jco",
                "@bytecodealliance/componentize-js",
                "@bytecodealliance/jco-std",
                "@astrojs/node",
                "rolldown",
            ],
        }
    }

    pub fn static_assets_dir(&self) -> Option<&'static str> {
        match self {
            Framework::Hono => None,
            Framework::Astro => Some("dist/client"),
        }
    }
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Framework::Hono => write!(f, "Hono"),
            Framework::Astro => write!(f, "Astro"),
        }
    }
}
