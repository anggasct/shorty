use std::fs;
use crate::utils::get_aliases_path;

pub fn remove_alias(alias: &str) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;
    let mut new_contents: String = contents
        .lines()
        .filter(|line| !line.starts_with(&format!("alias {}=", alias)))
        .collect::<Vec<_>>()
        .join("\n");

    if !new_contents.ends_with('\n') {
        new_contents.push('\n');
    }

    fs::write(&aliases_path, new_contents)?;
    println!("Removed alias: {}", alias);

    Ok(())
}

