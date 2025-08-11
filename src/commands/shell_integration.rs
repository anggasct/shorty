use crate::utils::get_aliases_path;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

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

pub fn install_shell_integration(shell: Shell, force: bool) -> anyhow::Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    match shell {
        Shell::Bash => install_bash_integration(&home_dir, force),
        Shell::Zsh => install_zsh_integration(&home_dir, force),
        Shell::Fish => install_fish_integration(&home_dir, force),
    }
}

pub fn generate_completion_script(shell: Shell) -> anyhow::Result<()> {
    let completion_script = match shell {
        Shell::Bash => generate_bash_completion(),
        Shell::Zsh => generate_zsh_completion(),
        Shell::Fish => generate_fish_completion(),
    };

    let shell_name = match shell {
        Shell::Bash => "bash",
        Shell::Zsh => "zsh",
        Shell::Fish => "fish",
    };

    let output_path = format!("shorty_completion.{}", shell_name);
    fs::write(&output_path, completion_script)?;

    println!(
        "Generated {} completion script: {}",
        shell_name, output_path
    );
    println!("Installation instructions:");

    match shell {
        Shell::Bash => {
            println!("  1. Copy to system completion directory:");
            println!("     sudo cp {} /etc/bash_completion.d/shorty", output_path);
            println!("  2. Or source in your ~/.bashrc:");
            println!("     echo 'source ~/{}'.bashrc", output_path);
        }
        Shell::Zsh => {
            println!("  1. Add to your fpath in ~/.zshrc:");
            println!("     fpath=(~/.zsh/completions $fpath)");
            println!("     mkdir -p ~/.zsh/completions");
            println!("     cp {} ~/.zsh/completions/_shorty", output_path);
            println!("  2. Restart zsh or run: autoload -U compinit && compinit");
        }
        Shell::Fish => {
            println!("  1. Copy to fish completions directory:");
            println!("     cp {} ~/.config/fish/completions/", output_path);
            println!("  2. Completions will be available immediately");
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn show_installation_status() -> anyhow::Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    println!("Shell Integration Status:\n");

    let bash_status = check_bash_integration(&home_dir);
    println!("Bash:");
    match bash_status {
        Ok(path) => println!("  Integrated in: {}", path.display()),
        Err(e) => println!("  Not integrated: {}", e),
    }

    let zsh_status = check_zsh_integration(&home_dir);
    println!("\nZsh:");
    match zsh_status {
        Ok(path) => println!("  Integrated in: {}", path.display()),
        Err(e) => println!("  Not integrated: {}", e),
    }

    let fish_status = check_fish_integration(&home_dir);
    println!("\nFish:");
    match fish_status {
        Ok(path) => println!("  Integrated in: {}", path.display()),
        Err(e) => println!("  Not integrated: {}", e),
    }

    println!("\nCompletion Scripts:");
    check_completion_status();

    if let Ok(shell) = env::var("SHELL") {
        println!("\nCurrent Shell: {}", shell);
        if shell.contains("zsh") {
            println!("Tip: Run 'shorty install --shell zsh' for better integration");
        } else if shell.contains("bash") {
            println!("Tip: Run 'shorty install --shell bash' for better integration");
        } else if shell.contains("fish") {
            println!("Tip: Run 'shorty install --shell fish' for better integration");
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn uninstall_shell_integration(shell: Option<Shell>) -> anyhow::Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    match shell {
        Some(shell) => {
            uninstall_specific_shell(&home_dir, shell)?;
        }
        None => {
            let _ = uninstall_specific_shell(&home_dir, Shell::Bash);
            let _ = uninstall_specific_shell(&home_dir, Shell::Zsh);
            let _ = uninstall_specific_shell(&home_dir, Shell::Fish);
            println!("Uninstalled shorty integration from all shells");
        }
    }

    Ok(())
}

fn install_bash_integration(home_dir: &Path, force: bool) -> anyhow::Result<()> {
    let bashrc_path = home_dir.join(".bashrc");
    let bash_profile_path = home_dir.join(".bash_profile");

    let target_file = if bashrc_path.exists() {
        bashrc_path
    } else if bash_profile_path.exists() {
        bash_profile_path
    } else {
        bashrc_path
    };

    let aliases_path = get_aliases_path();
    let integration_line = format!("source {}", aliases_path.display());
    let comment_line = "# Shorty aliases integration";

    if target_file.exists() {
        let content = fs::read_to_string(&target_file)?;
        if content.contains(&integration_line) && !force {
            anyhow::bail!(
                "Bash integration already exists in {}. Use --force to reinstall",
                target_file.display()
            );
        }
    }

    if !aliases_path.exists() {
        fs::write(&aliases_path, "# Shorty aliases file\n")?;
    }

    let mut content = if target_file.exists() {
        fs::read_to_string(&target_file)?
    } else {
        String::new()
    };

    if force {
        content = remove_integration_lines(&content, "bash");
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("\n{}\n{}\n", comment_line, integration_line));

    fs::write(&target_file, content)?;

    println!("Bash integration installed in: {}", target_file.display());
    println!(
        "Restart your terminal or run: source {}",
        target_file.display()
    );

    Ok(())
}

fn install_zsh_integration(home_dir: &Path, force: bool) -> anyhow::Result<()> {
    let zshrc_path = home_dir.join(".zshrc");

    let aliases_path = get_aliases_path();
    let integration_line = format!("source {}", aliases_path.display());
    let comment_line = "# Shorty aliases integration";

    if zshrc_path.exists() {
        let content = fs::read_to_string(&zshrc_path)?;
        if content.contains(&integration_line) && !force {
            anyhow::bail!(
                "Zsh integration already exists in {}. Use --force to reinstall",
                zshrc_path.display()
            );
        }
    }

    if !aliases_path.exists() {
        fs::write(&aliases_path, "# Shorty aliases file\n")?;
    }

    let mut content = if zshrc_path.exists() {
        fs::read_to_string(&zshrc_path)?
    } else {
        String::new()
    };

    if force {
        content = remove_integration_lines(&content, "zsh");
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("\n{}\n{}\n", comment_line, integration_line));

    fs::write(&zshrc_path, content)?;

    println!("Zsh integration installed in: {}", zshrc_path.display());
    println!(
        "Restart your terminal or run: source {}",
        zshrc_path.display()
    );

    Ok(())
}

fn install_fish_integration(home_dir: &Path, force: bool) -> anyhow::Result<()> {
    let fish_config_dir = home_dir.join(".config").join("fish");
    let fish_config_path = fish_config_dir.join("config.fish");

    fs::create_dir_all(&fish_config_dir)?;

    let aliases_path = get_aliases_path();
    let integration_line = format!("source {}", aliases_path.display());
    let comment_line = "# Shorty aliases integration";

    if fish_config_path.exists() {
        let content = fs::read_to_string(&fish_config_path)?;
        if content.contains(&integration_line) && !force {
            anyhow::bail!(
                "Fish integration already exists in {}. Use --force to reinstall",
                fish_config_path.display()
            );
        }
    }

    if !aliases_path.exists() {
        fs::write(&aliases_path, "# Shorty aliases file\n")?;
    }

    let mut content = if fish_config_path.exists() {
        fs::read_to_string(&fish_config_path)?
    } else {
        String::new()
    };

    if force {
        content = remove_integration_lines(&content, "fish");
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("\n{}\n{}\n", comment_line, integration_line));

    fs::write(&fish_config_path, content)?;

    println!(
        "Fish integration installed in: {}",
        fish_config_path.display()
    );
    println!(
        "Restart your terminal or run: source {}",
        fish_config_path.display()
    );

    Ok(())
}

#[allow(dead_code)]
fn check_bash_integration(home_dir: &Path) -> anyhow::Result<PathBuf> {
    let files = vec![home_dir.join(".bashrc"), home_dir.join(".bash_profile")];

    let aliases_path = get_aliases_path();
    let integration_line = format!("source {}", aliases_path.display());

    for file_path in files {
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            if content.contains(&integration_line) {
                return Ok(file_path);
            }
        }
    }

    anyhow::bail!("No integration found")
}

#[allow(dead_code)]
fn check_zsh_integration(home_dir: &Path) -> anyhow::Result<PathBuf> {
    let zshrc_path = home_dir.join(".zshrc");

    if !zshrc_path.exists() {
        anyhow::bail!("~/.zshrc not found");
    }

    let aliases_path = get_aliases_path();
    let integration_line = format!("source {}", aliases_path.display());
    let content = fs::read_to_string(&zshrc_path)?;

    if content.contains(&integration_line) {
        Ok(zshrc_path)
    } else {
        anyhow::bail!("No integration found in ~/.zshrc")
    }
}

#[allow(dead_code)]
fn check_fish_integration(home_dir: &Path) -> anyhow::Result<PathBuf> {
    let fish_config_path = home_dir.join(".config").join("fish").join("config.fish");

    if !fish_config_path.exists() {
        anyhow::bail!("Fish config not found");
    }

    let aliases_path = get_aliases_path();
    let integration_line = format!("source {}", aliases_path.display());
    let content = fs::read_to_string(&fish_config_path)?;

    if content.contains(&integration_line) {
        Ok(fish_config_path)
    } else {
        anyhow::bail!("No integration found in Fish config")
    }
}

#[allow(dead_code)]
fn check_completion_status() {
    let bash_completion_paths = vec![
        "/etc/bash_completion.d/shorty",
        "/usr/local/etc/bash_completion.d/shorty",
    ];

    let mut bash_found = false;
    for path in bash_completion_paths {
        if Path::new(path).exists() {
            println!("  Bash: {}", path);
            bash_found = true;
            break;
        }
    }
    if !bash_found {
        println!("  Bash: Not installed");
    }

    if let Some(home) = dirs::home_dir() {
        let zsh_completion_path = home.join(".zsh").join("completions").join("_shorty");
        if zsh_completion_path.exists() {
            println!("  Zsh: {}", zsh_completion_path.display());
        } else {
            println!("  Zsh: Not installed");
        }
    }

    if let Some(home) = dirs::home_dir() {
        let fish_completion_path = home
            .join(".config")
            .join("fish")
            .join("completions")
            .join("shorty.fish");
        if fish_completion_path.exists() {
            println!("  Fish: {}", fish_completion_path.display());
        } else {
            println!("  Fish: Not installed");
        }
    }
}

#[allow(dead_code)]
fn uninstall_specific_shell(home_dir: &Path, shell: Shell) -> anyhow::Result<()> {
    match shell {
        Shell::Bash => {
            let files = vec![home_dir.join(".bashrc"), home_dir.join(".bash_profile")];

            for file_path in files {
                if file_path.exists() {
                    let content = fs::read_to_string(&file_path)?;
                    let new_content = remove_integration_lines(&content, "bash");
                    if content != new_content {
                        fs::write(&file_path, new_content)?;
                        println!("Removed integration from: {}", file_path.display());
                    }
                }
            }
        }
        Shell::Zsh => {
            let zshrc_path = home_dir.join(".zshrc");
            if zshrc_path.exists() {
                let content = fs::read_to_string(&zshrc_path)?;
                let new_content = remove_integration_lines(&content, "zsh");
                if content != new_content {
                    fs::write(&zshrc_path, new_content)?;
                    println!("Removed integration from: {}", zshrc_path.display());
                }
            }
        }
        Shell::Fish => {
            let fish_config_path = home_dir.join(".config").join("fish").join("config.fish");
            if fish_config_path.exists() {
                let content = fs::read_to_string(&fish_config_path)?;
                let new_content = remove_integration_lines(&content, "fish");
                if content != new_content {
                    fs::write(&fish_config_path, new_content)?;
                    println!("Removed integration from: {}", fish_config_path.display());
                }
            }
        }
    }

    Ok(())
}

fn remove_integration_lines(content: &str, _shell: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip_next = false;

    for line in lines {
        if line.contains("# Shorty aliases integration") {
            skip_next = true;
            continue;
        }

        if skip_next && (line.starts_with("source") && line.contains("shorty_aliases")) {
            skip_next = false;
            continue;
        }

        skip_next = false;
        new_lines.push(line);
    }

    while new_lines.last() == Some(&"") {
        new_lines.pop();
    }

    new_lines.join("\n") + if !new_lines.is_empty() { "\n" } else { "" }
}

fn generate_bash_completion() -> String {
    r#"#!/bin/bash

_shorty_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    if [ ${COMP_CWORD} -eq 1 ]; then
        opts="add edit list remove search backup validate duplicates interactive config stats export import template category uninstall help"
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
    
    case "${COMP_WORDS[1]}" in
        backup)
            if [ ${COMP_CWORD} -eq 2 ]; then
                opts="create restore list clean"
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            fi
            ;;
        config)
            if [ ${COMP_CWORD} -eq 2 ]; then
                opts="set get list reset"
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            fi
            ;;
        template)
            if [ ${COMP_CWORD} -eq 2 ]; then
                opts="add list use remove show update"
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            fi
            ;;
        category)
            if [ ${COMP_CWORD} -eq 2 ]; then
                opts="add list remove move show group"
                COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            fi
            ;;
        export)
            case "${prev}" in
                --format)
                    opts="json csv bash"
                    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
                    ;;
                *)
                    opts="--format --output"
                    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
                    ;;
            esac
            ;;
        import)
            case "${prev}" in
                --format)
                    opts="json csv bash"
                    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
                    ;;
                *)
                    opts="--format --dry-run bash zsh fish"
                    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
                    ;;
            esac
            ;;
        *)
            COMPREPLY=( $(compgen -f -- ${cur}) )
            ;;
    esac
}

complete -F _shorty_completion shorty
"#.to_string()
}

fn generate_zsh_completion() -> String {
    r#"#compdef shorty

_shorty() {
    local context state state_descr line
    typeset -A opt_args
    
    _arguments -C \
        '1: :_shorty_commands' \
        '*::arg:->args'
        
    case $line[1] in
        add)
            _arguments \
                '1:alias name:' \
                '2:command:' \
                '--note[Add a note]:note:' \
                '--tags[Add tags]:tags:'
            ;;
        edit)
            _arguments \
                '1:alias name:_shorty_aliases' \
                '2:new command:' \
                '--note[Add a note]:note:' \
                '--tags[Add tags]:tags:'
            ;;
        remove|rm)
            _arguments '1:alias name:_shorty_aliases'
            ;;
        search)
            _arguments \
                '1:keyword:' \
                '--in[Search in field]:field:(command note tag)' \
                '--regex[Use regex]'
            ;;
        backup)
            case $line[2] in
                create)
                    _arguments '--name[Backup name]:name:'
                    ;;
                restore)
                    _arguments '1:backup file:_files'
                    ;;
                clean)
                    _arguments '--older-than[Days]:days:'
                    ;;
                *)
                    _values 'backup commands' \
                        'create[Create backup]' \
                        'restore[Restore backup]' \
                        'list[List backups]' \
                        'clean[Clean old backups]'
                    ;;
            esac
            ;;
        config)
            case $line[2] in
                set)
                    _arguments \
                        '1:key:_shorty_config_keys' \
                        '2:value:'
                    ;;
                get)
                    _arguments '1:key:_shorty_config_keys'
                    ;;
                *)
                    _values 'config commands' \
                        'set[Set config value]' \
                        'get[Get config value]' \
                        'list[List all config]' \
                        'reset[Reset to defaults]'
                    ;;
            esac
            ;;
        export)
            _arguments \
                '--format[Export format]:format:(json csv bash)' \
                '--output[Output file]:file:_files'
            ;;
        import)
            _arguments \
                '1:source:(bash zsh fish)' \
                '--format[Source format]:format:(json csv bash)' \
                '--dry-run[Preview only]'
            ;;
        template)
            _values 'template commands' \
                'add[Add template]' \
                'list[List templates]' \
                'use[Use template]' \
                'remove[Remove template]' \
                'show[Show template]' \
                'update[Update template]'
            ;;
        category)
            _values 'category commands' \
                'add[Add category]' \
                'list[List categories]' \
                'remove[Remove category]' \
                'move[Move alias to category]' \
                'show[Show category]' \
                'group[Group by category]'
            ;;
    esac
}

_shorty_commands() {
    local commands
    commands=(
        'add:Add a new alias'
        'edit:Edit an existing alias'
        'list:List all aliases'
        'remove:Remove an alias'
        'search:Search aliases'
        'backup:Backup and restore aliases'
        'validate:Validate aliases'
        'duplicates:Check duplicates'
        'interactive:Interactive mode'
        'config:Configuration'
        'stats:Statistics'
        'export:Export aliases'
        'import:Import aliases'
        'template:Template management'
        'category:Category management'
        'uninstall:Uninstall shorty'
    )
    _describe 'commands' commands
}

_shorty_aliases() {
    local aliases
    if [[ -f ~/.shorty_aliases ]]; then
        aliases=(${(f)"$(grep '^alias ' ~/.shorty_aliases | sed 's/alias \([^=]*\)=.*/\1/')"})
        _describe 'aliases' aliases
    fi
}

_shorty_config_keys() {
    local keys
    keys=(
        'backup.auto_backup:Auto backup'
        'backup.max_backups:Max backups'
        'display.color_output:Color output'
        'search.fuzzy_matching:Fuzzy matching'
        'aliases.file_path:Aliases file path'
    )
    _describe 'config keys' keys
}

_shorty
"#
    .to_string()
}

fn generate_fish_completion() -> String {
    r#"# Fish completion for shorty

complete -c shorty -f

complete -c shorty -n __fish_use_subcommand -a "add" -d "Add a new alias"
complete -c shorty -n __fish_use_subcommand -a "edit" -d "Edit an existing alias"
complete -c shorty -n __fish_use_subcommand -a "list" -d "List all aliases"
complete -c shorty -n __fish_use_subcommand -a "remove" -d "Remove an alias"
complete -c shorty -n __fish_use_subcommand -a "search" -d "Search aliases"
complete -c shorty -n __fish_use_subcommand -a "backup" -d "Backup and restore aliases"
complete -c shorty -n __fish_use_subcommand -a "validate" -d "Validate aliases"
complete -c shorty -n __fish_use_subcommand -a "duplicates" -d "Check for duplicates"
complete -c shorty -n __fish_use_subcommand -a "interactive" -d "Interactive mode"
complete -c shorty -n __fish_use_subcommand -a "config" -d "Configuration management"
complete -c shorty -n __fish_use_subcommand -a "stats" -d "Display statistics"
complete -c shorty -n __fish_use_subcommand -a "export" -d "Export aliases"
complete -c shorty -n __fish_use_subcommand -a "import" -d "Import aliases"
complete -c shorty -n __fish_use_subcommand -a "template" -d "Template management"
complete -c shorty -n __fish_use_subcommand -a "category" -d "Category management"
complete -c shorty -n __fish_use_subcommand -a "uninstall" -d "Uninstall shorty"

complete -c shorty -n "__fish_seen_subcommand_from add" -s n -l note -d "Add a note to the alias"
complete -c shorty -n "__fish_seen_subcommand_from add" -s t -l tags -d "Add tags to the alias"

complete -c shorty -n "__fish_seen_subcommand_from edit" -s n -l note -d "Add a new note"
complete -c shorty -n "__fish_seen_subcommand_from edit" -s t -l tags -d "Add new tags"

complete -c shorty -n "__fish_seen_subcommand_from list" -s t -l tag -d "Filter by tag"

complete -c shorty -n "__fish_seen_subcommand_from search" -l in -d "Search in specific field" -xa "command note tag"
complete -c shorty -n "__fish_seen_subcommand_from search" -l regex -d "Use regex pattern"

complete -c shorty -n "__fish_seen_subcommand_from backup" -n "not __fish_seen_subcommand_from create restore list clean" -a "create" -d "Create a backup"
complete -c shorty -n "__fish_seen_subcommand_from backup" -n "not __fish_seen_subcommand_from create restore list clean" -a "restore" -d "Restore from backup"
complete -c shorty -n "__fish_seen_subcommand_from backup" -n "not __fish_seen_subcommand_from create restore list clean" -a "list" -d "List available backups"
complete -c shorty -n "__fish_seen_subcommand_from backup" -n "not __fish_seen_subcommand_from create restore list clean" -a "clean" -d "Clean old backups"

complete -c shorty -n "__fish_seen_subcommand_from config" -n "not __fish_seen_subcommand_from set get list reset" -a "set" -d "Set configuration value"
complete -c shorty -n "__fish_seen_subcommand_from config" -n "not __fish_seen_subcommand_from set get list reset" -a "get" -d "Get configuration value"
complete -c shorty -n "__fish_seen_subcommand_from config" -n "not __fish_seen_subcommand_from set get list reset" -a "list" -d "List all configuration"
complete -c shorty -n "__fish_seen_subcommand_from config" -n "not __fish_seen_subcommand_from set get list reset" -a "reset" -d "Reset to defaults"

complete -c shorty -n "__fish_seen_subcommand_from export" -l format -d "Export format" -xa "json csv bash"
complete -c shorty -n "__fish_seen_subcommand_from export" -s o -l output -d "Output file path"

complete -c shorty -n "__fish_seen_subcommand_from import" -l format -d "Source format" -xa "json csv bash"
complete -c shorty -n "__fish_seen_subcommand_from import" -l dry-run -d "Preview import"

complete -c shorty -n "__fish_seen_subcommand_from template" -n "not __fish_seen_subcommand_from add list use remove show update" -a "add" -d "Add new template"
complete -c shorty -n "__fish_seen_subcommand_from template" -n "not __fish_seen_subcommand_from add list use remove show update" -a "list" -d "List available templates"
complete -c shorty -n "__fish_seen_subcommand_from template" -n "not __fish_seen_subcommand_from add list use remove show update" -a "use" -d "Use a template"
complete -c shorty -n "__fish_seen_subcommand_from template" -n "not __fish_seen_subcommand_from add list use remove show update" -a "remove" -d "Remove a template"
complete -c shorty -n "__fish_seen_subcommand_from template" -n "not __fish_seen_subcommand_from add list use remove show update" -a "show" -d "Show template details"
complete -c shorty -n "__fish_seen_subcommand_from template" -n "not __fish_seen_subcommand_from add list use remove show update" -a "update" -d "Update a template"

complete -c shorty -n "__fish_seen_subcommand_from category" -n "not __fish_seen_subcommand_from add list remove move show group" -a "add" -d "Add new category"
complete -c shorty -n "__fish_seen_subcommand_from category" -n "not __fish_seen_subcommand_from add list remove move show group" -a "list" -d "List categories"
complete -c shorty -n "__fish_seen_subcommand_from category" -n "not __fish_seen_subcommand_from add list remove move show group" -a "remove" -d "Remove category"
complete -c shorty -n "__fish_seen_subcommand_from category" -n "not __fish_seen_subcommand_from add list remove move show group" -a "move" -d "Move alias to category"
complete -c shorty -n "__fish_seen_subcommand_from category" -n "not __fish_seen_subcommand_from add list remove move show group" -a "show" -d "Show category details"
complete -c shorty -n "__fish_seen_subcommand_from category" -n "not __fish_seen_subcommand_from add list remove move show group" -a "group" -d "Group aliases by category"

function __fish_shorty_aliases
    if test -f ~/.shorty_aliases
        grep '^alias ' ~/.shorty_aliases | sed 's/alias \([^=]*\)=.*/\1/'
    end
end

complete -c shorty -n "__fish_seen_subcommand_from remove edit" -a "(__fish_shorty_aliases)"
"#.to_string()
}
