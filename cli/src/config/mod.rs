pub mod framework;
pub mod language;
pub mod package_manager;

pub use framework::Framework;
pub use language::Language;
pub use package_manager::PackageManager;

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub language: Language,
    pub package_manager: PackageManager,
    pub framework: Framework,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub language_env: LanguageEnvironment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageEnvironment {
    TypescriptBunHono,
    TypescriptBunAstro,
}

impl From<&ProjectConfig> for LanguageEnvironment {
    fn from(config: &ProjectConfig) -> Self {
        match (config.language, config.package_manager, config.framework) {
            (Language::TypeScript, PackageManager::Bun, Framework::Hono) => {
                LanguageEnvironment::TypescriptBunHono
            }
            (Language::TypeScript, PackageManager::Bun, Framework::Astro) => {
                LanguageEnvironment::TypescriptBunAstro
            }
        }
    }
}

impl Config {
    pub fn from_project_config(project_config: &ProjectConfig) -> Self {
        Self {
            language_env: LanguageEnvironment::from(project_config),
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
