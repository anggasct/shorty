use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn download_binary(url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("Failed to create HTTP client")?;

    let mut response = client
        .get(url)
        .send()
        .context("Failed to download binary")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let mut file = fs::File::create(dest)
        .with_context(|| format!("Failed to create file: {:?}", dest))?;

    response
        .copy_to(&mut file)
        .context("Failed to write downloaded binary")?;

    Ok(())
}

pub fn get_current_binary_path() -> Result<PathBuf> {
    std::env::current_exe()
        .context("Failed to get current executable path")
}

pub fn backup_current_binary(version: &str) -> Result<PathBuf> {
    let current_path = get_current_binary_path()?;
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let backup_dir = home_dir.join(".shorty").join("backups");

    fs::create_dir_all(&backup_dir)
        .with_context(|| format!("Failed to create backup directory: {:?}", backup_dir))?;

    let binary_name = get_platform_binary_name();
    let backup_filename = format!("shorty-v{}-{}", version, binary_name);
    let backup_path = backup_dir.join(backup_filename);

    fs::copy(&current_path, &backup_path)
        .with_context(|| format!("Failed to backup binary to {:?}", backup_path))?;

    println!("Backup created at: {:?}", backup_path);
    Ok(backup_path)
}

pub fn install_binary(temp_path: &Path) -> Result<()> {
    let current_path = get_current_binary_path()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(temp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(temp_path, perms)?;
    }

    #[cfg(target_os = "windows")]
    {
        let old_path = current_path.with_extension("old");
        if old_path.exists() {
            fs::remove_file(&old_path).ok();
        }
        fs::rename(&current_path, &old_path)
            .context("Failed to rename current binary")?;

        if let Err(e) = fs::copy(temp_path, &current_path) {
            fs::rename(&old_path, &current_path).ok();
            return Err(e).context("Failed to install new binary");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        fs::rename(temp_path, &current_path)
            .context("Failed to replace binary")?;
    }

    println!("Binary updated successfully!");
    Ok(())
}

pub fn verify_binary(path: &Path) -> Result<()> {
    let output = std::process::Command::new(path)
        .arg("--version")
        .output()
        .context("Failed to verify new binary")?;

    if !output.status.success() {
        return Err(anyhow!("New binary failed verification test"));
    }

    Ok(())
}

pub fn get_temp_download_path() -> PathBuf {
    let temp_dir = std::env::temp_dir();

    #[cfg(target_os = "windows")]
    return temp_dir.join("shorty-update.exe");

    #[cfg(not(target_os = "windows"))]
    return temp_dir.join("shorty-update");
}

fn get_platform_binary_name() -> &'static str {
    #[cfg(target_os = "linux")]
    return "linux";

    #[cfg(target_os = "macos")]
    return "macos";

    #[cfg(target_os = "windows")]
    return "windows";

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    compile_error!("Unsupported platform");
}

pub fn cleanup_max_backups(max_backups: usize) -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let backup_dir = home_dir.join(".shorty").join("backups");

    if !backup_dir.exists() {
        return Ok(());
    }

    let mut backups: Vec<_> = fs::read_dir(&backup_dir)
        .context("Failed to read backup directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name()
                .to_string_lossy()
                .starts_with("shorty-v")
        })
        .collect();

    if backups.len() <= max_backups {
        return Ok(());
    }

    backups.sort_by_key(|entry| {
        entry.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    let to_remove = backups.len() - max_backups;
    for entry in backups.iter().take(to_remove) {
        fs::remove_file(entry.path()).ok();
    }

    Ok(())
}
