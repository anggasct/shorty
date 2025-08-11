use crate::utils::get_aliases_path;
use chrono::{DateTime, Local, Utc};
use std::fs;
use std::path::PathBuf;

pub fn create_backup(custom_name: Option<&str>) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    if !aliases_path.exists() {
        anyhow::bail!("Aliases file not found. Nothing to backup.");
    }

    let backup_dir = get_backup_dir()?;
    fs::create_dir_all(&backup_dir)?;

    let backup_name = if let Some(name) = custom_name {
        format!("{name}.txt")
    } else {
        let now = Local::now();
        format!("aliases_backup_{}.txt", now.format("%Y-%m-%d_%H-%M-%S"))
    };

    let backup_path = backup_dir.join(&backup_name);
    fs::copy(&aliases_path, &backup_path)?;

    println!("Backup created: {}", backup_path.display());
    println!("Aliases backed up successfully!");

    Ok(())
}

pub fn restore_backup(backup_file: &str) -> anyhow::Result<()> {
    let backup_path = if backup_file.starts_with('/') {
        PathBuf::from(backup_file)
    } else {
        get_backup_dir()?.join(backup_file)
    };

    if !backup_path.exists() {
        anyhow::bail!("Backup file not found: {}", backup_path.display());
    }

    create_backup(Some("pre_restore"))?;

    let aliases_path = get_aliases_path();
    fs::copy(&backup_path, &aliases_path)?;

    println!("Restored from backup: {}", backup_path.display());
    println!("To apply the changes, please restart your terminal!");

    Ok(())
}

pub fn list_backups() -> anyhow::Result<()> {
    let backup_dir = get_backup_dir()?;

    if !backup_dir.exists() {
        println!("No backups found. Backup directory doesn't exist.");
        return Ok(());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(&backup_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "txt") {
            let metadata = entry.metadata()?;
            let modified = metadata.modified()?;
            let datetime: DateTime<Utc> = modified.into();
            let local_time = datetime.with_timezone(&Local);

            backups.push((path, local_time, metadata.len()));
        }
    }

    if backups.is_empty() {
        println!("No backup files found.");
        return Ok(());
    }

    backups.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Available backups:");
    println!("{:<30} {:<20} {:<10}", "Name", "Created", "Size");
    println!("{}", "-".repeat(60));

    for (path, datetime, size) in backups {
        let name = path.file_name().unwrap().to_string_lossy();
        let created = datetime.format("%Y-%m-%d %H:%M:%S");
        let size_str = if size < 1024 {
            format!("{size} B")
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        };

        println!("{name:<30} {created:<20} {size_str:<10}");
    }

    Ok(())
}

pub fn clean_backups(older_than_days: u32) -> anyhow::Result<()> {
    let backup_dir = get_backup_dir()?;

    if !backup_dir.exists() {
        println!("No backup directory found.");
        return Ok(());
    }

    let cutoff = Utc::now() - chrono::Duration::days(older_than_days as i64);
    let mut removed_count = 0;

    for entry in fs::read_dir(&backup_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "txt") {
            let metadata = entry.metadata()?;
            let modified = metadata.modified()?;
            let datetime: DateTime<Utc> = modified.into();

            if datetime < cutoff {
                fs::remove_file(&path)?;
                println!(
                    "Removed old backup: {}",
                    path.file_name().unwrap().to_string_lossy()
                );
                removed_count += 1;
            }
        }
    }

    if removed_count == 0 {
        println!("No old backups found to clean (older than {older_than_days} days).");
    } else {
        println!("Cleaned {removed_count} old backup(s).");
    }

    Ok(())
}

fn get_backup_dir() -> anyhow::Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home_dir.join(".shorty").join("backups"))
}

pub fn auto_backup() -> anyhow::Result<()> {
    let backup_dir = get_backup_dir()?;
    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir)?;
    }

    let aliases_path = get_aliases_path();
    if !aliases_path.exists() {
        return Ok(());
    }

    let auto_backup_path = backup_dir.join("auto_backup.txt");

    let should_backup = if auto_backup_path.exists() {
        let aliases_modified = fs::metadata(&aliases_path)?.modified()?;
        let backup_modified = fs::metadata(&auto_backup_path)?.modified()?;
        aliases_modified > backup_modified
    } else {
        true
    };

    if should_backup {
        fs::copy(&aliases_path, &auto_backup_path)?;
    }

    Ok(())
}
