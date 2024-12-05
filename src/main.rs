pub mod utils;

mod commands {
    pub mod add;
    pub mod edit;
    pub mod list;
    pub mod remove;
    pub mod search;
    pub mod uninstall;
}

use clap::{Parser, Subcommand};


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
    /// Edit an existing alias
    Edit {
        alias: String,
        new_command: String,
        #[arg(short, long, help = "Add a new note to the alias")]
        note: Option<String>,
        #[arg(short, long, num_args = 1.., use_value_delimiter = true, help = "Add new tags to the alias")]
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
    /// Uninstall shorty
    Uninstall,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add { alias, command, note, tags } => {
            commands::add::add_alias(alias, command, note, tags)?;
        }
        Commands::Edit { alias, new_command, note, tags } => {
            commands::edit::edit_alias(alias, new_command, note, tags)?;
        }
        Commands::List { tag } => {
            commands::list::list_aliases(tag.as_deref())?;
        }
        Commands::Remove { alias } => {
            commands::remove::remove_alias(alias)?;
        }
        Commands::Search { keyword } => {
            commands::search::search_aliases(keyword)?;
        }
        Commands::Uninstall => {
            commands::uninstall::uninstall()?;
        }
    }

    Ok(())
}