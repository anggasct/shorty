use anyhow::{Context, Result};
use std::io::{self, Write};
use crate::updater::{
    get_latest_release, compare_versions, current_version, find_asset_url,
    VersionComparison, download_binary, get_temp_download_path, backup_current_binary,
    install_binary, verify_binary, cleanup_max_backups,
};
use crate::utils::update_state;

pub fn run_update(check_only: bool, force: bool) -> Result<()> {
    println!("Checking for updates...");

    let release = get_latest_release(30)
        .context("Failed to check for updates. Please check your internet connection.")?;

    let current = current_version();
    let latest = &release.tag_name;

    println!("Current version: v{}", current);
    println!("Latest version:  {}", latest);

    let comparison = compare_versions(current, latest);

    match comparison {
        VersionComparison::UpToDate if !force => {
            println!("✓ You are already running the latest version!");
            return Ok(());
        }
        VersionComparison::Ahead if !force => {
            println!("✓ You are running a development version ahead of the latest release.");
            return Ok(());
        }
        VersionComparison::UpdateAvailable | _ => {
            if !release.body.is_empty() {
                println!("\nChangelog:");
                println!("{}", format_changelog(&release.body));
            }

            if check_only {
                println!("\nRun 'shorty update' to install the new version.");
                return Ok(());
            }

            if !confirm_update()? {
                println!("Update cancelled.");

                update_state(|state| {
                    if !state.update.skipped_versions.contains(latest) {
                        state.update.skipped_versions.push(latest.clone());
                    }
                })?;

                return Ok(());
            }

            perform_update(&release, current)?;
        }
    }

    Ok(())
}

fn confirm_update() -> Result<bool> {
    print!("\nDo you want to download and install this update? [Y/n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let answer = input.trim().to_lowercase();
    Ok(answer.is_empty() || answer == "y" || answer == "yes")
}

fn perform_update(release: &crate::updater::Release, current_version: &str) -> Result<()> {
    println!("\n=== Starting Update Process ===");

    println!("1. Backing up current binary...");
    backup_current_binary(current_version)?;

    println!("2. Finding download URL...");
    let download_url = find_asset_url(release)?;
    println!("   URL: {}", download_url);

    println!("3. Downloading new binary...");
    let temp_path = get_temp_download_path();
    download_binary(&download_url, &temp_path)?;
    println!("   Downloaded to: {:?}", temp_path);

    println!("4. Verifying new binary...");
    verify_binary(&temp_path)?;
    println!("   ✓ Verification passed");

    println!("5. Installing new binary...");
    install_binary(&temp_path)?;

    println!("6. Cleaning up old backups...");
    cleanup_max_backups(3)?;

    println!("7. Updating state...");
    update_state(|state| {
        state.update.last_check = Some(chrono::Utc::now().to_rfc3339());
        state.update.last_notified_version = None;
        state.update.skipped_versions.retain(|v| v != &release.tag_name);
    })?;

    if temp_path.exists() {
        std::fs::remove_file(&temp_path).ok();
    }

    println!("\n✓ Update completed successfully!");
    println!("  {} → {}", current_version, release.tag_name);
    println!("\nPlease restart shorty to use the new version.");

    Ok(())
}

fn format_changelog(body: &str) -> String {
    let lines: Vec<&str> = body.lines().take(10).collect();
    let formatted = lines.join("\n");

    if body.lines().count() > 10 {
        format!("{}\n... (truncated)", formatted)
    } else {
        formatted
    }
}

pub fn run_check_only() -> Result<()> {
    run_update(true, false)
}

pub fn run_force_update() -> Result<()> {
    run_update(false, true)
}
