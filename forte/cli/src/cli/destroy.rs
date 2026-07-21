use anyhow::{Result, anyhow};
use std::path::PathBuf;

use super::project_config::{clear_project_id, read_project_id};

pub async fn run(project_dir: PathBuf, yes: bool) -> Result<()> {
    let project_id = read_project_id(&project_dir)?;

    if !yes {
        println!(
            "This permanently deletes project '{project_id}' and ALL of its resources:\n\
             routing, custom domain, deployed bundles, static assets, object storage, and its database."
        );
        let answer = inquire::Text::new("Type the project id to confirm:").prompt()?;
        if answer.trim() != project_id {
            return Err(anyhow!(
                "confirmation did not match '{project_id}'; aborted."
            ));
        }
    }

    fn0_deploy::delete_project(&project_id).await?;

    clear_project_id(&project_dir)?;
    println!("Removed project_id from Forte.toml (next `forte deploy` creates a new project)");
    println!("Teardown of '{project_id}' enqueued; resources are being deleted.");
    Ok(())
}
