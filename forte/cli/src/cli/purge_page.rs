use anyhow::Result;
use std::path::PathBuf;

pub async fn run(paths: Vec<String>, project_dir: PathBuf) -> Result<()> {
    let project_id = crate::cli::purge::load_project_id(&project_dir)?;
    let purged = fn0_deploy::static_page_purge(&project_id, &paths).await?;
    for path in &purged {
        println!("{path}");
    }
    println!("queued {} invalidation(s)", purged.len());
    Ok(())
}
