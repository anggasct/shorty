use crate::commands::backup::auto_backup;
use crate::utils::get_aliases_path;
use std::collections::{HashMap, HashSet};
use std::fs;
use which::which;

#[derive(Debug)]
struct AliasIssue {
    line_number: usize,
    alias_name: String,
    issue_type: IssueType,
    description: String,
    suggestion: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum IssueType {
    InvalidSyntax,
    CommandNotFound,
    Duplicate,
    SystemConflict,
    EmptyCommand,
    SuspiciousCommand,
}

pub fn validate_aliases(fix_issues: bool) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    if !aliases_path.exists() {
        println!("No aliases file found. Nothing to validate.");
        return Ok(());
    }

    println!("Validating aliases...\n");

    let content = fs::read_to_string(&aliases_path)?;
    let mut issues = Vec::new();
    let mut seen_aliases = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_number = line_num + 1;

        if line.trim().is_empty() || line.trim().starts_with('#') {
            continue;
        }

        if let Some(issue) = validate_line(line, line_number, &mut seen_aliases) {
            issues.push(issue);
        }
    }

    if issues.is_empty() {
        println!("All aliases are valid! No issues found.");
        return Ok(());
    }

    let mut issues_by_type: HashMap<IssueType, Vec<&AliasIssue>> = HashMap::new();
    for issue in &issues {
        issues_by_type
            .entry(issue.issue_type.clone())
            .or_default()
            .push(issue);
    }

    println!("Found {} issue(s):\n", issues.len());

    for (issue_type, type_issues) in issues_by_type {
        println!("{}:", format_issue_type(&issue_type));

        for issue in type_issues {
            println!(
                "  Line {}: {} - {}",
                issue.line_number, issue.alias_name, issue.description
            );
            if let Some(suggestion) = &issue.suggestion {
                println!("    Suggestion: {suggestion}");
            }
        }
        println!();
    }

    if fix_issues {
        println!("Attempting to fix issues...");
        auto_backup()?;
        let fixed_count = fix_aliases(&issues)?;
        if fixed_count > 0 {
            println!("Fixed {fixed_count} issue(s).");
            println!("To apply the changes, please restart your terminal!");
        } else {
            println!("No issues could be automatically fixed.");
        }
    } else {
        println!("Run with --fix to attempt automatic fixes where possible.");
    }

    Ok(())
}

pub fn check_duplicates(remove_duplicates: bool) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();

    if !aliases_path.exists() {
        println!("No aliases file found.");
        return Ok(());
    }

    let content = fs::read_to_string(&aliases_path)?;
    let mut seen_aliases: HashMap<String, Vec<usize>> = HashMap::new();
    let mut duplicates = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_number = line_num + 1;

        if let Some(alias_name) = extract_alias_name(line) {
            seen_aliases
                .entry(alias_name.clone())
                .or_default()
                .push(line_number);
        }
    }

    for (alias_name, line_numbers) in seen_aliases {
        if line_numbers.len() > 1 {
            duplicates.push((alias_name, line_numbers));
        }
    }

    if duplicates.is_empty() {
        println!("No duplicate aliases found.");
        return Ok(());
    }

    println!("Found {} duplicate alias(es):", duplicates.len());

    for (alias_name, line_numbers) in &duplicates {
        println!(
            "  '{}' appears on lines: {}",
            alias_name,
            line_numbers
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if remove_duplicates {
        println!("\nRemoving duplicates (keeping the last occurrence)...");
        auto_backup()?;

        let lines: Vec<String> = content.lines().map(String::from).collect();
        let mut new_lines = Vec::new();
        let mut seen_in_final = HashSet::new();

        for (i, line) in lines.iter().enumerate().rev() {
            if let Some(alias_name) = extract_alias_name(line) {
                if seen_in_final.contains(&alias_name) {
                    continue;
                }
                seen_in_final.insert(alias_name);
            }
            new_lines.insert(0, (i, line.clone()));
        }

        let final_content = new_lines
            .iter()
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if !final_content.is_empty() && !final_content.ends_with('\n') {
            fs::write(&aliases_path, format!("{final_content}\n"))?;
        } else {
            fs::write(&aliases_path, final_content)?;
        }

        let removed_count = lines.len() - new_lines.len();
        println!("Removed {removed_count} duplicate(s).");
        println!("To apply the changes, please restart your terminal!");
    } else {
        println!("\nRun with --remove to automatically remove duplicates.");
    }

    Ok(())
}

fn validate_line(
    line: &str,
    line_number: usize,
    seen_aliases: &mut HashMap<String, usize>,
) -> Option<AliasIssue> {
    let line = line.trim();

    if !line.starts_with("alias ") {
        return Some(AliasIssue {
            line_number,
            alias_name: "unknown".to_string(),
            issue_type: IssueType::InvalidSyntax,
            description: "Line doesn't start with 'alias'".to_string(),
            suggestion: Some("Ensure line starts with 'alias name=command'".to_string()),
        });
    }

    if let Some(eq_pos) = line.find('=') {
        let alias_part = &line[6..eq_pos].trim();
        let command_part = &line[eq_pos + 1..];

        if alias_part.is_empty() {
            return Some(AliasIssue {
                line_number,
                alias_name: "empty".to_string(),
                issue_type: IssueType::InvalidSyntax,
                description: "Empty alias name".to_string(),
                suggestion: Some("Provide a valid alias name".to_string()),
            });
        }

        if let Some(&previous_line) = seen_aliases.get(&alias_part.to_string()) {
            return Some(AliasIssue {
                line_number,
                alias_name: alias_part.to_string(),
                issue_type: IssueType::Duplicate,
                description: format!("Duplicate of alias on line {previous_line}"),
                suggestion: Some("Remove one of the duplicate aliases".to_string()),
            });
        }
        seen_aliases.insert(alias_part.to_string(), line_number);

        let command = extract_command(command_part);

        if command.is_empty() {
            return Some(AliasIssue {
                line_number,
                alias_name: alias_part.to_string(),
                issue_type: IssueType::EmptyCommand,
                description: "Empty command".to_string(),
                suggestion: Some("Provide a valid command".to_string()),
            });
        }

        let first_word = command.split_whitespace().next().unwrap_or("");
        if !first_word.is_empty() && !command_exists(first_word) {
            if is_system_command(alias_part) {
                return Some(AliasIssue {
                    line_number,
                    alias_name: alias_part.to_string(),
                    issue_type: IssueType::SystemConflict,
                    description: format!("Conflicts with system command '{alias_part}'"),
                    suggestion: Some("Consider using a different alias name".to_string()),
                });
            }

            return Some(AliasIssue {
                line_number,
                alias_name: alias_part.to_string(),
                issue_type: IssueType::CommandNotFound,
                description: format!("Command '{first_word}' not found in PATH"),
                suggestion: Some("Check if command is installed or fix typo".to_string()),
            });
        }

        if is_suspicious_command(&command) {
            return Some(AliasIssue {
                line_number,
                alias_name: alias_part.to_string(),
                issue_type: IssueType::SuspiciousCommand,
                description: "Potentially dangerous command detected".to_string(),
                suggestion: Some("Review this alias carefully".to_string()),
            });
        }
    } else {
        return Some(AliasIssue {
            line_number,
            alias_name: "unknown".to_string(),
            issue_type: IssueType::InvalidSyntax,
            description: "Missing '=' in alias definition".to_string(),
            suggestion: Some("Use format: alias name=command".to_string()),
        });
    }

    None
}

fn extract_alias_name(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("alias ") {
        return None;
    }

    if let Some(eq_pos) = line.find('=') {
        let alias_name = line[6..eq_pos].trim();
        if !alias_name.is_empty() {
            return Some(alias_name.to_string());
        }
    }

    None
}

fn extract_command(command_part: &str) -> String {
    let command_part = command_part.trim();

    let mut command_end = command_part.len();
    let mut in_quotes = false;
    let mut quote_char = ' ';
    let mut i = 0;

    while i < command_part.len() {
        let ch = command_part.chars().nth(i).unwrap();
        match ch {
            '\'' | '"' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            '#' if !in_quotes => {
                command_end = i;
                break;
            }
            _ => {}
        }
        i += 1;
    }

    let mut command = command_part[..command_end].trim();

    if (command.starts_with('\'') && command.ends_with('\''))
        || (command.starts_with('"') && command.ends_with('"'))
    {
        command = &command[1..command.len() - 1];
    }

    command.to_string()
}

fn command_exists(command: &str) -> bool {
    let builtins = [
        "cd", "echo", "pwd", "exit", "source", ".", "alias", "unalias", "export", "set", "unset",
        "history", "jobs", "bg", "fg", "kill",
    ];

    if builtins.contains(&command) {
        return true;
    }

    which(command).is_ok()
}

fn is_system_command(alias_name: &str) -> bool {
    let system_commands = [
        "ls", "cd", "cp", "mv", "rm", "mkdir", "rmdir", "cat", "grep", "find", "ps", "kill", "top",
        "chmod", "chown",
    ];
    system_commands.contains(&alias_name)
}

fn is_suspicious_command(command: &str) -> bool {
    let suspicious_patterns = [
        "rm -rf /",
        "sudo rm -rf",
        ":(){ :|:& };:",
        "mkfs",
        "dd if=",
        "> /dev/",
        "shutdown",
        "reboot",
    ];

    suspicious_patterns
        .iter()
        .any(|pattern| command.contains(pattern))
}

fn format_issue_type(issue_type: &IssueType) -> &str {
    match issue_type {
        IssueType::InvalidSyntax => "Invalid Syntax",
        IssueType::CommandNotFound => "Command Not Found",
        IssueType::Duplicate => "Duplicate Aliases",
        IssueType::SystemConflict => "System Command Conflicts",
        IssueType::EmptyCommand => "Empty Commands",
        IssueType::SuspiciousCommand => "Suspicious Commands",
    }
}

fn fix_aliases(issues: &[AliasIssue]) -> anyhow::Result<usize> {
    let _fixable_count = issues
        .iter()
        .filter(|issue| matches!(issue.issue_type, IssueType::Duplicate))
        .count();

    Ok(0)
}
