use crate::utils::get_aliases_path;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Serialize, Deserialize)]
struct SyncConfig {
    remote_url: String,
    branch: String,
    last_sync: String,
    auto_sync: bool,
    sync_interval: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SyncMetadata {
    version: String,
    synced_at: String,
    device_id: String,
    user: String,
    aliases_count: usize,
    checksum: String,
}

pub fn init_sync(remote_url: Option<&str>, branch: Option<&str>) -> anyhow::Result<()> {
    let sync_dir = get_sync_dir()?;

    if sync_dir.exists() && sync_dir.join(".git").exists() {
        anyhow::bail!("Sync already initialized. Use 'shorty sync status' to check or 'shorty sync reset' to reinitialize");
    }

    fs::create_dir_all(&sync_dir)?;

    let output = Command::new("git")
        .args(["init"])
        .current_dir(&sync_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to initialize git repository: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    if let Some(url) = remote_url {
        let output = Command::new("git")
            .args(["remote", "add", "origin", url])
            .current_dir(&sync_dir)
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to add remote: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    let config = SyncConfig {
        remote_url: remote_url.unwrap_or("").to_string(),
        branch: branch.unwrap_or("main").to_string(),
        last_sync: "never".to_string(),
        auto_sync: false,
        sync_interval: 60,
    };

    save_sync_config(&config)?;

    copy_aliases_to_sync_dir(&sync_dir)?;
    create_initial_commit(&sync_dir)?;

    println!("Sync initialized successfully");
    println!("Sync directory: {}", sync_dir.display());

    if let Some(url) = remote_url {
        println!("Remote URL: {}", url);
        println!("Run 'shorty sync push' to upload your aliases");
    } else {
        println!("Add a remote with 'shorty sync remote add <url>' to enable cloud sync");
    }

    Ok(())
}

pub fn push_sync() -> anyhow::Result<()> {
    let sync_dir = get_sync_dir()?;
    let config = load_sync_config()?;

    if config.remote_url.is_empty() {
        anyhow::bail!("No remote configured. Add one with 'shorty sync remote add <url>'");
    }

    copy_aliases_to_sync_dir(&sync_dir)?;

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&sync_dir)
        .output()?;

    if status_output.stdout.is_empty() {
        println!("No changes to sync");
        return Ok(());
    }

    let changes = String::from_utf8_lossy(&status_output.stdout);
    let change_count = changes.lines().count();

    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(&sync_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to stage changes: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let commit_message = format!(
        "Update aliases - {} changes from {}",
        change_count,
        whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string())
    );

    let output = Command::new("git")
        .args(["commit", "-m", &commit_message])
        .current_dir(&sync_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to commit changes: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new("git")
        .args(["push", "origin", &config.branch])
        .current_dir(&sync_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("rejected") {
            println!("Push rejected. There might be remote changes.");
            println!("Run 'shorty sync pull' first to merge remote changes");
            return Ok(());
        }
        anyhow::bail!("Failed to push: {}", stderr);
    }

    let mut new_config = config;
    new_config.last_sync = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    save_sync_config(&new_config)?;

    println!("Successfully pushed {} changes", change_count);
    println!("Synced to: {}", new_config.remote_url);

    Ok(())
}

pub fn pull_sync() -> anyhow::Result<()> {
    let sync_dir = get_sync_dir()?;
    let config = load_sync_config()?;

    if config.remote_url.is_empty() {
        anyhow::bail!("No remote configured. Add one with 'shorty sync remote add <url>'");
    }

    let output = Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&sync_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to fetch from remote: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let local_changes = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&sync_dir)
        .output()?;

    if !local_changes.stdout.is_empty() {
        println!("Local changes detected. Stashing before pull...");

        let output = Command::new("git")
            .args(["stash", "push", "-m", "Auto-stash before sync pull"])
            .current_dir(&sync_dir)
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to stash local changes: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    let output = Command::new("git")
        .args(["pull", "origin", &config.branch])
        .current_dir(&sync_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to pull changes: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    copy_aliases_from_sync_dir(&sync_dir)?;

    let mut new_config = config;
    new_config.last_sync = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    save_sync_config(&new_config)?;

    println!("Successfully pulled remote changes");
    println!("Aliases updated from remote");

    let stash_list = Command::new("git")
        .args(["stash", "list"])
        .current_dir(&sync_dir)
        .output()?;

    if !stash_list.stdout.is_empty() {
        println!("Restoring local changes...");

        let output = Command::new("git")
            .args(["stash", "pop"])
            .current_dir(&sync_dir)
            .output()?;

        if !output.status.success() {
            println!("Conflict detected while restoring local changes");
            println!("Resolve conflicts manually in: {}", sync_dir.display());
        } else {
            println!("Local changes restored successfully");
        }
    }

    Ok(())
}

pub fn sync_status() -> anyhow::Result<()> {
    let sync_dir = get_sync_dir()?;

    if !sync_dir.exists() {
        println!("Sync not initialized");
        println!("Run 'shorty sync init' to get started");
        return Ok(());
    }

    let config = load_sync_config()?;

    println!("Sync Status:\n");

    println!("Sync directory: {}", sync_dir.display());
    println!(
        "Remote URL: {}",
        if config.remote_url.is_empty() {
            "Not configured"
        } else {
            &config.remote_url
        }
    );
    println!("Branch: {}", config.branch);
    println!("Last sync: {}", config.last_sync);
    println!(
        "Auto sync: {}",
        if config.auto_sync {
            "Enabled"
        } else {
            "Disabled"
        }
    );

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&sync_dir)
        .output()?;

    if status_output.stdout.is_empty() {
        println!("Working tree clean - no changes to sync");
    } else {
        let changes = String::from_utf8_lossy(&status_output.stdout);
        let change_count = changes.lines().count();
        println!("{} uncommitted changes", change_count);

        println!("\nChanges:");
        for line in changes.lines().take(10) {
            let status = &line[0..2];
            let file = &line[3..];
            let status_desc = match status.trim() {
                "M" => "Modified",
                "A" => "Added",
                "D" => "Deleted",
                "??" => "Untracked",
                _ => "Changed",
            };
            println!("  {} {}", status_desc, file);
        }

        if change_count > 10 {
            println!("  ... and {} more", change_count - 10);
        }
    }

    if !config.remote_url.is_empty() {
        println!("\nRemote Status:");

        let ahead_behind = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "HEAD...origin/main"])
            .current_dir(&sync_dir)
            .output();

        match ahead_behind {
            Ok(output) if output.status.success() => {
                let counts = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = counts.trim().split('\t').collect();
                if parts.len() == 2 {
                    let ahead = parts[0];
                    let behind = parts[1];
                    println!("  {} commits ahead", ahead);
                    println!("  {} commits behind", behind);

                    if ahead != "0" {
                        println!("Run 'shorty sync push' to upload your changes");
                    }
                    if behind != "0" {
                        println!("Run 'shorty sync pull' to get remote changes");
                    }
                }
            }
            _ => {
                println!("  Unable to check remote status (fetch first)");
            }
        }
    }

    Ok(())
}

pub fn share_alias(alias_name: &str, method: &str) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    if !aliases_path.exists() {
        anyhow::bail!("No aliases file found");
    }

    let content = fs::read_to_string(&aliases_path)?;
    let mut alias_line = None;

    for line in content.lines() {
        if line.contains(&format!("alias {}=", alias_name)) {
            alias_line = Some(line);
            break;
        }
    }

    let alias_line =
        alias_line.ok_or_else(|| anyhow::anyhow!("Alias '{}' not found", alias_name))?;

    match method {
        "clipboard" => {
            let copy_cmd = if cfg!(target_os = "macos") {
                "pbcopy"
            } else if cfg!(target_os = "linux") {
                "xclip -selection clipboard"
            } else {
                anyhow::bail!("Clipboard sharing not supported on this platform");
            };

            let output = if cfg!(target_os = "macos") {
                Command::new("pbcopy").arg(alias_line).output()?
            } else {
                Command::new("sh")
                    .args(["-c", &format!("echo '{}' | {}", alias_line, copy_cmd)])
                    .output()?
            };

            if output.status.success() {
                println!("Alias copied to clipboard:");
                println!("{}", alias_line);
            } else {
                anyhow::bail!("Failed to copy to clipboard");
            }
        }
        "qr" => {
            generate_qr_code(alias_line)?;
        }
        "file" => {
            let share_file = format!("shorty_share_{}.sh", alias_name);
            let content = format!("#!/bin/bash\n# Shared alias from Shorty\n{}\n", alias_line);
            fs::write(&share_file, content)?;

            println!("Alias saved to: {}", share_file);
            println!("Share this file or run it to add the alias");
        }
        _ => {
            anyhow::bail!(
                "Unsupported sharing method: {}. Use: clipboard, qr, file",
                method
            );
        }
    }

    Ok(())
}

pub fn add_remote(url: &str, name: Option<&str>) -> anyhow::Result<()> {
    let sync_dir = get_sync_dir()?;

    if !sync_dir.exists() {
        anyhow::bail!("Sync not initialized. Run 'shorty sync init' first");
    }

    let remote_name = name.unwrap_or("origin");

    let output = Command::new("git")
        .args(["remote", "add", remote_name, url])
        .current_dir(&sync_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already exists") {
            let output = Command::new("git")
                .args(["remote", "set-url", remote_name, url])
                .current_dir(&sync_dir)
                .output()?;

            if !output.status.success() {
                anyhow::bail!(
                    "Failed to update remote: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            println!("Updated remote '{}': {}", remote_name, url);
        } else {
            anyhow::bail!("Failed to add remote: {}", stderr);
        }
    } else {
        println!("Added remote '{}': {}", remote_name, url);
    }

    if remote_name == "origin" {
        let mut config = load_sync_config()?;
        config.remote_url = url.to_string();
        save_sync_config(&config)?;
    }

    Ok(())
}

pub fn reset_sync() -> anyhow::Result<()> {
    let sync_dir = get_sync_dir()?;

    if sync_dir.exists() {
        fs::remove_dir_all(&sync_dir)?;
        println!("Sync reset - removed sync directory");
        println!("Run 'shorty sync init' to reinitialize");
    } else {
        println!("Sync was not initialized");
    }

    Ok(())
}

fn get_sync_dir() -> anyhow::Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    Ok(home_dir.join(".shorty").join("sync"))
}

fn load_sync_config() -> anyhow::Result<SyncConfig> {
    let config_path = get_sync_dir()?.join("sync_config.toml");

    if !config_path.exists() {
        anyhow::bail!("Sync not configured. Run 'shorty sync init' first");
    }

    let content = fs::read_to_string(&config_path)?;
    let config: SyncConfig = toml::from_str(&content)?;

    Ok(config)
}

fn save_sync_config(config: &SyncConfig) -> anyhow::Result<()> {
    let config_path = get_sync_dir()?.join("sync_config.toml");

    let content = toml::to_string_pretty(config)?;
    fs::write(&config_path, content)?;

    Ok(())
}

fn copy_aliases_to_sync_dir(sync_dir: &Path) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let sync_aliases_path = sync_dir.join("aliases");

    if aliases_path.exists() {
        fs::copy(&aliases_path, &sync_aliases_path)?;
    } else {
        fs::write(&sync_aliases_path, "# Shorty aliases\n")?;
    }

    let metadata = SyncMetadata {
        version: "1.0".to_string(),
        synced_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        device_id: whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()),
        user: whoami::fallible::username().unwrap_or_else(|_| "unknown".to_string()),
        aliases_count: count_aliases(&sync_aliases_path)?,
        checksum: calculate_checksum(&sync_aliases_path)?,
    };

    let metadata_path = sync_dir.join("metadata.json");
    let metadata_content = serde_json::to_string_pretty(&metadata)?;
    fs::write(&metadata_path, metadata_content)?;

    Ok(())
}

fn copy_aliases_from_sync_dir(sync_dir: &Path) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let sync_aliases_path = sync_dir.join("aliases");

    if sync_aliases_path.exists() {
        if aliases_path.exists() {
            let backup_path = aliases_path.with_extension("backup");
            fs::copy(&aliases_path, &backup_path)?;
        }

        fs::copy(&sync_aliases_path, &aliases_path)?;
    }

    Ok(())
}

fn create_initial_commit(sync_dir: &Path) -> anyhow::Result<()> {
    let _ = Command::new("git")
        .args(["config", "user.email", "shorty@example.com"])
        .current_dir(sync_dir)
        .output();

    let _ = Command::new("git")
        .args(["config", "user.name", "Shorty Sync"])
        .current_dir(sync_dir)
        .output();

    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(sync_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to stage files: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new("git")
        .args(["commit", "-m", "Initial commit: Shorty aliases sync"])
        .current_dir(sync_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create initial commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

fn count_aliases(path: &Path) -> anyhow::Result<usize> {
    if !path.exists() {
        return Ok(0);
    }

    let content = fs::read_to_string(path)?;
    let count = content
        .lines()
        .filter(|line| line.trim().starts_with("alias "))
        .count();

    Ok(count)
}

fn calculate_checksum(path: &Path) -> anyhow::Result<String> {
    if !path.exists() {
        return Ok("0".to_string());
    }

    let content = fs::read_to_string(path)?;
    let hash = content.len();

    Ok(hash.to_string())
}

fn generate_qr_code(text: &str) -> anyhow::Result<()> {
    println!("QR Code for alias:");
    println!("┌─────────────────────────────────────┐");
    println!("│  QR code would be generated here    │");
    println!("│  Alias: {}                          │", text);
    println!("│                                     │");
    println!("│  Use a proper QR code library for   │");
    println!("│  actual QR code generation          │");
    println!("└─────────────────────────────────────┘");

    let qr_file = "shorty_alias_qr.txt";
    fs::write(qr_file, text)?;
    println!("Alias saved to: {}", qr_file);

    Ok(())
}
