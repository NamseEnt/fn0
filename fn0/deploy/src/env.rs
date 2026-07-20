use crate::credentials::Credentials;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const ENV_YAML_FILENAME: &str = "env.yaml";
const ENV_LOCAL_YAML_FILENAME: &str = "env.local.yaml";
const DEK_KEY: &str = "__dek";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Plain,
    Secret,
}

/// Environment for `forte dev`, resolved without network or credentials:
/// plain `env.yaml` entries overlaid with `env.local.yaml`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DevEnv {
    pub vars: Vec<(String, String)>,
    /// `env.yaml` secrets with no `env.local.yaml` override. Their ciphertext
    /// can only be opened by the worker's vault, so dev leaves them unset.
    pub unresolved_secrets: Vec<String>,
}

pub fn set_plain(project_dir: &Path, key: &str, value: &str) -> Result<()> {
    set_plain_at(&env_yaml_path(project_dir), key, value)
}

pub fn set_plain_local(project_dir: &Path, key: &str, value: &str) -> Result<()> {
    set_plain_at(&env_local_yaml_path(project_dir), key, value)
}

fn set_plain_at(env_path: &Path, key: &str, value: &str) -> Result<()> {
    reject_reserved(key)?;
    let mut mapping = load_mapping(env_path)?;
    mapping.insert(
        serde_yaml::Value::String(key.to_string()),
        serde_yaml::Value::String(value.to_string()),
    );
    save_mapping(env_path, &mapping)?;
    Ok(())
}

pub fn load_dev_env(project_dir: &Path) -> Result<DevEnv> {
    let mut dev_env = DevEnv::default();

    let shared_path = env_yaml_path(project_dir);
    for (key, value) in load_mapping(&shared_path)? {
        let key = entry_key(&shared_path, &key)?;
        if key == DEK_KEY {
            continue;
        }
        match value {
            serde_yaml::Value::Mapping(m) if m.contains_key("secret") => {
                dev_env.unresolved_secrets.push(key.to_string());
            }
            value => dev_env
                .vars
                .push((key.to_string(), plain_value(&shared_path, key, &value)?)),
        }
    }

    let local_path = env_local_yaml_path(project_dir);
    for (key, value) in load_mapping(&local_path)? {
        let key = entry_key(&local_path, &key)?;
        reject_reserved(key)?;
        if matches!(&value, serde_yaml::Value::Mapping(m) if m.contains_key("secret")) {
            return Err(anyhow!(
                "{}: {key} uses `secret:`, but {ENV_LOCAL_YAML_FILENAME} is plaintext and gitignored — write the value directly",
                local_path.display()
            ));
        }
        let value = plain_value(&local_path, key, &value)?;
        dev_env.unresolved_secrets.retain(|k| k != key);
        match dev_env.vars.iter_mut().find(|(k, _)| k == key) {
            Some(existing) => existing.1 = value,
            None => dev_env.vars.push((key.to_string(), value)),
        }
    }

    Ok(dev_env)
}

fn entry_key<'a>(path: &Path, key: &'a serde_yaml::Value) -> Result<&'a str> {
    key.as_str()
        .ok_or_else(|| anyhow!("{}: key is not a string", path.display()))
}

// Mirrors the worker's stricter parse (fn0/worker/src/env_yaml.rs): a bare
// `PORT: 8080` is a YAML number and must fail here too, not silently work in
// dev and then break the deploy.
fn plain_value(path: &Path, key: &str, value: &serde_yaml::Value) -> Result<String> {
    match value {
        serde_yaml::Value::String(s) => Ok(s.clone()),
        _ => Err(anyhow!(
            "{}: {key} must be a quoted string or a `secret:` mapping",
            path.display()
        )),
    }
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

pub fn env_local_yaml_path(project_dir: &Path) -> PathBuf {
    project_dir.join(ENV_LOCAL_YAML_FILENAME)
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

    #[test]
    fn dev_env_is_empty_without_files() {
        let dir = TempDir::new().unwrap();
        assert_eq!(load_dev_env(dir.path()).unwrap(), DevEnv::default());
    }

    #[test]
    fn dev_env_takes_plain_shared_entries_and_defers_secrets() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            env_yaml_path(dir.path()),
            "__dek:\n  encrypted: ct\nSHARED: from_shared\nAPI_KEY:\n  secret: ct\n",
        )
        .unwrap();

        let dev_env = load_dev_env(dir.path()).unwrap();
        assert_eq!(
            dev_env.vars,
            vec![("SHARED".to_string(), "from_shared".to_string())]
        );
        assert_eq!(dev_env.unresolved_secrets, vec!["API_KEY".to_string()]);
    }

    #[test]
    fn local_overrides_shared_and_resolves_secret() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            env_yaml_path(dir.path()),
            "SHARED: from_shared\nAPI_KEY:\n  secret: ct\n",
        )
        .unwrap();
        std::fs::write(
            env_local_yaml_path(dir.path()),
            "SHARED: from_local\nAPI_KEY: dev_key\nLOCAL_ONLY: x\n",
        )
        .unwrap();

        let dev_env = load_dev_env(dir.path()).unwrap();
        assert_eq!(
            dev_env.vars,
            vec![
                ("SHARED".to_string(), "from_local".to_string()),
                ("API_KEY".to_string(), "dev_key".to_string()),
                ("LOCAL_ONLY".to_string(), "x".to_string()),
            ]
        );
        assert!(dev_env.unresolved_secrets.is_empty());
    }

    #[test]
    fn local_secret_entry_is_rejected() {
        let dir = TempDir::new().unwrap();
        std::fs::write(env_local_yaml_path(dir.path()), "API_KEY:\n  secret: ct\n").unwrap();
        let err = load_dev_env(dir.path()).unwrap_err();
        assert!(err.to_string().contains("plaintext and gitignored"));
    }

    #[test]
    fn non_string_plain_value_is_rejected() {
        let dir = TempDir::new().unwrap();
        std::fs::write(env_yaml_path(dir.path()), "PORT: 8080\n").unwrap();
        let err = load_dev_env(dir.path()).unwrap_err();
        assert!(err.to_string().contains("must be a quoted string"));
    }
}
