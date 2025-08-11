use crate::utils::get_aliases_path;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub name: String,
    pub description: String,
    pub parent: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
    pub alias_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoriesData {
    version: String,
    categories: Vec<Category>,
}

pub fn add_category(
    name: &str,
    description: Option<&str>,
    parent: Option<&str>,
    color: Option<&str>,
    icon: Option<&str>,
) -> anyhow::Result<()> {
    let mut categories = load_categories()?;

    if categories.iter().any(|c| c.name == name) {
        anyhow::bail!("Category '{}' already exists", name);
    }

    if let Some(parent_name) = parent {
        if !categories.iter().any(|c| c.name == parent_name) {
            anyhow::bail!("Parent category '{}' does not exist", parent_name);
        }
    }

    let category = Category {
        name: name.to_string(),
        description: description.unwrap_or("No description").to_string(),
        parent: parent.map(|p| p.to_string()),
        color: color.map(|c| c.to_string()),
        icon: icon.map(|i| i.to_string()),
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        alias_count: 0,
    };

    categories.push(category);
    save_categories(&categories)?;

    println!("Category '{name}' added successfully");
    if let Some(desc) = description {
        println!("Description: {desc}");
    }
    if let Some(parent_name) = parent {
        println!("Parent: {parent_name}");
    }

    Ok(())
}

pub fn list_categories(show_tree: bool, show_counts: bool) -> anyhow::Result<()> {
    let mut categories = load_categories()?;

    if categories.is_empty() {
        println!("No categories found. Create your first category with 'shorty category add'");
        return Ok(());
    }

    if show_counts {
        update_alias_counts(&mut categories)?;
        save_categories(&categories)?;
    }

    if show_tree {
        display_category_tree(&categories)?;
    } else {
        display_category_list(&categories, show_counts)?;
    }

    Ok(())
}

pub fn remove_category(name: &str, force: bool) -> anyhow::Result<()> {
    let mut categories = load_categories()?;

    let category_index = categories
        .iter()
        .position(|c| c.name == name)
        .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", name))?;

    let has_children = categories
        .iter()
        .any(|c| c.parent.as_ref() == Some(&name.to_string()));

    if has_children && !force {
        anyhow::bail!("Category '{}' has child categories. Use --force to remove it and move children to root level", name);
    }

    let alias_count = count_aliases_in_category(name)?;
    if alias_count > 0 && !force {
        anyhow::bail!("Category '{}' contains {} aliases. Use --force to remove category (aliases will become uncategorized)", name, alias_count);
    }

    if has_children && force {
        for category in &mut categories {
            if category.parent.as_ref() == Some(&name.to_string()) {
                category.parent = None;
                println!("Moved '{}' to root level", category.name);
            }
        }
    }

    categories.remove(category_index);
    save_categories(&categories)?;

    println!("Category '{name}' removed successfully");
    if alias_count > 0 {
        println!("{alias_count} aliases are now uncategorized");
    }

    Ok(())
}

pub fn move_alias_to_category(alias_name: &str, category_name: &str) -> anyhow::Result<()> {
    let categories = load_categories()?;

    if !categories.iter().any(|c| c.name == category_name) {
        anyhow::bail!("Category '{}' does not exist", category_name);
    }

    let aliases_path = get_aliases_path();
    if !aliases_path.exists() {
        anyhow::bail!("No aliases file found");
    }

    let content = fs::read_to_string(&aliases_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut found = false;

    for line in &mut lines {
        if let Some((name, command, note, mut tags)) = parse_alias_line(line) {
            if name == alias_name {
                found = true;

                tags.retain(|tag| !tag.starts_with("category:"));

                tags.push(format!("category:{category_name}"));

                *line = build_alias_line(&name, &command, note.as_deref(), &tags);
                break;
            }
        }
    }

    if !found {
        anyhow::bail!("Alias '{}' not found", alias_name);
    }

    let new_content = lines.join("\n");
    fs::write(&aliases_path, new_content)?;

    println!("Moved alias '{alias_name}' to category '{category_name}'");

    Ok(())
}

pub fn show_category(name: &str) -> anyhow::Result<()> {
    let mut categories = load_categories()?;

    let category = categories
        .iter_mut()
        .find(|c| c.name == name)
        .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", name))?;

    category.alias_count = count_aliases_in_category(name)?;

    println!("Category: {}", category.name);
    println!("Description: {}", category.description);

    if let Some(parent) = &category.parent {
        println!("Parent: {parent}");
    }

    if let Some(color) = &category.color {
        println!("Color: {color}");
    }

    if let Some(icon) = &category.icon {
        println!("Icon: {icon}");
    }

    println!("Aliases: {}", category.alias_count);
    println!("Created: {}", category.created_at);

    let children: Vec<_> = categories
        .iter()
        .filter(|c| c.parent.as_ref() == Some(&name.to_string()))
        .collect();

    if !children.is_empty() {
        println!("\nChild Categories:");
        for child in children {
            println!("  • {}", child.name);
        }
    }

    let aliases = get_aliases_in_category(name)?;
    if !aliases.is_empty() {
        println!("\nAliases in this category:");
        for (alias_name, command) in aliases {
            let display_command = if command.len() > 50 {
                format!("{}...", &command[..47])
            } else {
                command
            };
            println!("  • {alias_name} → {display_command}");
        }
    }

    Ok(())
}

pub fn group_aliases_by_category() -> anyhow::Result<()> {
    let categories = load_categories()?;
    let aliases_path = get_aliases_path();

    if !aliases_path.exists() {
        println!("No aliases file found");
        return Ok(());
    }

    let content = fs::read_to_string(&aliases_path)?;
    let mut categorized_aliases: HashMap<String, Vec<(String, String, Option<String>)>> =
        HashMap::new();
    let mut uncategorized_aliases = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((name, command, note, tags)) = parse_alias_line(line) {
            let category = tags
                .iter()
                .find(|tag| tag.starts_with("category:"))
                .map(|tag| tag[9..].to_string())
                .unwrap_or_else(|| "uncategorized".to_string());

            if category == "uncategorized" {
                uncategorized_aliases.push((name, command, note));
            } else {
                categorized_aliases
                    .entry(category)
                    .or_default()
                    .push((name, command, note));
            }
        }
    }

    println!("Aliases grouped by category:\n");

    for (category_name, aliases) in &categorized_aliases {
        let category_info = categories.iter().find(|c| c.name == *category_name);

        let display_name = if let Some(cat) = category_info {
            if let Some(icon) = &cat.icon {
                format!("{icon} {category_name}")
            } else {
                category_name.to_string()
            }
        } else {
            format!("{category_name} (category not found)")
        };

        println!("{} ({} aliases):", display_name, aliases.len());

        for (alias_name, command, note) in aliases {
            let display_command = if command.len() > 40 {
                format!("{}...", &command[..37])
            } else {
                command.clone()
            };

            if let Some(note_text) = note {
                println!("  • {alias_name} → {display_command} # {note_text}");
            } else {
                println!("  • {alias_name} → {display_command}");
            }
        }
        println!();
    }

    if !uncategorized_aliases.is_empty() {
        println!("Uncategorized ({} aliases):", uncategorized_aliases.len());
        for (alias_name, command, note) in &uncategorized_aliases {
            let display_command = if command.len() > 40 {
                format!("{}...", &command[..37])
            } else {
                command.clone()
            };

            if let Some(note_text) = note {
                println!("  • {alias_name} → {display_command} # {note_text}");
            } else {
                println!("  • {alias_name} → {display_command}");
            }
        }
        println!();
    }

    let total_categorized: usize = categorized_aliases.values().map(|v| v.len()).sum();
    let total_aliases = total_categorized + uncategorized_aliases.len();

    println!("Summary:");
    println!("  Total aliases: {total_aliases}");
    println!(
        "  Categorized: {} ({:.1}%)",
        total_categorized,
        if total_aliases > 0 {
            (total_categorized as f64 / total_aliases as f64) * 100.0
        } else {
            0.0
        }
    );
    println!(
        "  Uncategorized: {} ({:.1}%)",
        uncategorized_aliases.len(),
        if total_aliases > 0 {
            (uncategorized_aliases.len() as f64 / total_aliases as f64) * 100.0
        } else {
            0.0
        }
    );

    if !uncategorized_aliases.is_empty() {
        println!("\nSuggestions:");
        let command_patterns = analyze_command_patterns(&uncategorized_aliases);
        for (pattern, count) in command_patterns {
            if count > 1 {
                println!("  • Create '{pattern}' category for {count} similar commands");
            }
        }
    }

    Ok(())
}

fn load_categories() -> anyhow::Result<Vec<Category>> {
    let categories_path = get_categories_path()?;

    if !categories_path.exists() {
        let default_categories = create_default_categories();
        save_categories(&default_categories)?;
        return Ok(default_categories);
    }

    let content = fs::read_to_string(&categories_path)?;
    let data: CategoriesData = toml::from_str(&content)?;

    Ok(data.categories)
}

fn save_categories(categories: &[Category]) -> anyhow::Result<()> {
    let categories_path = get_categories_path()?;

    if let Some(parent) = categories_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = CategoriesData {
        version: "1.0".to_string(),
        categories: categories.to_vec(),
    };

    let content = toml::to_string_pretty(&data)?;
    fs::write(&categories_path, content)?;

    Ok(())
}

fn get_categories_path() -> anyhow::Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    Ok(home_dir.join(".shorty").join("categories.toml"))
}

fn update_alias_counts(categories: &mut [Category]) -> anyhow::Result<()> {
    for category in categories {
        category.alias_count = count_aliases_in_category(&category.name)?;
    }
    Ok(())
}

fn count_aliases_in_category(category_name: &str) -> anyhow::Result<usize> {
    let aliases = get_aliases_in_category(category_name)?;
    Ok(aliases.len())
}

fn get_aliases_in_category(category_name: &str) -> anyhow::Result<Vec<(String, String)>> {
    let aliases_path = get_aliases_path();
    if !aliases_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&aliases_path)?;
    let mut aliases = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((name, command, _note, tags)) = parse_alias_line(line) {
            if tags
                .iter()
                .any(|tag| tag == &format!("category:{category_name}"))
            {
                aliases.push((name, command));
            }
        }
    }

    Ok(aliases)
}

fn display_category_tree(categories: &[Category]) -> anyhow::Result<()> {
    println!("Category Tree:\n");

    let root_categories: Vec<_> = categories.iter().filter(|c| c.parent.is_none()).collect();

    for root in root_categories {
        display_category_node(root, categories, 0);
    }

    Ok(())
}

fn display_category_node(category: &Category, all_categories: &[Category], depth: usize) {
    let indent = "  ".repeat(depth);
    let icon = category.icon.as_deref().unwrap_or("[FOLDER]");

    println!(
        "{}{} {} ({} aliases)",
        indent, icon, category.name, category.alias_count
    );

    let children: Vec<_> = all_categories
        .iter()
        .filter(|c| c.parent.as_ref() == Some(&category.name))
        .collect();

    for child in children {
        display_category_node(child, all_categories, depth + 1);
    }
}

fn display_category_list(categories: &[Category], show_counts: bool) -> anyhow::Result<()> {
    println!("Categories:\n");

    for category in categories {
        let icon = category.icon.as_deref().unwrap_or("[FOLDER]");

        print!("{} {}", icon, category.name);

        if let Some(parent) = &category.parent {
            print!(" (child of {parent})");
        }

        if show_counts {
            print!(" - {} aliases", category.alias_count);
        }

        println!();

        if !category.description.is_empty() && category.description != "No description" {
            println!("    {}", category.description);
        }

        println!();
    }

    Ok(())
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
    if let Some(stripped) = rest.strip_prefix('\'') {
        if let Some(end_quote) = stripped.find('\'') {
            command = rest[1..end_quote + 1].to_string();
            remaining = &rest[end_quote + 2..];
        }
    } else if let Some(stripped) = rest.strip_prefix('"') {
        if let Some(end_quote) = stripped.find('"') {
            command = rest[1..end_quote + 1].to_string();
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
        let comment_text = remaining.trim()[1..].trim();
        if comment_text.contains("#tags:") {
            if let Some(tags_pos) = comment_text.find("#tags:") {
                let note_part = comment_text[..tags_pos].trim();
                if !note_part.is_empty() {
                    note = Some(note_part.to_string());
                }
                let tags_part = &comment_text[tags_pos + 6..];
                tags = tags_part.split(',').map(|s| s.trim().to_string()).collect();
            }
        } else if !comment_text.is_empty() {
            note = Some(comment_text.to_string());
        }
    }

    Some((name, command, note, tags))
}

fn build_alias_line(name: &str, command: &str, note: Option<&str>, tags: &[String]) -> String {
    let mut line = format!("alias {name}='{command}'");

    let mut comment_parts = Vec::new();

    if let Some(note_text) = note {
        comment_parts.push(note_text.to_string());
    }

    if !tags.is_empty() {
        comment_parts.push(format!("#tags:{}", tags.join(",")));
    }

    if !comment_parts.is_empty() {
        line.push_str(&format!(" # {}", comment_parts.join(" ")));
    }

    line
}

fn analyze_command_patterns(aliases: &[(String, String, Option<String>)]) -> Vec<(String, usize)> {
    let mut patterns: HashMap<String, usize> = HashMap::new();

    for (_, command, _) in aliases {
        let first_word = command.split_whitespace().next().unwrap_or(command);

        let pattern = match first_word {
            cmd if cmd.starts_with("git") => "git",
            "docker" | "docker-compose" => "docker",
            "npm" | "yarn" | "pnpm" => "nodejs",
            "kubectl" | "k8s" => "kubernetes",
            "ssh" | "scp" | "rsync" => "network",
            "ls" | "ll" | "la" | "dir" => "listing",
            "cd" | "pushd" | "popd" => "navigation",
            "cat" | "less" | "more" | "head" | "tail" => "viewing",
            _ => "general",
        };

        *patterns.entry(pattern.to_string()).or_insert(0) += 1;
    }

    let mut pattern_vec: Vec<_> = patterns.into_iter().collect();
    pattern_vec.sort_by(|a, b| b.1.cmp(&a.1));
    pattern_vec
}

fn create_default_categories() -> Vec<Category> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    vec![
        Category {
            name: "git".to_string(),
            description: "Git version control commands".to_string(),
            parent: None,
            color: Some("orange".to_string()),
            icon: Some("🔀".to_string()),
            created_at: timestamp.clone(),
            alias_count: 0,
        },
        Category {
            name: "docker".to_string(),
            description: "Docker and containerization commands".to_string(),
            parent: None,
            color: Some("blue".to_string()),
            icon: Some("🐳".to_string()),
            created_at: timestamp.clone(),
            alias_count: 0,
        },
        Category {
            name: "nodejs".to_string(),
            description: "Node.js and npm commands".to_string(),
            parent: None,
            color: Some("green".to_string()),
            icon: Some("📦".to_string()),
            created_at: timestamp.clone(),
            alias_count: 0,
        },
        Category {
            name: "network".to_string(),
            description: "Network and SSH commands".to_string(),
            parent: None,
            color: Some("purple".to_string()),
            icon: Some("🌐".to_string()),
            created_at: timestamp.clone(),
            alias_count: 0,
        },
        Category {
            name: "system".to_string(),
            description: "System administration commands".to_string(),
            parent: None,
            color: Some("red".to_string()),
            icon: Some("⚙️".to_string()),
            created_at: timestamp,
            alias_count: 0,
        },
    ]
}
