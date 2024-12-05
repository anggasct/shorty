use std::fs;
use std::io::Write;
use crate::utils::get_aliases_path;

pub fn edit_alias(
    alias: &str,
    new_command: &str,
    new_note: &Option<String>,
    new_tags: &[String],
) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    let contents = fs::read_to_string(&aliases_path)?;
    let mut new_contents: Vec<String> = Vec::new();
    let mut alias_found = false;

    for line in contents.lines() {
        if line.starts_with(&format!("alias {}=", alias)) {
            alias_found = true;

            let existing_note = line.split('#').nth(1).map(|s| s.trim().to_string()).unwrap_or_default();
            let existing_tags = line.split('#').nth(2).map(|s| s.trim().to_string()).unwrap_or_default();

            let note_comment = if let Some(note) = new_note {
                format!(" # {}", note)
            } else {
                if !existing_note.is_empty() {
                    format!(" # {}", existing_note)
                } else {
                    String::new()
                }
            };

            let tags_str = if !new_tags.is_empty() {
                format!(" #tags:{}", new_tags.join(","))
            } else {
                if !existing_tags.is_empty() {
                    format!(" #tags:{}", existing_tags)
                } else {
                    String::new()
                }
            };

            new_contents.push(format!("alias {}='{}'{}{}", alias, new_command, note_comment, tags_str));
        } else {
            new_contents.push(line.to_string());
        }
    }

    if !alias_found {
        println!("Alias '{}' not found.", alias);
        return Ok(());
    }

    let mut file = fs::File::create(&aliases_path)?;
    for line in new_contents {
        writeln!(file, "{}", line)?;
    }

    println!("Edited alias: {} -> {}", alias, new_command);
    println!("To apply the changes, please restart your terminal!");

    Ok(())
}