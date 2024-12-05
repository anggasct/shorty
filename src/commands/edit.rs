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
            let tags_str = if new_tags.is_empty() {
                String::new()
            } else {
                format!(" #tags:{}", new_tags.join(","))
            };

            let note_comment = new_note.as_ref().map(|n| format!(" # {}", n)).unwrap_or_default();
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