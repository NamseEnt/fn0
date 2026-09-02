use anyhow::{Result, anyhow};
use std::path::Path;
use toml_edit::{DocumentMut, value};

const CONFIG_FILE_NAME: &str = "Forte.toml";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct CloudConfig {
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub zone: Option<String>,
    pub domain: Option<String>,
    pub cloudflare_account_id: Option<String>,
    pub cloudflare_broker_url: Option<String>,
}

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

fn read_optional_string(document: &DocumentMut, key: &str) -> Option<String> {
    document
        .get(key)
        .and_then(|item| item.as_str())
        .map(str::to_string)
}

pub fn read_cloud_config(project_dir: &Path) -> Result<CloudConfig> {
    let document = load(project_dir)?;
    Ok(CloudConfig {
        project_id: read_optional_string(&document, "project_id"),
        project_name: read_optional_string(&document, "project_name"),
        zone: read_optional_string(&document, "zone"),
        domain: read_optional_string(&document, "domain"),
        cloudflare_account_id: read_optional_string(&document, "cloudflare_account_id"),
        cloudflare_broker_url: read_optional_string(&document, "cloudflare_broker_url"),
    })
}

pub fn read_optional_project_id(project_dir: &Path) -> Result<Option<String>> {
    Ok(read_cloud_config(project_dir)?.project_id)
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
    Ok(read_cloud_config(project_dir)?.domain)
}

pub fn write_cloud_config(
    project_dir: &Path,
    project_id: &str,
    project_name: &str,
    zone: &str,
    domain: &str,
    cloudflare_account_id: &str,
    cloudflare_broker_url: &str,
) -> Result<()> {
    let mut document = load(project_dir)?;
    document["project_id"] = value(project_id);
    document["project_name"] = value(project_name);
    document["zone"] = value(zone);
    document["domain"] = value(domain);
    document["cloudflare_account_id"] = value(cloudflare_account_id);
    document["cloudflare_broker_url"] = value(cloudflare_broker_url);
    save(project_dir, &document)
}

pub fn clear_cloud_config(project_dir: &Path) -> Result<()> {
    let mut document = load(project_dir)?;
    document.remove("project_id");
    document.remove("project_name");
    document.remove("zone");
    document.remove("domain");
    document.remove("cloudflare_account_id");
    document.remove("cloudflare_broker_url");
    save(project_dir, &document)
}

pub fn clear_broker_config(project_dir: &Path) -> Result<()> {
    let mut document = load(project_dir)?;
    document.remove("cloudflare_account_id");
    document.remove("cloudflare_broker_url");
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
        write_cloud_config(
            dir.path(),
            "abc123",
            "my-app",
            "example.com",
            "my-app.example.com",
            "0123456789abcdef0123456789abcdef",
            "https://fn0-broker.example.workers.dev",
        )
        .unwrap();
        assert_eq!(
            read_back(&dir),
            "# keep me\nother = \"value\"\nproject_id = \"abc123\"\nproject_name = \"my-app\"\nzone = \"example.com\"\ndomain = \"my-app.example.com\"\ncloudflare_account_id = \"0123456789abcdef0123456789abcdef\"\ncloudflare_broker_url = \"https://fn0-broker.example.workers.dev\"\n\n[table]\nnested = 1\n"
        );
    }

    #[test]
    fn read_optional_domain_reads_what_was_written() {
        let dir = project_with("");
        write_cloud_config(
            dir.path(),
            "abc123",
            "my-app",
            "example.com",
            "my-app.example.com",
            "0123456789abcdef0123456789abcdef",
            "https://fn0-broker.example.workers.dev",
        )
        .unwrap();
        assert_eq!(
            read_optional_domain(dir.path()).unwrap().as_deref(),
            Some("my-app.example.com")
        );
    }

    #[test]
    fn read_cloud_config_reads_all_cloud_fields() {
        let dir = project_with("");
        write_cloud_config(
            dir.path(),
            "abc123",
            "my-app",
            "example.com",
            "my-app.example.com",
            "0123456789abcdef0123456789abcdef",
            "https://fn0-broker.example.workers.dev",
        )
        .unwrap();
        assert_eq!(
            read_cloud_config(dir.path()).unwrap(),
            CloudConfig {
                project_id: Some("abc123".to_string()),
                project_name: Some("my-app".to_string()),
                zone: Some("example.com".to_string()),
                domain: Some("my-app.example.com".to_string()),
                cloudflare_account_id: Some("0123456789abcdef0123456789abcdef".to_string()),
                cloudflare_broker_url: Some("https://fn0-broker.example.workers.dev".to_string()),
            }
        );
    }

    #[test]
    fn clear_cloud_config_keeps_other_keys_and_formatting() {
        let dir = project_with(
            "project_id = \"abc123\"\nproject_name = \"my-app\"\nzone = \"example.com\"\ndomain = \"my-app.example.com\"\ncloudflare_account_id = \"0123456789abcdef0123456789abcdef\"\ncloudflare_broker_url = \"https://fn0-broker.example.workers.dev\"\n# keep me\nother = \"value\"\n\n[table]\nnested = 1\n",
        );
        clear_cloud_config(dir.path()).unwrap();
        assert_eq!(
            read_back(&dir),
            "# keep me\nother = \"value\"\n\n[table]\nnested = 1\n"
        );
    }

    #[test]
    fn clear_cloud_config_drops_the_comment_attached_to_the_id() {
        let dir = project_with(
            "# about the id\nproject_id = \"abc123\"\nproject_name = \"my-app\"\nzone = \"example.com\"\ndomain = \"my-app.example.com\"\ncloudflare_account_id = \"0123456789abcdef0123456789abcdef\"\ncloudflare_broker_url = \"https://fn0-broker.example.workers.dev\"\nother = \"value\"\n",
        );
        clear_cloud_config(dir.path()).unwrap();
        assert_eq!(read_back(&dir), "other = \"value\"\n");
    }
}
