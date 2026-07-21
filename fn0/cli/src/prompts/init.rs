use crate::config::{Framework, Language, PackageManager};
use color_eyre::{Result, eyre::eyre};
use inquire::Select;

pub struct InitPrompts;

impl InitPrompts {
    pub fn ask_project_name(default: Option<String>) -> Result<String> {
        let name = if let Some(default_name) = default {
            let trimmed = default_name.trim();
            if let Err(e) = fn0_deploy::validate_project_name(trimmed) {
                return Err(eyre!(e));
            }
            trimmed.to_string()
        } else {
            inquire::Text::new("What is your project name?")
                .with_validator(|input: &str| {
                    match fn0_deploy::validate_project_name(input.trim()) {
                        Ok(()) => Ok(inquire::validator::Validation::Valid),
                        Err(e) => Ok(inquire::validator::Validation::Invalid(
                            e.to_string().into(),
                        )),
                    }
                })
                .prompt()?
                .trim()
                .to_string()
        };
        Ok(name)
    }

    pub fn ask_language() -> Result<Language> {
        let languages = vec![Language::TypeScript];

        let selection = Select::new("Which language do you want to use?", languages).prompt()?;

        Ok(selection)
    }

    pub fn ask_package_manager() -> Result<PackageManager> {
        let managers = vec![PackageManager::Bun];

        let selection =
            Select::new("Which package manager do you want to use?", managers).prompt()?;

        Ok(selection)
    }

    pub fn ask_framework() -> Result<Framework> {
        let frameworks = vec![Framework::Hono];

        let selection = Select::new("Which framework do you want to use?", frameworks).prompt()?;

        Ok(selection)
    }
}
