use chrono::{DateTime, Local};
use std::{collections::HashMap, fs, path::Path};

use crate::utils::get_aliases_path;

#[derive(Debug)]
struct AliasStats {
    total_aliases: usize,
    aliases_with_notes: usize,
    aliases_with_tags: usize,
    unique_tags: usize,
    tag_frequency: HashMap<String, usize>,
    command_types: HashMap<String, usize>,
    avg_command_length: f64,
    longest_command: String,
    shortest_command: String,
    most_common_commands: Vec<(String, usize)>,
}

pub fn show_stats() -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    if !aliases_path.exists() {
        println!("No aliases file found. Create some aliases first!");
        return Ok(());
    }

    let stats = analyze_aliases(&aliases_path)?;
    let file_stats = get_file_stats(&aliases_path)?;

    display_stats(&stats, &file_stats)?;

    Ok(())
}

fn analyze_aliases(aliases_path: &Path) -> anyhow::Result<AliasStats> {
    let content = fs::read_to_string(aliases_path)?;
    let mut stats = AliasStats {
        total_aliases: 0,
        aliases_with_notes: 0,
        aliases_with_tags: 0,
        unique_tags: 0,
        tag_frequency: HashMap::new(),
        command_types: HashMap::new(),
        avg_command_length: 0.0,
        longest_command: String::new(),
        shortest_command: String::new(),
        most_common_commands: Vec::new(),
    };

    let mut command_lengths = Vec::new();
    let mut command_frequency = HashMap::new();
    let mut all_tags = std::collections::HashSet::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("alias ") {
            stats.total_aliases += 1;

            if let Some((_, command, note, tags)) = parse_alias_line(line) {
                command_lengths.push(command.len());

                if stats.longest_command.len() < command.len() {
                    stats.longest_command = command.clone();
                }

                if stats.shortest_command.is_empty() || stats.shortest_command.len() > command.len()
                {
                    stats.shortest_command = command.clone();
                }

                let command_type = classify_command(&command);
                *stats.command_types.entry(command_type).or_insert(0) += 1;

                let first_word = command.split_whitespace().next().unwrap_or(&command);
                *command_frequency.entry(first_word.to_string()).or_insert(0) += 1;

                if note.is_some() {
                    stats.aliases_with_notes += 1;
                }

                if !tags.is_empty() {
                    stats.aliases_with_tags += 1;
                    for tag in &tags {
                        all_tags.insert(tag.clone());
                        *stats.tag_frequency.entry(tag.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    stats.avg_command_length = if command_lengths.is_empty() {
        0.0
    } else {
        command_lengths.iter().sum::<usize>() as f64 / command_lengths.len() as f64
    };

    stats.unique_tags = all_tags.len();

    let mut sorted_commands: Vec<_> = command_frequency.into_iter().collect();
    sorted_commands.sort_by(|a, b| b.1.cmp(&a.1));
    stats.most_common_commands = sorted_commands.into_iter().take(5).collect();

    Ok(stats)
}

fn get_file_stats(aliases_path: &Path) -> anyhow::Result<FileStats> {
    let metadata = fs::metadata(aliases_path)?;
    let modified = metadata.modified()?;
    let datetime: DateTime<Local> = DateTime::from(modified);

    Ok(FileStats {
        file_size: metadata.len(),
        last_modified: datetime,
        line_count: fs::read_to_string(aliases_path)?.lines().count(),
    })
}

fn display_stats(stats: &AliasStats, file_stats: &FileStats) -> anyhow::Result<()> {
    println!("Shorty Statistics Report");
    println!("═══════════════════════════\n");

    println!("Overview:");
    println!("  Total aliases: {}", stats.total_aliases);
    println!(
        "  Aliases with notes: {} ({:.1}%)",
        stats.aliases_with_notes,
        percentage(stats.aliases_with_notes, stats.total_aliases)
    );
    println!(
        "  Aliases with tags: {} ({:.1}%)",
        stats.aliases_with_tags,
        percentage(stats.aliases_with_tags, stats.total_aliases)
    );
    println!("  Unique tags: {}", stats.unique_tags);

    println!("\nCommand Analysis:");
    println!(
        "  Average command length: {:.1} characters",
        stats.avg_command_length
    );
    if !stats.longest_command.is_empty() {
        println!(
            "  Longest command: {} ({} chars)",
            truncate(&stats.longest_command, 50),
            stats.longest_command.len()
        );
    }
    if !stats.shortest_command.is_empty() {
        println!(
            "  Shortest command: {} ({} chars)",
            truncate(&stats.shortest_command, 50),
            stats.shortest_command.len()
        );
    }

    if !stats.command_types.is_empty() {
        println!("\nCommand Types:");
        let mut sorted_types: Vec<_> = stats.command_types.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));

        for (cmd_type, count) in sorted_types.iter().take(5) {
            println!(
                "  {}: {} ({:.1}%)",
                cmd_type,
                count,
                percentage(**count, stats.total_aliases)
            );
        }
    }

    if !stats.most_common_commands.is_empty() {
        println!("\nMost Common Commands:");
        for (i, (command, count)) in stats.most_common_commands.iter().enumerate() {
            println!("  {}. {} ({}x)", i + 1, command, count);
        }
    }

    if !stats.tag_frequency.is_empty() {
        println!("\nPopular Tags:");
        let mut sorted_tags: Vec<_> = stats.tag_frequency.iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(a.1));

        for (tag, count) in sorted_tags.iter().take(5) {
            println!("  #{}: {}x", tag, count);
        }
    }

    println!("\nFile Information:");
    println!("  File size: {}", format_file_size(file_stats.file_size));
    println!("  Total lines: {}", file_stats.line_count);
    println!(
        "  Last modified: {}",
        file_stats.last_modified.format("%Y-%m-%d %H:%M:%S")
    );

    println!("\nRecommendations:");

    if stats.aliases_with_notes < stats.total_aliases / 2 {
        println!("  • Consider adding notes to more aliases for better organization");
    }

    if stats.aliases_with_tags < stats.total_aliases / 3 {
        println!("  • Try using tags to categorize your aliases");
    }

    if stats.avg_command_length > 100.0 {
        println!("  • Some commands are quite long - consider breaking them down");
    }

    if stats.total_aliases > 50 && stats.unique_tags < 5 {
        println!(
            "  • With {} aliases, more tags could help with organization",
            stats.total_aliases
        );
    }

    println!("\nUse 'shorty validate' to check for potential issues");

    Ok(())
}

#[derive(Debug)]
struct FileStats {
    file_size: u64,
    last_modified: DateTime<Local>,
    line_count: usize,
}

fn parse_alias_line(line: &str) -> Option<(String, String, Option<String>, Vec<String>)> {
    if !line.starts_with("alias ") {
        return None;
    }

    let eq_pos = line.find('=')?;
    let name = line[6..eq_pos].trim().to_string();
    let rest = &line[eq_pos + 1..];

    let mut command = String::new();
    let mut remaining = "";

    let rest = rest.trim();
    if rest.starts_with('\'') {
        if let Some(end_quote) = rest[1..].find('\'') {
            command = rest[1..end_quote + 1].to_string();
            remaining = &rest[end_quote + 2..];
        }
    } else if rest.starts_with('"') {
        if let Some(end_quote) = rest[1..].find('"') {
            command = rest[1..end_quote + 1].to_string();
            remaining = &rest[end_quote + 2..];
        }
    } else {
        if let Some(hash_pos) = rest.find('#') {
            command = rest[..hash_pos].trim().to_string();
            remaining = &rest[hash_pos..];
        } else {
            command = rest.to_string();
        }
    }

    let mut note = None;
    let mut tags = Vec::new();

    if let Some(tags_pos) = remaining.find("#tags:") {
        let tags_part = &remaining[tags_pos + 6..];
        tags = tags_part.split(',').map(|s| s.trim().to_string()).collect();

        let note_part = remaining[..tags_pos].trim();
        if note_part.starts_with('#') {
            let note_text = note_part[1..].trim();
            if !note_text.is_empty() {
                note = Some(note_text.to_string());
            }
        }
    } else if remaining.trim().starts_with('#') {
        let note_text = remaining.trim()[1..].trim();
        if !note_text.is_empty() {
            note = Some(note_text.to_string());
        }
    }

    Some((name, command, note, tags))
}

fn classify_command(command: &str) -> String {
    let first_word = command.split_whitespace().next().unwrap_or(command);

    match first_word {
        cmd if cmd.starts_with("git") => "Git".to_string(),
        "ls" | "ll" | "la" | "dir" => "File Listing".to_string(),
        "cd" | "pushd" | "popd" => "Navigation".to_string(),
        "cp" | "mv" | "rm" | "mkdir" | "rmdir" => "File Operations".to_string(),
        "cat" | "less" | "more" | "head" | "tail" => "File Viewing".to_string(),
        "grep" | "find" | "locate" | "which" => "Search".to_string(),
        "npm" | "yarn" | "pnpm" => "Node.js".to_string(),
        "cargo" | "rustc" => "Rust".to_string(),
        "python" | "python3" | "pip" | "pip3" => "Python".to_string(),
        "docker" | "docker-compose" => "Docker".to_string(),
        "kubectl" | "k8s" => "Kubernetes".to_string(),
        "ssh" | "scp" | "rsync" => "Network".to_string(),
        "curl" | "wget" | "http" => "HTTP".to_string(),
        _ if command.contains("sudo") => "System Admin".to_string(),
        _ if command.contains("|") => "Pipeline".to_string(),
        _ if command.contains("&&") || command.contains("||") => "Compound".to_string(),
        _ => "Other".to_string(),
    }
}

fn percentage(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
