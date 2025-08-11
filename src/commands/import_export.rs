use crate::utils::get_aliases_path;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
struct AliasData {
    name: String,
    command: String,
    note: Option<String>,
    tags: Vec<String>,
    created_at: Option<String>,
    shell_source: Option<String>,
}

#[derive(Debug)]
pub enum ExportFormat {
    Json,
    Csv,
    Bash,
}

#[derive(Debug)]
pub enum ImportSource {
    File(PathBuf),
    Bash,
    Zsh,
    Fish,
}

impl std::str::FromStr for ExportFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "bash" => Ok(ExportFormat::Bash),
            _ => anyhow::bail!("Unsupported format: {}. Supported: json, csv, bash", s),
        }
    }
}

impl std::str::FromStr for ImportSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(ImportSource::Bash),
            "zsh" => Ok(ImportSource::Zsh),
            "fish" => Ok(ImportSource::Fish),
            path => Ok(ImportSource::File(PathBuf::from(path))),
        }
    }
}

pub fn export_aliases(format: ExportFormat, output_path: Option<&str>) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    if !aliases_path.exists() {
        println!("No aliases file found at {}", aliases_path.display());
        return Ok(());
    }

    let aliases = parse_aliases_file(&aliases_path)?;

    if aliases.is_empty() {
        println!("No aliases found to export");
        return Ok(());
    }

    let content = match format {
        ExportFormat::Json => export_to_json(&aliases)?,
        ExportFormat::Csv => export_to_csv(&aliases)?,
        ExportFormat::Bash => export_to_bash(&aliases)?,
    };

    let output_file = match output_path {
        Some(path) => PathBuf::from(path),
        None => {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let extension = match format {
                ExportFormat::Json => "json",
                ExportFormat::Csv => "csv",
                ExportFormat::Bash => "sh",
            };
            PathBuf::from(format!("shorty_export_{timestamp}. {extension}"))
        }
    };

    fs::write(&output_file, content)?;

    println!(
        "Exported {} aliases to {}",
        aliases.len(),
        output_file.display()
    );
    println!("Export summary:");
    println!("   • Total aliases: {}", aliases.len());
    println!(
        "   • With notes: {}",
        aliases.iter().filter(|a| a.note.is_some()).count()
    );
    println!(
        "   • With tags: {}",
        aliases.iter().filter(|a| !a.tags.is_empty()).count()
    );

    Ok(())
}

pub fn import_aliases(
    source: ImportSource,
    format: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let aliases = match source {
        ImportSource::File(path) => {
            println!("Importing from file: {}", path.display());
            import_from_file(&path, format)?
        }
        ImportSource::Bash => {
            println!("Importing from Bash configuration...");
            import_from_bash()?
        }
        ImportSource::Zsh => {
            println!("Importing from Zsh configuration...");
            import_from_zsh()?
        }
        ImportSource::Fish => {
            println!("Importing from Fish configuration...");
            import_from_fish()?
        }
    };

    if aliases.is_empty() {
        println!("No aliases found to import");
        return Ok(());
    }

    println!(
        "Found {aliases_len} aliases to import",
        aliases_len = aliases.len()
    );

    if dry_run {
        println!("\nDRY RUN - Preview of aliases to import:");
        for alias in &aliases {
            println!(
                "  • {} → {}",
                alias.name,
                if alias.command.len() > 50 {
                    format!("{}...", &alias.command[..47])
                } else {
                    alias.command.clone()
                }
            );
        }
        println!("\nRun without --dry-run to actually import these aliases");
        return Ok(());
    }

    let existing_aliases = parse_aliases_file(&get_aliases_path()).unwrap_or_default();
    let existing_names: std::collections::HashSet<_> =
        existing_aliases.iter().map(|a| &a.name).collect();

    let conflicts: Vec<_> = aliases
        .iter()
        .filter(|a| existing_names.contains(&a.name))
        .collect();

    if !conflicts.is_empty() {
        println!(
            "Found {conflicts_len} conflicting aliases:",
            conflicts_len = conflicts.len()
        );
        for alias in &conflicts {
            println!("  • {}", alias.name);
        }

        println!("\nHow do you want to handle conflicts?");
        println!("  1. Skip conflicting aliases (safe)");
        println!("  2. Overwrite existing aliases");
        println!("  3. Rename with suffix (e.g., alias_imported)");

        println!(
            "Skipping {} conflicting aliases for safety",
            conflicts.len()
        );
    }

    let safe_aliases: Vec<_> = aliases
        .into_iter()
        .filter(|a| !existing_names.contains(&a.name))
        .collect();

    if safe_aliases.is_empty() {
        println!("All aliases would conflict with existing ones. Import cancelled for safety.");
        return Ok(());
    }

    append_aliases_to_file(&safe_aliases)?;

    println!("Successfully imported {} aliases", safe_aliases.len());
    println!("Aliases added to: {}", get_aliases_path().display());

    Ok(())
}

fn parse_aliases_file(path: &Path) -> anyhow::Result<Vec<AliasData>> {
    let content = fs::read_to_string(path)?;
    let mut aliases = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(alias) = parse_alias_line(line) {
            aliases.push(alias);
        }
    }

    Ok(aliases)
}

fn parse_alias_line(line: &str) -> Option<AliasData> {
    if !line.starts_with("alias ") {
        return None;
    }

    let eq_pos = line.find('=')?;
    let name = line[6..eq_pos].trim().to_string();
    let rest = &line[eq_pos + 1..];

    let mut command = String::new();
    let mut remaining = "";

    let rest = rest.trim();
    if let Some(stripped) = rest.strip_prefix('\'') {
        if let Some(end_quote) = stripped.find('\'') {
            command = stripped[..end_quote].to_string();
            remaining = &rest[end_quote + 2..];
        }
    } else if let Some(stripped) = rest.strip_prefix('"') {
        if let Some(end_quote) = stripped.find('"') {
            command = stripped[..end_quote].to_string();
            remaining = &rest[end_quote + 2..];
        }
    } else if let Some(hash_pos) = rest.find('#') {
        command = rest[..hash_pos].trim().to_string();
        remaining = &rest[hash_pos..];
    } else {
        command = rest.to_string();
    }

    let mut note = None;
    let mut tags = Vec::new();

    if let Some(tags_pos) = remaining.find("#tags:") {
        let tags_part = &remaining[tags_pos + 6..];
        tags = tags_part.split(',').map(|s| s.trim().to_string()).collect();

        let note_part = remaining[..tags_pos].trim();
        if let Some(stripped) = note_part.strip_prefix('#') {
            let note_text = stripped.trim();
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

    Some(AliasData {
        name,
        command,
        note,
        tags,
        created_at: Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        shell_source: None,
    })
}

fn export_to_json(aliases: &[AliasData]) -> anyhow::Result<String> {
    let mut export_data = HashMap::new();
    export_data.insert("version", "1.0");
    let timestamp = Local::now().to_rfc3339();
    export_data.insert("exported_at", &timestamp);
    export_data.insert("tool", "shorty");

    let json_aliases = serde_json::to_value(aliases)?;

    let mut full_export = serde_json::Map::new();
    full_export.insert("metadata".to_string(), serde_json::to_value(export_data)?);
    full_export.insert("aliases".to_string(), json_aliases);

    Ok(serde_json::to_string_pretty(&full_export)?)
}

fn export_to_csv(aliases: &[AliasData]) -> anyhow::Result<String> {
    let mut csv = String::new();
    csv.push_str("name,command,note,tags,created_at,shell_source\n");

    for alias in aliases {
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            alias.name.replace('"', "\"\""),
            alias.command.replace('"', "\"\""),
            alias
                .note
                .as_ref()
                .unwrap_or(&"".to_string())
                .replace('"', "\"\""),
            alias.tags.join(";").replace('"', "\"\""),
            alias.created_at.as_ref().unwrap_or(&"".to_string()),
            alias.shell_source.as_ref().unwrap_or(&"".to_string())
        ));
    }

    Ok(csv)
}

fn export_to_bash(aliases: &[AliasData]) -> anyhow::Result<String> {
    let mut bash = String::new();
    bash.push_str("#!/bin/bash\n");
    bash.push_str("# Exported by Shorty alias manager\n");
    bash.push_str(&format!(
        "# Generated on: {}\n\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    for alias in aliases {
        let mut comment_parts = Vec::new();

        if let Some(note) = &alias.note {
            comment_parts.push(note.clone());
        }

        if !alias.tags.is_empty() {
            comment_parts.push(format!("tags:{}", alias.tags.join(",")));
        }

        if !comment_parts.is_empty() {
            bash.push_str(&format!("# {}\n", comment_parts.join(" | ")));
        }

        bash.push_str(&format!("alias {}='{}'\n", alias.name, alias.command));
        bash.push('\n');
    }

    Ok(bash)
}

fn import_from_file(path: &Path, format: Option<&str>) -> anyhow::Result<Vec<AliasData>> {
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    let content = fs::read_to_string(path)?;

    match format {
        Some("json") => import_from_json(&content),
        Some("csv") => import_from_csv(&content),
        Some("bash") | Some("sh") => import_from_bash_file(&content),
        None => {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "json" => import_from_json(&content),
                    "csv" => import_from_csv(&content),
                    "sh" | "bash" => import_from_bash_file(&content),
                    _ => import_from_bash_file(&content),
                }
            } else {
                import_from_bash_file(&content)
            }
        }
        Some(fmt) => anyhow::bail!("Unsupported format: {}", fmt),
    }
}

fn import_from_json(content: &str) -> anyhow::Result<Vec<AliasData>> {
    let data: serde_json::Value = serde_json::from_str(content)?;

    let aliases_value = if data.is_array() {
        &data
    } else if let Some(aliases) = data.get("aliases") {
        aliases
    } else {
        anyhow::bail!(
            "JSON format not recognized. Expected array of aliases or object with 'aliases' field"
        );
    };

    let aliases: Vec<AliasData> = serde_json::from_value(aliases_value.clone())?;
    Ok(aliases)
}

fn import_from_csv(content: &str) -> anyhow::Result<Vec<AliasData>> {
    let mut aliases = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Ok(aliases);
    }

    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        let fields: Vec<&str> = line
            .split(',')
            .map(|s| s.trim_matches('"').trim())
            .collect();

        if fields.len() >= 2 {
            let name = fields[0].to_string();
            let command = fields[1].to_string();
            let note = if fields.len() > 2 && !fields[2].is_empty() {
                Some(fields[2].to_string())
            } else {
                None
            };
            let tags = if fields.len() > 3 && !fields[3].is_empty() {
                fields[3].split(';').map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            };

            aliases.push(AliasData {
                name,
                command,
                note,
                tags,
                created_at: Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                shell_source: Some("csv".to_string()),
            });
        }
    }

    Ok(aliases)
}

fn import_from_bash_file(content: &str) -> anyhow::Result<Vec<AliasData>> {
    let mut aliases = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(alias) = parse_alias_line(line) {
            let mut alias = alias;
            alias.shell_source = Some("bash".to_string());
            aliases.push(alias);
        }
    }

    Ok(aliases)
}

fn import_from_bash() -> anyhow::Result<Vec<AliasData>> {
    let mut aliases = Vec::new();
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    let bash_files = vec![
        home_dir.join(".bashrc"),
        home_dir.join(".bash_aliases"),
        home_dir.join(".bash_profile"),
    ];

    for file_path in bash_files {
        if file_path.exists() {
            println!("Scanning {}", file_path.display());
            match extract_aliases_from_shell_file(&file_path) {
                Ok(mut file_aliases) => {
                    let count = file_aliases.len();
                    for alias in &mut file_aliases {
                        alias.shell_source = Some("bash".to_string());
                    }
                    aliases.extend(file_aliases);
                    println!("  Found {count} aliases");
                }
                Err(e) => {
                    println!("  Error reading file: {e}");
                }
            }
        }
    }

    Ok(aliases)
}

fn import_from_zsh() -> anyhow::Result<Vec<AliasData>> {
    let mut aliases = Vec::new();
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    let zsh_files = vec![home_dir.join(".zshrc"), home_dir.join(".zsh_aliases")];

    for file_path in zsh_files {
        if file_path.exists() {
            println!("Scanning {}", file_path.display());
            match extract_aliases_from_shell_file(&file_path) {
                Ok(mut file_aliases) => {
                    let count = file_aliases.len();
                    for alias in &mut file_aliases {
                        alias.shell_source = Some("zsh".to_string());
                    }
                    aliases.extend(file_aliases);
                    println!("  Found {count} aliases");
                }
                Err(e) => {
                    println!("  Error reading file: {e}");
                }
            }
        }
    }

    Ok(aliases)
}

fn import_from_fish() -> anyhow::Result<Vec<AliasData>> {
    let mut aliases = Vec::new();
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    let fish_config_dir = home_dir.join(".config").join("fish");

    if fish_config_dir.exists() {
        let fish_files = vec![
            fish_config_dir.join("config.fish"),
            fish_config_dir.join("functions"),
        ];

        for file_path in fish_files {
            if file_path.is_file() {
                println!("Scanning {}", file_path.display());
                match extract_fish_abbreviations(&file_path) {
                    Ok(mut file_aliases) => {
                        let count = file_aliases.len();
                        for alias in &mut file_aliases {
                            alias.shell_source = Some("fish".to_string());
                        }
                        aliases.extend(file_aliases);
                        println!("  Found {count} abbreviations");
                    }
                    Err(e) => {
                        println!("  Error reading file: {e}");
                    }
                }
            } else if file_path.is_dir() {
                if let Ok(entries) = fs::read_dir(&file_path) {
                    for entry in entries.flatten() {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "fish" {
                                println!("Scanning function {}", entry.path().display());
                                println!("  Fish function files not yet supported");
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(aliases)
}

fn extract_aliases_from_shell_file(path: &Path) -> anyhow::Result<Vec<AliasData>> {
    let content = fs::read_to_string(path)?;
    let mut aliases = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("alias ") {
            if let Some(alias) = parse_alias_line(line) {
                aliases.push(alias);
            }
        }
    }

    Ok(aliases)
}

fn extract_fish_abbreviations(path: &Path) -> anyhow::Result<Vec<AliasData>> {
    let content = fs::read_to_string(path)?;
    let mut aliases = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("abbr ") {
            if let Some(alias) = parse_fish_abbr(line) {
                aliases.push(alias);
            }
        }
    }

    Ok(aliases)
}

fn parse_fish_abbr(line: &str) -> Option<AliasData> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 3 {
        return None;
    }

    let (name, command) = if parts[1] == "-a" && parts.len() >= 4 {
        (parts[2], parts[3..].join(" "))
    } else {
        (parts[1], parts[2..].join(" "))
    };

    Some(AliasData {
        name: name.to_string(),
        command,
        note: Some("Imported from Fish abbreviation".to_string()),
        tags: vec!["fish".to_string()],
        created_at: Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        shell_source: Some("fish".to_string()),
    })
}

fn append_aliases_to_file(aliases: &[AliasData]) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    if !aliases_path.exists() {
        fs::write(&aliases_path, "")?;
    }

    let mut content = fs::read_to_string(&aliases_path)?;

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    content.push_str(&format!(
        "\n# Imported aliases - {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    for alias in aliases {
        let mut alias_line = format!("alias {}='{}'", alias.name, alias.command);

        let mut comment_parts = Vec::new();
        if let Some(note) = &alias.note {
            comment_parts.push(note.clone());
        }

        if !alias.tags.is_empty() {
            comment_parts.push(format!("#tags:{}", alias.tags.join(",")));
        }

        if !comment_parts.is_empty() {
            alias_line.push_str(&format!(" # {}", comment_parts.join(" ")));
        }

        content.push_str(&alias_line);
        content.push('\n');
    }

    fs::write(&aliases_path, content)?;

    Ok(())
}
