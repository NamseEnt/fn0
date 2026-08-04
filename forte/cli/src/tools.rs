use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static SCRATCH_PATH_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct ExtractedArchive {
    root: PathBuf,
}

impl Drop for ExtractedArchive {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

async fn download_and_extract(
    dir: &Path,
    name: &str,
    download_url: &str,
) -> Result<ExtractedArchive> {
    let scratch_id = format!(
        "{}-{}",
        std::process::id(),
        SCRATCH_PATH_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let archive_path = dir.join(format!("{name}-temp-{scratch_id}.tar.xz"));

    let response = reqwest::get(download_url).await?.error_for_status()?;
    let bytes = response.bytes().await?;
    std::fs::write(&archive_path, &bytes)?;

    let extract_root = dir.join(format!("{name}-temp-extract-{scratch_id}"));
    std::fs::create_dir_all(&extract_root)?;
    let extracted = ExtractedArchive { root: extract_root };

    let status = Command::new("tar")
        .arg("xf")
        .arg(&archive_path)
        .arg("-C")
        .arg(&extracted.root)
        .status()
        .context("Failed to run tar")?;

    std::fs::remove_file(&archive_path)?;

    if !status.success() {
        anyhow::bail!("Failed to extract {} archive", name);
    }

    Ok(extracted)
}

fn tools_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".forte").join("bin"))
}

fn target_triple() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-gnu"),
        (os, arch) => anyhow::bail!("Unsupported platform: {os}-{arch}"),
    }
}

pub async fn ensure_github_tool(
    name: &str,
    version: &str,
    download_url: &str,
    binary_name: &str,
) -> Result<PathBuf> {
    let dir = tools_dir()?;
    let path = dir.join(format!("{}-{}", name, version));

    if path.exists() {
        return Ok(path);
    }

    std::fs::create_dir_all(&dir)?;

    println!("Downloading {} {}...", name, version);

    let extracted = download_and_extract(&dir, name, download_url).await?;
    let extracted_binary = find_binary(&extracted.root, binary_name)?;
    std::fs::rename(&extracted_binary, &path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("{} {} ready.", name, version);
    Ok(path)
}

pub async fn ensure_github_tool_with_libs(
    name: &str,
    version: &str,
    download_url: &str,
    binary_name: &str,
) -> Result<PathBuf> {
    let dir = tools_dir()?;
    let install_dir = dir.join(format!("{}-{}", name, version));
    let binary_path = install_dir.join(binary_name);

    if binary_path.exists() {
        return Ok(binary_path);
    }

    std::fs::create_dir_all(&dir)?;

    println!("Downloading {} {}...", name, version);

    let extracted = download_and_extract(&dir, name, download_url).await?;
    let extracted_binary = find_binary(&extracted.root, binary_name)?;
    let extracted_dir = extracted_binary.parent().unwrap();

    if let Err(rename_error) = std::fs::rename(extracted_dir, &install_dir) {
        if binary_path.exists() {
            return Ok(binary_path);
        }
        copy_dir_recursive(extracted_dir, &install_dir).with_context(|| {
            format!("Failed to install {name} after rename failed: {rename_error}")
        })?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("{} {} ready.", name, version);
    Ok(binary_path)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn find_binary(dir: &Path, binary_name: &str) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Ok(found) = find_binary(&path, binary_name) {
                return Ok(found);
            }
        } else if path.file_name().is_some_and(|n| n == binary_name) {
            return Ok(path);
        }
    }
    anyhow::bail!("{} binary not found in archive", binary_name)
}

pub fn fn0_release_url(package: &str, version: &str) -> Result<String> {
    let triple = target_triple()?;
    Ok(format!(
        "https://github.com/NamseEnt/fn0/releases/download/{package}-v{version}/{package}-{triple}.tar.xz"
    ))
}
