use std::fs;
use std::path::PathBuf;

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
