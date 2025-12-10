pub mod framework;
pub mod language;
pub mod package_manager;

pub use framework::Framework;
pub use language::Language;
pub use package_manager::PackageManager;

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub language: Language,
    pub package_manager: PackageManager,
    pub framework: Framework,
}
