pub mod utils;

mod commands {
    pub mod add;
    pub mod backup;
    pub mod categories;
    pub mod config;
    pub mod edit;
    pub mod import_export;
    pub mod interactive;
    pub mod list;
    pub mod plugins;
    pub mod remove;
    pub mod search;
    pub mod shell_integration;
    pub mod stats;
    pub mod sync;
    pub mod templates;
    pub mod uninstall;
    pub mod validate;
}

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
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
    Share {
        alias: String,
        #[arg(
            long,
            default_value = "clipboard",
            help = "Sharing method (clipboard, qr, file)"
        )]
        method: String,
    },
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
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

#[derive(Subcommand)]
enum SyncAction {
    Init {
        #[arg(long, help = "Remote Git repository URL")]
        remote: Option<String>,
        #[arg(long, help = "Git branch name")]
        branch: Option<String>,
    },
    Push,
    Pull,
    Status,
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },
    Reset,
}

#[derive(Subcommand)]
enum RemoteAction {
    Add {
        url: String,
        #[arg(help = "Remote name (default: origin)")]
        name: Option<String>,
    },
    List,
}

#[derive(Subcommand)]
enum PluginAction {
    List {
        #[arg(long, help = "Show all plugins (including disabled)")]
        all: bool,
    },
    Install {
        #[arg(help = "Plugin name, path, or URL")]
        plugin: String,
    },
    Remove {
        name: String,
    },
    Enable {
        name: String,
    },
    Disable {
        name: String,
    },
    Show {
        name: String,
    },
    Run {
        plugin: String,
        command: String,
        #[arg(trailing_var_arg = true, help = "Plugin command arguments")]
        args: Vec<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add {
            alias,
            command,
            note,
            tags,
        } => {
            commands::add::add_alias(alias, command, note, tags)?;
        }
        Commands::Edit {
            alias,
            new_command,
            note,
            tags,
        } => {
            commands::edit::edit_alias(alias, new_command, note, tags)?;
        }
        Commands::List { tag } => {
            commands::list::list_aliases(tag.as_deref())?;
        }
        Commands::Remove { alias } => {
            commands::remove::remove_alias(alias)?;
        }
        Commands::Search {
            keyword,
            r#in,
            regex,
        } => {
            commands::search::search_aliases(keyword, r#in.as_deref(), *regex)?;
        }
        Commands::Backup { action } => match action {
            BackupAction::Create { name } => {
                commands::backup::create_backup(name.as_deref())?;
            }
            BackupAction::Restore { backup_file } => {
                commands::backup::restore_backup(backup_file)?;
            }
            BackupAction::List => {
                commands::backup::list_backups()?;
            }
            BackupAction::Clean { older_than } => {
                commands::backup::clean_backups(*older_than)?;
            }
        },
        Commands::Validate { fix } => {
            commands::validate::validate_aliases(*fix)?;
        }
        Commands::Duplicates { remove } => {
            commands::validate::check_duplicates(*remove)?;
        }
        Commands::Interactive => {
            commands::interactive::run_interactive_mode()?;
        }
        Commands::Config { action } => match action {
            ConfigAction::Set { key, value } => {
                commands::config::set_config(key, value)?;
            }
            ConfigAction::Get { key } => {
                commands::config::get_config(key)?;
            }
            ConfigAction::List => {
                commands::config::list_config()?;
            }
            ConfigAction::Reset => {
                commands::config::reset_config()?;
            }
        },
        Commands::Stats => {
            commands::stats::show_stats()?;
        }
        Commands::Export { format, output } => {
            let format = format.parse()?;
            commands::import_export::export_aliases(format, output.as_deref())?;
        }
        Commands::Import {
            source,
            format,
            dry_run,
        } => {
            let source = source.parse()?;
            commands::import_export::import_aliases(source, format.as_deref(), *dry_run)?;
        }
        Commands::Template { action } => match action {
            TemplateAction::Add {
                name,
                pattern,
                description,
                category,
            } => {
                commands::templates::add_template(
                    name,
                    pattern,
                    description.as_deref(),
                    category.as_deref(),
                )?;
            }
            TemplateAction::List { category } => {
                commands::templates::list_templates(category.as_deref())?;
            }
            TemplateAction::Use {
                name,
                params,
                alias_name,
            } => {
                let param_map = parse_template_params(params.as_deref())?;
                commands::templates::use_template(name, &param_map, alias_name.as_deref())?;
            }
            TemplateAction::Remove { name } => {
                commands::templates::remove_template(name)?;
            }
            TemplateAction::Show { name } => {
                commands::templates::show_template(name)?;
            }
            TemplateAction::Update {
                name,
                pattern,
                description,
                category,
            } => {
                commands::templates::update_template(
                    name,
                    pattern.as_deref(),
                    description.as_deref(),
                    category.as_deref(),
                )?;
            }
        },
        Commands::Category { action } => match action {
            CategoryAction::Add {
                name,
                description,
                parent,
                color,
                icon,
            } => {
                commands::categories::add_category(
                    name,
                    description.as_deref(),
                    parent.as_deref(),
                    color.as_deref(),
                    icon.as_deref(),
                )?;
            }
            CategoryAction::List { tree, counts } => {
                commands::categories::list_categories(*tree, *counts)?;
            }
            CategoryAction::Remove { name, force } => {
                commands::categories::remove_category(name, *force)?;
            }
            CategoryAction::Move { alias, category } => {
                commands::categories::move_alias_to_category(alias, category)?;
            }
            CategoryAction::Show { name } => {
                commands::categories::show_category(name)?;
            }
            CategoryAction::Group => {
                commands::categories::group_aliases_by_category()?;
            }
        },
        Commands::Install { shell, force } => {
            let shell = shell.parse()?;
            commands::shell_integration::install_shell_integration(shell, *force)?;
        }
        Commands::Completion { shell } => {
            let shell = shell.parse()?;
            commands::shell_integration::generate_completion_script(shell)?;
        }
        Commands::Sync { action } => match action {
            SyncAction::Init { remote, branch } => {
                commands::sync::init_sync(remote.as_deref(), branch.as_deref())?;
            }
            SyncAction::Push => {
                commands::sync::push_sync()?;
            }
            SyncAction::Pull => {
                commands::sync::pull_sync()?;
            }
            SyncAction::Status => {
                commands::sync::sync_status()?;
            }
            SyncAction::Remote { action } => match action {
                RemoteAction::Add { url, name } => {
                    commands::sync::add_remote(url, name.as_deref())?;
                }
                RemoteAction::List => {
                    println!("List remotes feature coming soon");
                }
            },
            SyncAction::Reset => {
                commands::sync::reset_sync()?;
            }
        },
        Commands::Share { alias, method } => {
            commands::sync::share_alias(alias, method)?;
        }
        Commands::Plugin { action } => match action {
            PluginAction::List { all } => {
                commands::plugins::list_plugins(*all)?;
            }
            PluginAction::Install { plugin } => {
                commands::plugins::install_plugin(plugin)?;
            }
            PluginAction::Remove { name } => {
                commands::plugins::remove_plugin(name)?;
            }
            PluginAction::Enable { name } => {
                commands::plugins::enable_plugin(name)?;
            }
            PluginAction::Disable { name } => {
                commands::plugins::disable_plugin(name)?;
            }
            PluginAction::Show { name } => {
                commands::plugins::show_plugin(name)?;
            }
            PluginAction::Run {
                plugin,
                command,
                args,
            } => {
                commands::plugins::execute_plugin_command(plugin, command, args)?;
            }
        },
        Commands::Uninstall => {
            commands::uninstall::uninstall()?;
        }
    }

    Ok(())
}

fn parse_template_params(
    params_str: Option<&str>,
) -> anyhow::Result<std::collections::HashMap<String, String>> {
    let mut params = std::collections::HashMap::new();

    if let Some(params_str) = params_str {
        for param_pair in params_str.split(',') {
            let parts: Vec<&str> = param_pair.trim().splitn(2, '=').collect();
            if parts.len() == 2 {
                params.insert(parts[0].to_string(), parts[1].to_string());
            } else {
                anyhow::bail!(
                    "Invalid parameter format: '{}'. Use key=value format",
                    param_pair
                );
            }
        }
    }

    Ok(params)
}
