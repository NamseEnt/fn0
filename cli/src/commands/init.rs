use crate::{
    config::{Language, ProjectConfig},
    generators::typescript::TypeScriptGenerator,
    prompts::init::InitPrompts,
};
use color_eyre::{eyre::eyre, Result};
use std::path::PathBuf;

pub async fn execute(name: Option<String>) -> Result<()> {
    println!("🚀 Initializing new project...\n");

    let project_name = match name {
        Some(n) => n,
        None => InitPrompts::ask_project_name(None)?,
    };

    let project_path = PathBuf::from(&project_name);
    if project_path.exists() {
        return Err(eyre!(
            "Directory '{}' already exists. Please choose a different name.",
            project_name
        ));
    }

    let language = InitPrompts::ask_language()?;

    let config = match language {
        Language::TypeScript => {
            let package_manager = InitPrompts::ask_package_manager()?;
            let framework = InitPrompts::ask_framework()?;

            ProjectConfig {
                name: project_name.clone(),
                language,
                package_manager,
                framework,
            }
        }
    };

    println!("\n📝 Configuration:");
    println!("  Project name: {}", config.name);
    println!("  Language: {}", config.language);
    println!("  Package manager: {}", config.package_manager);
    println!("  Framework: {}", config.framework);
    println!();

    match language {
        Language::TypeScript => {
            TypeScriptGenerator::generate(&project_path, &config).await?;
        }
    }

    println!("✅ Project '{}' created successfully!\n", project_name);
    println!("To get started:");
    println!("  cd {}", project_name);
    println!("  bun run dev");

    Ok(())
}
