use color_eyre::{Result, eyre::eyre};

pub async fn execute() -> Result<()> {
    Err(eyre!(
        "destroy is not yet migrated to control. See GitHub issue #4."
    ))
}
