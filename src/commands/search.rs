use crate::utils::get_aliases_path;
use regex::Regex;
use std::fs;

pub fn search_aliases(query: &str, search_in: Option<&str>, use_regex: bool) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;

    let regex = if use_regex {
        Some(Regex::new(query)?)
    } else {
        None
    };

    let results: Vec<&str> = contents
        .lines()
        .filter(|line| {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                return false;
            }

            if let Some(ref regex) = regex {
                regex.is_match(line)
            } else if let Some(field) = search_in {
                search_in_field(line, query, field)
            } else {
                line.contains(query)
            }
        })
        .collect();

    if results.is_empty() {
        let search_desc = match search_in {
            Some(field) => format!(" in field '{field}'"),
            None => String::new(),
        };
        let regex_desc = if use_regex { " (regex)" } else { "" };
        println!("No aliases found matching: '{query}'{search_desc}{regex_desc}");
    } else {
        println!("Found {} matching alias(es):", results.len());
        for alias in results {
            println!("{alias}");
        }
    }

    Ok(())
}

fn search_in_field(line: &str, query: &str, field: &str) -> bool {
    match field.to_lowercase().as_str() {
        "command" => {
            if let Some(eq_pos) = line.find('=') {
                let command_part = &line[eq_pos + 1..];
                let command = extract_command_from_line(command_part);
                command.to_lowercase().contains(&query.to_lowercase())
            } else {
                false
            }
        }
        "note" => {
            if let Some(hash_pos) = line.find('#') {
                let note_part = &line[hash_pos + 1..];
                if let Some(tags_pos) = note_part.find("#tags:") {
                    let note = note_part[..tags_pos].trim();
                    note.to_lowercase().contains(&query.to_lowercase())
                } else {
                    note_part
                        .trim()
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                }
            } else {
                false
            }
        }
        "tag" => {
            if let Some(tags_pos) = line.find("#tags:") {
                let tags_part = &line[tags_pos + 6..];
                tags_part.to_lowercase().contains(&query.to_lowercase())
            } else {
                false
            }
        }
        _ => line.to_lowercase().contains(&query.to_lowercase()),
    }
}

fn extract_command_from_line(command_part: &str) -> String {
    let mut command = command_part.trim();

    if (command.starts_with('\'') && command.ends_with('\''))
        || (command.starts_with('"') && command.ends_with('"'))
    {
        command = &command[1..command.len() - 1];
    }

    if let Some(hash_pos) = command.find('#') {
        command = command[..hash_pos].trim();
    }

    command.to_string()
}
