use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use crate::utils::{read_state, update_state};
use super::github::{get_latest_release, compare_versions, current_version, VersionComparison};

pub fn should_check_for_updates(check_interval_hours: i64) -> Result<bool> {
    let state = read_state()?;

    let last_check = match &state.update.last_check {
        Some(timestamp) => timestamp,
        None => return Ok(true),
    };

    let last_check_time = DateTime::parse_from_rfc3339(last_check)?;
    let now = Utc::now();
    let elapsed = now.signed_duration_since(last_check_time);

    Ok(elapsed > Duration::hours(check_interval_hours))
}

pub fn check_for_updates_background(check_interval_hours: i64) -> Result<()> {
    if !should_check_for_updates(check_interval_hours)? {
        return Ok(());
    }

    match get_latest_release(2) {
        Ok(release) => {
            let current = current_version();
            let latest = &release.tag_name;

            update_state(|state| {
                state.update.last_check = Some(Utc::now().to_rfc3339());
            })?;

            match compare_versions(current, latest) {
                VersionComparison::UpdateAvailable => {
                    let state = read_state()?;
                    if state.update.last_notified_version.as_ref() != Some(latest)
                        && !state.update.skipped_versions.contains(latest)
                    {
                        println!("📦 Update available: {} → {}", current, latest);
                        println!("   Run 'shorty update' to install");

                        update_state(|state| {
                            state.update.last_notified_version = Some(latest.clone());
                        })?;
                    }
                }
                VersionComparison::UpToDate => {}
                VersionComparison::Ahead => {}
            }
        }
        Err(_) => {
            update_state(|state| {
                state.update.last_check = Some(Utc::now().to_rfc3339());
            })?;
        }
    }

    Ok(())
}
