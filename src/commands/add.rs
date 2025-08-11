use crate::commands::remove::remove_alias;
use crate::utils::get_aliases_path;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

pub fn add_alias(
    alias: &str,
    command: &str,
    note: &Option<String>,
    tags: &[String],
) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    if alias_exists(&aliases_path, alias)? {
        print!("Warning: Alias '{alias}' already exists. Do you want to overwrite it? (y/n): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("y") {
            remove_alias(alias)?;
        } else {
            println!("Operation aborted.");
            return Ok(());
        }
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&aliases_path)?;

    let tags_str = if tags.is_empty() {
        String::new()
    } else {
        format!(" #tags:{}", tags.join(","))
    };

    let note_comment = note.as_ref().map(|n| format!(" # {n}")).unwrap_or_default();

    writeln!(file, "alias {alias}='{command}'{note_comment}{tags_str}")?;

    println!("Added alias: {alias} -> {command}");
    println!("To apply the changes, please restart your terminal!");

    Ok(())
}

fn alias_exists(aliases_path: &PathBuf, alias: &str) -> io::Result<bool> {
    if let Ok(file) = fs::File::open(aliases_path) {
        for line in io::BufReader::new(file).lines().map_while(Result::ok) {
            if line.starts_with(&format!("alias {alias}=")) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
