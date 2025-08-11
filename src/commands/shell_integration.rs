use clap::{Command, CommandFactory};
use clap_complete::{generate, Shell as CompletionShell};
use std::fs;

#[derive(Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl std::str::FromStr for Shell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            _ => anyhow::bail!("Unsupported shell: {}. Supported: bash, zsh, fish", s),
        }
    }
}

fn build_cli() -> Command {
    use clap::{Parser, Subcommand};

    #[derive(Parser)]
    #[command(name = "shorty")]
    #[command(about = "Manage your shell aliases", version = env!("CARGO_PKG_VERSION"))]
    struct Cli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(Subcommand)]
    enum Commands {
        Add {
            alias: String,
            command: String,
            #[arg(short, long, help = "Add a note to the alias")]
            note: Option<String>,
            #[arg(short, long, num_args = 1.., use_value_delimiter = true, help = "Add tags to the alias")]
            tags: Vec<String>,
        },
        Edit {
            alias: String,
            new_command: String,
            #[arg(short, long, help = "Add a new note to the alias")]
            note: Option<String>,
            #[arg(short, long, num_args = 1.., use_value_delimiter = true, help = "Add new tags to the alias")]
            tags: Vec<String>,
        },
        List {
            #[arg(short, long, help = "Filter aliases by tag")]
            tag: Option<String>,
        },
        Remove {
            alias: String,
        },
        Search {
            keyword: String,
            #[arg(long, help = "Search in specific field (command, note, tag)")]
            r#in: Option<String>,
            #[arg(long, help = "Use regex pattern matching")]
            regex: bool,
        },
        Backup {
            #[command(subcommand)]
            action: BackupAction,
        },
        Validate {
            #[arg(long, help = "Automatically fix issues where possible")]
            fix: bool,
        },
        Duplicates {
            #[arg(long, help = "Remove duplicate aliases")]
            remove: bool,
        },
        #[command(alias = "i")]
        Interactive,
        Config {
            #[command(subcommand)]
            action: ConfigAction,
        },
        Stats,
        Export {
            #[arg(long, default_value = "json", help = "Export format (json, csv, bash)")]
            format: String,
            #[arg(short, long, help = "Output file path")]
            output: Option<String>,
        },
        Import {
            #[arg(help = "Source to import from (file path, bash, zsh, fish)")]
            source: String,
            #[arg(long, help = "Source format (json, csv, bash)")]
            format: Option<String>,
            #[arg(long, help = "Preview import without making changes")]
            dry_run: bool,
        },
        Template {
            #[command(subcommand)]
            action: TemplateAction,
        },
        Category {
            #[command(subcommand)]
            action: CategoryAction,
        },
        Install {
            #[arg(long, help = "Target shell (bash, zsh, fish)")]
            shell: String,
            #[arg(long, help = "Force reinstall even if already integrated")]
            force: bool,
        },
        Completion {
            #[arg(long, help = "Target shell (bash, zsh, fish)")]
            shell: String,
        },
        Uninstall,
    }

    #[derive(Subcommand)]
    enum BackupAction {
        Create {
            #[arg(long, help = "Custom backup name")]
            name: Option<String>,
        },
        Restore {
            backup_file: String,
        },
        List,
        Clean {
            #[arg(long, default_value = "30", help = "Remove backups older than N days")]
            older_than: u32,
        },
    }

    #[derive(Subcommand)]
    enum ConfigAction {
        Set { key: String, value: String },
        Get { key: String },
        List,
        Reset,
    }

    #[derive(Subcommand)]
    enum TemplateAction {
        Add {
            name: String,
            pattern: String,
            #[arg(short, long, help = "Template description")]
            description: Option<String>,
            #[arg(short, long, help = "Template category")]
            category: Option<String>,
        },
        List {
            #[arg(short, long, help = "Filter by category")]
            category: Option<String>,
        },
        Use {
            name: String,
            #[arg(long, help = "Template parameters (key=value,key2=value2)")]
            params: Option<String>,
            #[arg(short, long, help = "Custom alias name")]
            alias_name: Option<String>,
        },
        Remove {
            name: String,
        },
        Show {
            name: String,
        },
        Update {
            name: String,
            #[arg(long, help = "New pattern")]
            pattern: Option<String>,
            #[arg(long, help = "New description")]
            description: Option<String>,
            #[arg(long, help = "New category")]
            category: Option<String>,
        },
    }

    #[derive(Subcommand)]
    enum CategoryAction {
        Add {
            name: String,
            #[arg(short, long, help = "Category description")]
            description: Option<String>,
            #[arg(short, long, help = "Parent category")]
            parent: Option<String>,
            #[arg(short, long, help = "Category color")]
            color: Option<String>,
            #[arg(short, long, help = "Category icon")]
            icon: Option<String>,
        },
        List {
            #[arg(long, help = "Show as tree structure")]
            tree: bool,
            #[arg(long, help = "Show alias counts")]
            counts: bool,
        },
        Remove {
            name: String,
            #[arg(long, help = "Force removal even if category has children or aliases")]
            force: bool,
        },
        Move {
            alias: String,
            category: String,
        },
        Show {
            name: String,
        },
        Group,
    }

    Cli::command()
}

pub fn generate_completion_script(shell: Shell) -> anyhow::Result<()> {
    let completion_shell = match shell {
        Shell::Bash => CompletionShell::Bash,
        Shell::Zsh => CompletionShell::Zsh,
        Shell::Fish => CompletionShell::Fish,
    };

    let shell_name = match shell {
        Shell::Bash => "bash",
        Shell::Zsh => "zsh",
        Shell::Fish => "fish",
    };

    let mut cmd = build_cli();
    let output_path = format!("shorty_completion.{shell_name}");

    let mut file = fs::File::create(&output_path)?;
    generate(completion_shell, &mut cmd, "shorty", &mut file);

    println!("Generated {shell_name} completion script: {output_path}");
    println!("Installation instructions:");

    match shell {
        Shell::Bash => {
            println!("  1. Copy to system completion directory:");
            println!("     sudo cp {output_path} /etc/bash_completion.d/shorty");
            println!("  2. Or source in your ~/.bashrc:");
            println!("     echo 'source ~/{output_path}'.bashrc");
        }
        Shell::Zsh => {
            println!("  1. Add to your fpath in ~/.zshrc:");
            println!("     fpath=(~/.zsh/completions $fpath)");
            println!("     mkdir -p ~/.zsh/completions");
            println!("     cp {output_path} ~/.zsh/completions/_shorty");
            println!("  2. Restart zsh or run: autoload -U compinit && compinit");
        }
        Shell::Fish => {
            println!("  1. Copy to fish completions directory:");
            println!("     cp {output_path} ~/.config/fish/completions/");
            println!("  2. Completions will be available immediately");
        }
    }

    Ok(())
}
