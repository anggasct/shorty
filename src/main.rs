use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use std::path::PathBuf;


#[derive(Parser)]
#[command(name = "shorty")]
#[command(about = "Manage your shell aliases", version = "1.0")]
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
        #[arg(short, long)]
        note: Option<String>,
    },
    /// List all aliases
    List,
    /// Remove an alias
    Remove {
        alias: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add { alias, command, note } => add_alias(alias, command, note)?,
        Commands::List => list_aliases()?,
        Commands::Remove { alias } => remove_alias(alias)?,
    }

    Ok(())
}

fn add_alias(alias: &str, command: &str, note: &Option<String>) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&aliases_path)?;

    let note_comment = note.as_ref().map(|n| format!(" # {}", n)).unwrap_or_default();
    writeln!(file, "alias {}='{}'{}", alias, command, note_comment)?;
    println!("Added alias: {} -> {}", alias, command);
    println!("To apply the changes, please restart your terminal!");

    Ok(())
}

fn list_aliases() -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;
    println!("{}", contents);

    Ok(())
}

fn remove_alias(alias: &str) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;
    let new_contents: String = contents
        .lines()
        .filter(|line| !line.starts_with(&format!("alias {}=", alias)))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&aliases_path, new_contents)?;
    println!("Removed alias: {}", alias);

    Ok(())
}

fn get_aliases_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    home_dir.join(".shorty_aliases")
}
