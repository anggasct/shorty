use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub backup: BackupConfig,
    pub display: DisplayConfig,
    pub search: SearchConfig,
    pub aliases: AliasConfig,
    pub update: UpdateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub auto_backup: bool,
    pub max_backups: u32,
    pub backup_before_edit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub color_output: bool,
    pub show_line_numbers: bool,
    pub truncate_commands: bool,
    pub max_command_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub fuzzy_matching: bool,
    pub case_sensitive: bool,
    pub search_in_notes: bool,
    pub search_in_tags: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasConfig {
    pub file_path: String,
    pub sort_on_add: bool,
    pub validate_on_add: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub enabled: bool,
    pub check_interval_hours: i64,
    pub auto_download: bool,
    pub backup_old_versions: bool,
    pub max_backups: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backup: BackupConfig {
                auto_backup: true,
                max_backups: 10,
                backup_before_edit: true,
            },
            display: DisplayConfig {
                color_output: true,
                show_line_numbers: false,
                truncate_commands: true,
                max_command_length: 50,
            },
            search: SearchConfig {
                fuzzy_matching: false,
                case_sensitive: false,
                search_in_notes: true,
                search_in_tags: true,
            },
            aliases: AliasConfig {
                file_path: "~/.shorty/aliases".to_string(),
                sort_on_add: false,
                validate_on_add: true,
            },
            update: UpdateConfig {
                enabled: true,
                check_interval_hours: 24,
                auto_download: true,
                backup_old_versions: true,
                max_backups: 3,
            },
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = get_config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = get_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "backup.auto_backup" => Some(self.backup.auto_backup.to_string()),
            "backup.max_backups" => Some(self.backup.max_backups.to_string()),
            "backup.backup_before_edit" => Some(self.backup.backup_before_edit.to_string()),

            "display.color_output" => Some(self.display.color_output.to_string()),
            "display.show_line_numbers" => Some(self.display.show_line_numbers.to_string()),
            "display.truncate_commands" => Some(self.display.truncate_commands.to_string()),
            "display.max_command_length" => Some(self.display.max_command_length.to_string()),

            "search.fuzzy_matching" => Some(self.search.fuzzy_matching.to_string()),
            "search.case_sensitive" => Some(self.search.case_sensitive.to_string()),
            "search.search_in_notes" => Some(self.search.search_in_notes.to_string()),
            "search.search_in_tags" => Some(self.search.search_in_tags.to_string()),

            "aliases.file_path" => Some(self.aliases.file_path.clone()),
            "aliases.sort_on_add" => Some(self.aliases.sort_on_add.to_string()),
            "aliases.validate_on_add" => Some(self.aliases.validate_on_add.to_string()),

            "update.enabled" => Some(self.update.enabled.to_string()),
            "update.check_interval_hours" => Some(self.update.check_interval_hours.to_string()),
            "update.auto_download" => Some(self.update.auto_download.to_string()),
            "update.backup_old_versions" => Some(self.update.backup_old_versions.to_string()),
            "update.max_backups" => Some(self.update.max_backups.to_string()),

            _ => None,
        }
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        match key {
            "backup.auto_backup" => {
                self.backup.auto_backup = parse_bool(value)?;
            }
            "backup.max_backups" => {
                self.backup.max_backups = value.parse()?;
            }
            "backup.backup_before_edit" => {
                self.backup.backup_before_edit = parse_bool(value)?;
            }

            "display.color_output" => {
                self.display.color_output = parse_bool(value)?;
            }
            "display.show_line_numbers" => {
                self.display.show_line_numbers = parse_bool(value)?;
            }
            "display.truncate_commands" => {
                self.display.truncate_commands = parse_bool(value)?;
            }
            "display.max_command_length" => {
                self.display.max_command_length = value.parse()?;
            }

            "search.fuzzy_matching" => {
                self.search.fuzzy_matching = parse_bool(value)?;
            }
            "search.case_sensitive" => {
                self.search.case_sensitive = parse_bool(value)?;
            }
            "search.search_in_notes" => {
                self.search.search_in_notes = parse_bool(value)?;
            }
            "search.search_in_tags" => {
                self.search.search_in_tags = parse_bool(value)?;
            }

            "aliases.file_path" => {
                self.aliases.file_path = value.to_string();
            }
            "aliases.sort_on_add" => {
                self.aliases.sort_on_add = parse_bool(value)?;
            }
            "aliases.validate_on_add" => {
                self.aliases.validate_on_add = parse_bool(value)?;
            }

            "update.enabled" => {
                self.update.enabled = parse_bool(value)?;
            }
            "update.check_interval_hours" => {
                self.update.check_interval_hours = value.parse()?;
            }
            "update.auto_download" => {
                self.update.auto_download = parse_bool(value)?;
            }
            "update.backup_old_versions" => {
                self.update.backup_old_versions = parse_bool(value)?;
            }
            "update.max_backups" => {
                self.update.max_backups = value.parse()?;
            }

            _ => {
                anyhow::bail!("Unknown configuration key: {}", key);
            }
        }

        Ok(())
    }

    pub fn get_all_keys(&self) -> Vec<(String, String)> {
        vec![
            (
                "backup.auto_backup".to_string(),
                "Automatically create backups before destructive operations".to_string(),
            ),
            (
                "backup.max_backups".to_string(),
                "Maximum number of backup files to keep".to_string(),
            ),
            (
                "backup.backup_before_edit".to_string(),
                "Create backup before editing aliases".to_string(),
            ),
            (
                "display.color_output".to_string(),
                "Enable colored output in terminal".to_string(),
            ),
            (
                "display.show_line_numbers".to_string(),
                "Show line numbers in alias listings".to_string(),
            ),
            (
                "display.truncate_commands".to_string(),
                "Truncate long commands in listings".to_string(),
            ),
            (
                "display.max_command_length".to_string(),
                "Maximum command length before truncation".to_string(),
            ),
            (
                "search.fuzzy_matching".to_string(),
                "Enable fuzzy matching in searches".to_string(),
            ),
            (
                "search.case_sensitive".to_string(),
                "Make searches case sensitive".to_string(),
            ),
            (
                "search.search_in_notes".to_string(),
                "Include notes in search results".to_string(),
            ),
            (
                "search.search_in_tags".to_string(),
                "Include tags in search results".to_string(),
            ),
            (
                "aliases.file_path".to_string(),
                "Path to the aliases file".to_string(),
            ),
            (
                "aliases.sort_on_add".to_string(),
                "Automatically sort aliases when adding new ones".to_string(),
            ),
            (
                "aliases.validate_on_add".to_string(),
                "Validate aliases when adding new ones".to_string(),
            ),
            (
                "update.enabled".to_string(),
                "Enable automatic update checking".to_string(),
            ),
            (
                "update.check_interval_hours".to_string(),
                "Hours between update checks".to_string(),
            ),
            (
                "update.auto_download".to_string(),
                "Automatically download updates (still requires confirmation)".to_string(),
            ),
            (
                "update.backup_old_versions".to_string(),
                "Backup old binary before updating".to_string(),
            ),
            (
                "update.max_backups".to_string(),
                "Maximum number of binary backups to keep".to_string(),
            ),
        ]
    }
}

pub fn set_config(key: &str, value: &str) -> anyhow::Result<()> {
    let mut config = Config::load()?;
    config.set_value(key, value)?;
    config.save()?;

    println!("Configuration updated: {key} = {value}");
    Ok(())
}

pub fn get_config(key: &str) -> anyhow::Result<()> {
    let config = Config::load()?;

    if let Some(value) = config.get_value(key) {
        println!("{key} = {value}");
    } else {
        println!("Unknown configuration key: {key}");
        println!("\nAvailable keys:");
        for (k, description) in config.get_all_keys() {
            println!("  {k} - {description}");
        }
    }

    Ok(())
}

pub fn list_config() -> anyhow::Result<()> {
    let config = Config::load()?;

    println!("Current Configuration:\n");

    println!("Backup:");
    println!("  auto_backup         = {}", config.backup.auto_backup);
    println!("  max_backups         = {}", config.backup.max_backups);
    println!(
        "  backup_before_edit  = {}",
        config.backup.backup_before_edit
    );

    println!("\nDisplay:");
    println!("  color_output        = {}", config.display.color_output);
    println!(
        "  show_line_numbers   = {}",
        config.display.show_line_numbers
    );
    println!(
        "  truncate_commands   = {}",
        config.display.truncate_commands
    );
    println!(
        "  max_command_length  = {}",
        config.display.max_command_length
    );

    println!("\nSearch:");
    println!("  fuzzy_matching      = {}", config.search.fuzzy_matching);
    println!("  case_sensitive      = {}", config.search.case_sensitive);
    println!("  search_in_notes     = {}", config.search.search_in_notes);
    println!("  search_in_tags      = {}", config.search.search_in_tags);

    println!("\nAliases:");
    println!("  file_path           = {}", config.aliases.file_path);
    println!("  sort_on_add         = {}", config.aliases.sort_on_add);
    println!("  validate_on_add     = {}", config.aliases.validate_on_add);

    println!("\nUpdate:");
    println!("  enabled             = {}", config.update.enabled);
    println!("  check_interval_hours= {}", config.update.check_interval_hours);
    println!("  auto_download       = {}", config.update.auto_download);
    println!("  backup_old_versions = {}", config.update.backup_old_versions);
    println!("  max_backups         = {}", config.update.max_backups);

    println!("\nUse 'shorty config set <key> <value>' to change settings");

    Ok(())
}

pub fn reset_config() -> anyhow::Result<()> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        fs::remove_file(&config_path)?;
        println!("Removed existing configuration file");
    }

    let default_config = Config::default();
    default_config.save()?;

    println!("Configuration reset to defaults");
    println!("Configuration file: {}", config_path.display());

    Ok(())
}

fn get_config_path() -> anyhow::Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    Ok(home_dir.join(".shorty").join("config.toml"))
}

fn parse_bool(value: &str) -> anyhow::Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" | "enable" | "enabled" => Ok(true),
        "false" | "0" | "no" | "off" | "disable" | "disabled" => Ok(false),
        _ => anyhow::bail!(
            "Invalid boolean value: '{}'. Use true/false, yes/no, on/off, or 1/0",
            value
        ),
    }
}

#[allow(dead_code)]
pub fn load_config() -> Config {
    Config::load().unwrap_or_default()
}
