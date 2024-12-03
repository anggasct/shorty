use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "shorty")]
#[command(about = "Manage your shell aliases", version = "1.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new alias
    Add {
        alias: String,
        command: String,
        #[arg(short, long, help = "Add a note to the alias")]
        note: Option<String>,
        #[arg(short, long, num_args = 1.., use_value_delimiter = true, help = "Add tags to the alias")]
        tags: Vec<String>,
    },
    /// List all aliases
    #[command(alias = "ls")]
    List {
        #[arg(short, long, help = "Filter aliases by tag")]
        tag: Option<String>,
    },
    /// Remove an alias
    #[command(alias = "rm")]
    Remove { alias: String },
    /// Search aliases
    Search { keyword: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add {
            alias,
            command,
            note,
            tags,
        } => add_alias(alias, command, note, tags)?,
        Commands::List { tag } => {
            if let Some(tag) = tag {
                filter_aliases_by_tag(tag)?
            } else {
                list_aliases()?
            }
        }
        Commands::Remove { alias } => remove_alias(alias)?,
        Commands::Search { keyword } => search_aliases(keyword)?,
    }

    Ok(())
}

fn add_alias(
    alias: &str,
    command: &str,
    note: &Option<String>,
    tags: &[String],
) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    if alias_exists(&aliases_path, alias)? {
        print!(
            "Warning: Alias '{}' already exists. Do you want to overwrite it? (y/n): ",
            alias
        );
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
        .write(true)
        .open(&aliases_path)?;

    let tags_str = if tags.is_empty() {
        String::new()
    } else {
        format!(" #tags:{}", tags.join(","))
    };

    let note_comment = note
        .as_ref()
        .map(|n| format!(" # {}", n))
        .unwrap_or_default();

    writeln!(
        file,
        "alias {}='{}'{}{}",
        alias, command, note_comment, tags_str
    )?;

    println!("Added alias: {} -> {}", alias, command);
    println!("To apply the changes, please restart your terminal!");

    Ok(())
}

fn alias_exists(aliases_path: &PathBuf, alias: &str) -> io::Result<bool> {
    if let Ok(file) = fs::File::open(aliases_path) {
        for line in io::BufReader::new(file).lines() {
            if let Ok(line) = line {
                if line.starts_with(&format!("alias {}=", alias)) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn list_aliases() -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;
    println!("{}", contents);

    Ok(())
}

fn filter_aliases_by_tag(tag: &str) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;
    let filtered: Vec<&str> = contents
        .lines()
        .filter(|line| line.contains(&format!("#tags:{}", tag)))
        .collect();

    if filtered.is_empty() {
        println!("No aliases found with tag: {}", tag);
    } else {
        for alias in filtered {
            println!("{}", alias);
        }
    }

    Ok(())
}

fn remove_alias(alias: &str) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;
    let mut new_contents: String = contents
        .lines()
        .filter(|line| !line.starts_with(&format!("alias {}=", alias)))
        .collect::<Vec<_>>()
        .join("\n");

    // Ensure the new contents end with a newline
    if !new_contents.ends_with('\n') {
        new_contents.push('\n');
    }

    fs::write(&aliases_path, new_contents)?;
    println!("Removed alias: {}", alias);

    Ok(())
}

fn search_aliases(query: &str) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;

    let results: Vec<&str> = contents
        .lines()
        .filter(|line| line.contains(query))
        .collect();

    if results.is_empty() {
        println!("No aliases found matching: {}", query);
    } else {
        println!("Search results for '{}':", query);
        for line in results {
            println!("{}", line);
        }
    }

    Ok(())
}

fn get_aliases_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    home_dir.join(".shorty_aliases")
}
