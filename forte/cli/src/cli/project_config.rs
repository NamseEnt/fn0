use anyhow::{Result, anyhow};
use std::path::Path;
use toml_edit::{DocumentMut, value};

const CONFIG_FILE_NAME: &str = "Forte.toml";

fn load(project_dir: &Path) -> Result<DocumentMut> {
    let config_path = project_dir.join(CONFIG_FILE_NAME);
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    content
        .parse::<DocumentMut>()
        .map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))
}

fn save(project_dir: &Path, document: &DocumentMut) -> Result<()> {
    std::fs::write(project_dir.join(CONFIG_FILE_NAME), document.to_string())
        .map_err(|e| anyhow!("Failed to write Forte.toml: {}", e))
}

pub fn read_optional_project_id(project_dir: &Path) -> Result<Option<String>> {
    Ok(load(project_dir)?
        .get("project_id")
        .and_then(|item| item.as_str())
        .map(str::to_string))
}

pub fn read_project_id(project_dir: &Path) -> Result<String> {
    read_optional_project_id(project_dir)?.ok_or_else(|| {
        anyhow!(
            "'project_id' field missing in Forte.toml. Run `forte cloud init` to register the project."
        )
    })
}

/// The domain the project answers on, as declared by the repository. `forte
/// deploy` reconciles control against this, so moving a project is an edit here
/// rather than a command.
pub fn read_optional_domain(project_dir: &Path) -> Result<Option<String>> {
    Ok(load(project_dir)?
        .get("domain")
        .and_then(|item| item.as_str())
        .map(str::to_string))
}

pub fn write_cloud_config(project_dir: &Path, project_id: &str, domain: &str) -> Result<()> {
    let mut document = load(project_dir)?;
    document["project_id"] = value(project_id);
    document["domain"] = value(domain);
    save(project_dir, &document)
}

pub fn clear_cloud_config(project_dir: &Path) -> Result<()> {
    let mut document = load(project_dir)?;
    document.remove("project_id");
    document.remove("domain");
    save(project_dir, &document)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_with(content: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE_NAME), content).unwrap();
        dir
    }

    fn read_back(dir: &tempfile::TempDir) -> String {
        std::fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap()
    }

    #[test]
    fn write_cloud_config_keeps_other_keys_and_formatting() {
        let dir = project_with("# keep me\nother = \"value\"\n\n[table]\nnested = 1\n");
        write_cloud_config(dir.path(), "abc123", "app.example.com").unwrap();
        assert_eq!(
            read_back(&dir),
            "# keep me\nother = \"value\"\nproject_id = \"abc123\"\ndomain = \"app.example.com\"\n\n[table]\nnested = 1\n"
        );
    }

    #[test]
    fn read_optional_domain_reads_what_was_written() {
        let dir = project_with("");
        write_cloud_config(dir.path(), "abc123", "app.example.com").unwrap();
        assert_eq!(
            read_optional_domain(dir.path()).unwrap().as_deref(),
            Some("app.example.com")
        );
    }

    #[test]
    fn clear_cloud_config_keeps_other_keys_and_formatting() {
        let dir = project_with(
            "project_id = \"abc123\"\ndomain = \"app.example.com\"\n# keep me\nother = \"value\"\n\n[table]\nnested = 1\n",
        );
        clear_cloud_config(dir.path()).unwrap();
        assert_eq!(
            read_back(&dir),
            "# keep me\nother = \"value\"\n\n[table]\nnested = 1\n"
        );
    }

    #[test]
    fn clear_cloud_config_drops_the_comment_attached_to_the_id() {
        let dir = project_with("# about the id\nproject_id = \"abc123\"\nother = \"value\"\n");
        clear_cloud_config(dir.path()).unwrap();
        assert_eq!(read_back(&dir), "other = \"value\"\n");
    }
}
