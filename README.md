# Shorty

**A shell alias manager for organizing and maintaining command-line aliases.**

Shorty provides tools for managing shell aliases, including backup & restore, template system, interactive terminal UI, category organization, and import/export capabilities. Written in Rust.

[![Version](https://img.shields.io/badge/version-1.4.0-blue.svg)](https://github.com/anggasct/shorty)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## Key Features

- **Alias Management**: Add, edit, list, remove, and search aliases with notes and tags
- **Backup & Restore**: Automatic backups with easy recovery
- **Validation**: Check for duplicate aliases and validate commands
- **Interactive UI**: Terminal interface with keyboard navigation
- **Configuration**: TOML-based settings for customization
- **Statistics**: View usage analytics and command insights
- **Import/Export**: Support for JSON, CSV, and shell script formats
- **Templates**: Create reusable alias templates with parameters
- **Categories**: Organize aliases with hierarchical categories
- **Shell Integration**: Tab completion for bash, zsh, and fish

## Quick Start

### Installation

#### Using Curl (Recommended)

```bash
curl -sSfL https://github.com/anggasct/shorty/raw/main/install.sh | sh
```

#### From Source

```bash
git clone https://github.com/anggasct/shorty.git
cd shorty
cargo build --release
sudo cp target/release/shorty /usr/local/bin/
```

### Basic Usage

```bash
# Add a new alias with note and tags
shorty add gs "git status" --note "Quick git status" --tags git,status

# List all aliases
shorty list

# Search aliases
shorty search git --in command

# Edit an existing alias
shorty edit gs "git status -sb" --note "Short git status"

# Remove an alias
shorty remove gs
```

## Comprehensive Command Reference

### **Core Commands**

#### **Add Alias**

```bash
shorty add <alias> <command> [OPTIONS]
```

**Options:**

- `--note, -n <NOTE>`: Add descriptive note
- `--tags, -t <TAGS>`: Comma-separated tags for organization

**Examples:**

```bash
shorty add ll "ls -la" --note "Detailed file listing" --tags list,files
shorty add gp "git push origin main" --tags git,push
```

#### **List Aliases**

```bash
shorty list [OPTIONS]
shorty ls [OPTIONS]    # Short alias
```

**Options:**

- `--tag <TAG>`: Filter by specific tag

**Examples:**

```bash
shorty list              # All aliases
shorty list --tag git   # Only git-related aliases
```

#### **Search Aliases**

```bash
shorty search <KEYWORD> [OPTIONS]
```

**Options:**

- `--in <FIELD>`: Search in specific field (command, note, tag)
- `--regex`: Use regex pattern matching

**Examples:**

```bash
shorty search docker                    # General search
shorty search "git" --in command       # Search only in commands
shorty search "test.*unit" --regex     # Regex search
```

#### **Edit Alias**

```bash
shorty edit <alias> <new_command> [OPTIONS]
```

**Options:**

- `--note, -n <NOTE>`: Update note
- `--tags, -t <TAGS>`: Update tags

#### **Remove Alias**

```bash
shorty remove <alias>
shorty rm <alias>      # Short alias
```

### **Backup & Recovery**

#### **Create Backup**

```bash
shorty backup create [OPTIONS]
```

**Options:**

- `--name <NAME>`: Custom backup name

**Examples:**

```bash
shorty backup create                        # Timestamped backup
shorty backup create --name "before-update" # Named backup
```

#### **List Backups**

```bash
shorty backup list
```

#### **Restore from Backup**

```bash
shorty backup restore <backup-file>
```

#### **Clean Old Backups**

```bash
shorty backup clean [OPTIONS]
```

**Options:**

- `--older-than <DAYS>`: Remove backups older than N days (default: 30)

### **Validation & Health**

#### **Validate Aliases**

```bash
shorty validate [OPTIONS]
```

**Options:**

- `--fix`: Automatically fix issues where possible

#### **Check Duplicates**

```bash
shorty duplicates [OPTIONS]
```

**Options:**

- `--remove`: Automatically remove duplicates

### **Interactive Mode**

```bash
shorty interactive
shorty i           # Short alias
```

**Features:**

- Full-screen terminal interface
- Keyboard navigation (↑/↓, Enter, Space, Esc)
- Real-time search and filtering
- Bulk selection and operations
- Visual command preview

### **Configuration Management**

#### **List Configuration**

```bash
shorty config list
```

#### **Get Configuration Value**

```bash
shorty config get <key>
```

#### **Set Configuration Value**

```bash
shorty config set <key> <value>
```

#### **Reset Configuration**

```bash
shorty config reset
```

**Configuration Categories:**

- **Backup**: `auto_backup`, `max_backups`, `backup_before_edit`
- **Display**: `color_output`, `show_line_numbers`, `max_command_length`
- **Search**: `fuzzy_matching`, `case_sensitive`, `search_in_notes`
- **Aliases**: `file_path`, `sort_on_add`, `validate_on_add`

**Examples:**

```bash
shorty config set backup.auto_backup false
shorty config set display.color_output true
shorty config get search.fuzzy_matching
```

### **Statistics & Analytics**

```bash
shorty stats
```

**Provides:**

- Total alias count and categorization
- Command analysis and complexity metrics
- Tag usage statistics
- Most common commands and patterns
- File information and recommendations

### **Data Management**

#### **Export Aliases**

```bash
shorty export [OPTIONS]
```

**Options:**

- `--format <FORMAT>`: Export format (json, csv, bash)
- `--output, -o <FILE>`: Output file path

**Examples:**

```bash
shorty export --format json --output my-aliases.json
shorty export --format bash --output aliases-backup.sh
```

#### **Import Aliases**

```bash
shorty import <source> [OPTIONS]
```

**Options:**

- `--format <FORMAT>`: Source format (json, csv, bash)
- `--dry-run`: Preview import without applying changes

**Examples:**

```bash
shorty import ~/.bashrc --format bash --dry-run
shorty import aliases.json --format json
```

### **Template System**

#### **Add Template**

```bash
shorty template add <name> <pattern> [OPTIONS]
```

**Options:**

- `--description, -d <DESC>`: Template description
- `--category, -c <CATEGORY>`: Template category

**Examples:**

```bash
shorty template add docker-run "docker run -it {{image}} {{command}}" --description "Run Docker container"
shorty template add ssh-tunnel "ssh -L {{local}}:localhost:{{remote}} {{user}}@{{host}} -N"
```

#### **List Templates**

```bash
shorty template list [OPTIONS]
```

**Options:**

- `--category, -c <CATEGORY>`: Filter by category

#### **Use Template**

```bash
shorty template use <name> [OPTIONS]
```

**Options:**

- `--params <PARAMS>`: Template parameters (key=value,key2=value2)
- `--alias-name, -a <NAME>`: Custom alias name

**Examples:**

```bash
shorty template use docker-run --params "image=nginx,command=bash" --alias-name "nginx-shell"
```

#### **Show Template Details**

```bash
shorty template show <name>
```

#### **Remove Template**

```bash
shorty template remove <name>
```

### **Category Management**

#### **List Categories**

```bash
shorty category list [OPTIONS]
```

**Options:**

- `--tree`: Show as tree structure
- `--counts`: Show alias counts per category

#### **Add Category**

```bash
shorty category add <name> [OPTIONS]
```

**Options:**

- `--description, -d <DESC>`: Category description
- `--parent, -p <PARENT>`: Parent category
- `--color, -c <COLOR>`: Category color
- `--icon, -i <ICON>`: Category icon

#### **Move Alias to Category**

```bash
shorty category move <alias> <category>
```

#### **Group Aliases by Category**

```bash
shorty category group
```

#### **Remove Category**

```bash
shorty category remove <name> [OPTIONS]
```

**Options:**

- `--force`: Force removal even if category has children or aliases

#### **Generate Completion Scripts**

```bash
shorty completion --shell <SHELL>
```

**Supported shells:** bash, zsh, fish

**Examples:**

```bash
# Generate bash completion
shorty completion --shell bash
# Then install with:
sudo cp shorty_completion.bash /etc/bash_completion.d/shorty

# Generate zsh completion  
shorty completion --shell zsh
# Then install with:
mkdir -p ~/.zsh/completions
cp shorty_completion.zsh ~/.zsh/completions/_shorty
# Add to ~/.zshrc: fpath=(~/.zsh/completions $fpath)
# Then reload: autoload -U compinit && compinit

# Generate fish completion
shorty completion --shell fish  
# Then install with:
cp shorty_completion.fish ~/.config/fish/completions/shorty.fish
```

### **System Management**

#### **Uninstall Shorty**

```bash
shorty uninstall
```

Safely removes Shorty and all its configuration files.

### **Template Management (Additional Commands)**

#### **Update Template**

```bash
shorty template update <name> [OPTIONS]
```

**Options:**

- `--pattern <PATTERN>`: New pattern
- `--description <DESC>`: New description  
- `--category <CATEGORY>`: New category

## Configuration

### **Configuration File Location**

- **Config**: `~/.shorty/config.toml`
- **Aliases**: `~/.shorty/aliases`
- **Backups**: `~/.shorty/backups/`
- **Templates**: `~/.shorty/templates.toml`
- **Categories**: `~/.shorty/categories.toml`

### **Example Configuration**

```toml
[backup]
auto_backup = true
max_backups = 15
backup_before_edit = true

[display]
color_output = true
show_line_numbers = false
truncate_commands = true
max_command_length = 50

[search]
fuzzy_matching = false
case_sensitive = false
search_in_notes = true
search_in_tags = true

[aliases]
file_path = "~/.shorty/aliases"
sort_on_add = false
validate_on_add = true
```

## Performance & Compatibility

- **Fast**: Built with Rust for maximum performance
- **Cross-platform**: Linux, macOS support
- **Shell Compatibility**: bash, zsh, fish, PowerShell
- **Memory Efficient**: Minimal resource usage
- **Large Datasets**: Handles thousands of aliases efficiently

## Contributing

Contributions are welcome!

### **Feature Requests & Bug Reports**

Please use [GitHub Issues](https://github.com/anggasct/shorty/issues) to report bugs or request features.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
