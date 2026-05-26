use crate::credentials::Credentials;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const ENV_YAML_FILENAME: &str = "env.yaml";
const DEK_KEY: &str = "__dek";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Plain,
    Secret,
}

pub fn set_plain(project_dir: &Path, key: &str, value: &str) -> Result<()> {
    reject_reserved(key)?;
    let env_path = env_yaml_path(project_dir);
    let mut mapping = load_mapping(&env_path)?;
    mapping.insert(
        serde_yaml::Value::String(key.to_string()),
        serde_yaml::Value::String(value.to_string()),
    );
    save_mapping(&env_path, &mapping)?;
    Ok(())
}

pub async fn set_secret(
    project_dir: &Path,
    key: &str,
    value: &str,
    creds: &Credentials,
) -> Result<()> {
    reject_reserved(key)?;
    let env_path = env_yaml_path(project_dir);
    let mut mapping = load_mapping(&env_path)?;

    let dek_ct = ensure_dek(&mut mapping, creds).await?;
    let ciphertext = call_secrets_encrypt(creds, &dek_ct, value).await?;

    let mut entry = serde_yaml::Mapping::new();
    entry.insert(
        serde_yaml::Value::String("secret".to_string()),
        serde_yaml::Value::String(ciphertext),
    );
    mapping.insert(
        serde_yaml::Value::String(key.to_string()),
        serde_yaml::Value::Mapping(entry),
    );

    save_mapping(&env_path, &mapping)?;
    Ok(())
}

pub fn list_entries(project_dir: &Path) -> Result<Vec<(String, EntryKind)>> {
    let env_path = env_yaml_path(project_dir);
    if !env_path.exists() {
        return Ok(Vec::new());
    }
    let mapping = load_mapping(&env_path)?;
    let mut out = Vec::new();
    for (key_v, value_v) in &mapping {
        let Some(name) = key_v.as_str() else {
            continue;
        };
        if name == DEK_KEY {
            continue;
        }
        let kind = match value_v {
            serde_yaml::Value::Mapping(m) if m.contains_key("secret") => EntryKind::Secret,
            _ => EntryKind::Plain,
        };
        out.push((name.to_string(), kind));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

pub fn unset(project_dir: &Path, key: &str) -> Result<()> {
    reject_reserved(key)?;
    let env_path = env_yaml_path(project_dir);
    if !env_path.exists() {
        return Err(anyhow!("no {} in this project", env_path.display()));
    }
    let mut mapping = load_mapping(&env_path)?;
    if mapping
        .remove(serde_yaml::Value::String(key.to_string()))
        .is_none()
    {
        return Err(anyhow!("{} not present in {}", key, env_path.display()));
    }
    save_mapping(&env_path, &mapping)?;
    Ok(())
}

pub fn env_yaml_path(project_dir: &Path) -> PathBuf {
    project_dir.join(ENV_YAML_FILENAME)
}

fn reject_reserved(key: &str) -> Result<()> {
    if key == DEK_KEY {
        return Err(anyhow!("{} is reserved", DEK_KEY));
    }
    Ok(())
}

async fn ensure_dek(mapping: &mut serde_yaml::Mapping, creds: &Credentials) -> Result<String> {
    if let Some(serde_yaml::Value::Mapping(dek_map)) = mapping.get(DEK_KEY)
        && let Some(serde_yaml::Value::String(s)) = dek_map.get("encrypted")
    {
        return Ok(s.clone());
    }

    let ct = call_secrets_init(creds).await?;
    let mut dek_entry = serde_yaml::Mapping::new();
    dek_entry.insert(
        serde_yaml::Value::String("encrypted".to_string()),
        serde_yaml::Value::String(ct.clone()),
    );
    mapping.insert(
        serde_yaml::Value::String(DEK_KEY.to_string()),
        serde_yaml::Value::Mapping(dek_entry),
    );
    Ok(ct)
}

async fn call_secrets_init(creds: &Credentials) -> Result<String> {
    #[derive(Serialize)]
    struct Empty {}
    #[derive(Deserialize)]
    #[serde(tag = "t", rename_all_fields = "camelCase")]
    enum InitResp {
        Ok { encrypted_dek: String },
        Unauthorized,
        Error { message: String },
    }
    let resp: InitResp = post_action(creds, "secrets_init", &Empty {}).await?;
    match resp {
        InitResp::Ok { encrypted_dek } => Ok(encrypted_dek),
        InitResp::Unauthorized => Err(anyhow!("unauthorized — `fn0 login` again")),
        InitResp::Error { message } => Err(anyhow!("control error: {message}")),
    }
}

async fn call_secrets_encrypt(
    creds: &Credentials,
    encrypted_dek: &str,
    value: &str,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Req<'a> {
        encrypted_dek: &'a str,
        value: &'a str,
    }
    #[derive(Deserialize)]
    #[serde(tag = "t", rename_all_fields = "camelCase")]
    enum EncResp {
        Ok { ciphertext: String },
        Unauthorized,
        Error { message: String },
    }
    let resp: EncResp = post_action(
        creds,
        "secrets_encrypt",
        &Req {
            encrypted_dek,
            value,
        },
    )
    .await?;
    match resp {
        EncResp::Ok { ciphertext } => Ok(ciphertext),
        EncResp::Unauthorized => Err(anyhow!("unauthorized — `fn0 login` again")),
        EncResp::Error { message } => Err(anyhow!("control error: {message}")),
    }
}

async fn post_action<I, O>(creds: &Credentials, name: &str, body: &I) -> Result<O>
where
    I: Serialize,
    O: serde::de::DeserializeOwned,
{
    let url = format!(
        "{}/__forte_action/{}",
        creds.control_url.trim_end_matches('/'),
        name
    );
    let resp = reqwest::Client::new()
        .post(url)
        .bearer_auth(&creds.token)
        .json(body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("control returned {status}: {body}"));
    }
    let parsed = resp.json().await?;
    Ok(parsed)
}

fn load_mapping(p: &Path) -> Result<serde_yaml::Mapping> {
    if !p.exists() {
        return Ok(serde_yaml::Mapping::new());
    }
    let content = std::fs::read_to_string(p)?;
    if content.trim().is_empty() {
        return Ok(serde_yaml::Mapping::new());
    }
    let value: serde_yaml::Value = serde_yaml::from_str(&content)?;
    match value {
        serde_yaml::Value::Mapping(m) => Ok(m),
        _ => Err(anyhow!("{} must contain a mapping", p.display())),
    }
}

fn save_mapping(p: &Path, m: &serde_yaml::Mapping) -> Result<()> {
    let s = serde_yaml::to_string(&serde_yaml::Value::Mapping(m.clone()))?;
    std::fs::write(p, s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn set_plain_writes_scalar() {
        let dir = TempDir::new().unwrap();
        set_plain(dir.path(), "FOO", "bar").unwrap();
        let content = std::fs::read_to_string(env_yaml_path(dir.path())).unwrap();
        assert!(content.contains("FOO: bar"));
    }

    #[test]
    fn set_plain_overwrites_existing() {
        let dir = TempDir::new().unwrap();
        set_plain(dir.path(), "FOO", "first").unwrap();
        set_plain(dir.path(), "FOO", "second").unwrap();
        let content = std::fs::read_to_string(env_yaml_path(dir.path())).unwrap();
        assert!(content.contains("FOO: second"));
        assert!(!content.contains("FOO: first"));
    }

    #[test]
    fn list_entries_classifies_plain_and_secret() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            env_yaml_path(dir.path()),
            "__dek:\n  encrypted: ct\nFOO: plain_value\nBAR:\n  secret: ct\n",
        )
        .unwrap();
        let entries = list_entries(dir.path()).unwrap();
        assert_eq!(
            entries,
            vec![
                ("BAR".to_string(), EntryKind::Secret),
                ("FOO".to_string(), EntryKind::Plain),
            ]
        );
    }

    #[test]
    fn unset_removes_entry() {
        let dir = TempDir::new().unwrap();
        set_plain(dir.path(), "FOO", "bar").unwrap();
        set_plain(dir.path(), "BAZ", "qux").unwrap();
        unset(dir.path(), "FOO").unwrap();
        let entries = list_entries(dir.path()).unwrap();
        assert_eq!(entries, vec![("BAZ".to_string(), EntryKind::Plain)]);
    }

    #[test]
    fn unset_missing_key_errors() {
        let dir = TempDir::new().unwrap();
        set_plain(dir.path(), "FOO", "bar").unwrap();
        let err = unset(dir.path(), "NOPE").unwrap_err();
        assert!(err.to_string().contains("not present"));
    }

    #[test]
    fn reject_reserved_dek_key() {
        let dir = TempDir::new().unwrap();
        let err = set_plain(dir.path(), DEK_KEY, "x").unwrap_err();
        assert!(err.to_string().contains("reserved"));
    }
}
