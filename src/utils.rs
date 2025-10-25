use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

pub fn get_aliases_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let shorty_dir = home_dir.join(".shorty");
    let new_path = shorty_dir.join("aliases");
    let old_path = home_dir.join(".shorty_aliases");

    if let Err(e) = fs::create_dir_all(&shorty_dir) {
        eprintln!("Warning: Could not create .shorty directory: {e}");
    }
    if old_path.exists() && !new_path.exists() {
        if let Err(e) = fs::copy(&old_path, &new_path) {
            eprintln!("Warning: Could not migrate aliases file: {e}");
            return old_path;
        }
        let backup_path = home_dir.join(".shorty_aliases.backup");
        if let Err(e) = fs::rename(&old_path, &backup_path) {
            eprintln!("Warning: Could not backup old aliases file: {e}");
        } else {
            println!("Migrated aliases to ~/.shorty/aliases");
            println!("Backup created at ~/.shorty_aliases.backup");
        }
    }

    new_path
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ShortyState {
    #[serde(default)]
    pub update: UpdateState,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct UpdateState {
    pub last_check: Option<String>,
    pub last_notified_version: Option<String>,
    pub skipped_versions: Vec<String>,
}

pub fn get_state_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let shorty_dir = home_dir.join(".shorty");

    if let Err(e) = fs::create_dir_all(&shorty_dir) {
        eprintln!("Warning: Could not create .shorty directory: {e}");
    }

    shorty_dir.join("shorty.json")
}

pub fn read_state() -> Result<ShortyState> {
    let state_path = get_state_path();

    if !state_path.exists() {
        return Ok(ShortyState::default());
    }

    let content = fs::read_to_string(&state_path)
        .with_context(|| format!("Failed to read state from {:?}", state_path))?;

    let state: ShortyState = serde_json::from_str(&content)
        .with_context(|| "Failed to parse state JSON")?;

    Ok(state)
}

pub fn write_state(state: &ShortyState) -> Result<()> {
    let state_path = get_state_path();

    let content = serde_json::to_string_pretty(state)
        .with_context(|| "Failed to serialize state")?;

    fs::write(&state_path, content)
        .with_context(|| format!("Failed to write state to {:?}", state_path))?;

    Ok(())
}

pub fn update_state<F>(updater: F) -> Result<()>
where
    F: FnOnce(&mut ShortyState),
{
    let mut state = read_state()?;
    updater(&mut state);
    write_state(&state)?;
    Ok(())
}
