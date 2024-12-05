use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;

pub fn uninstall() -> anyhow::Result<()> {
    let binary_path = PathBuf::from("/usr/local/bin/shorty");
    if binary_path.exists() {
        let status = Command::new("sudo")
            .arg("rm")
            .arg(&binary_path)
            .status()?;
        if status.success() {
            println!("Removed shorty binary from /usr/local/bin.");
        } else {
            println!("Failed to remove shorty binary from /usr/local/bin.");
        }
    } else {
        println!("shorty binary not found in /usr/local/bin.");
    }

    let aliases_path = dirs::home_dir().expect("Could not find home directory").join(".shorty_aliases");
    if aliases_path.exists() {
        print!("Do you want to remove the ~/.shorty_aliases file? (y/n): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("y") {
            fs::remove_file(&aliases_path)?;
            println!("Removed ~/.shorty_aliases file.");
        } else {
            println!("Kept ~/.shorty_aliases file.");
        }
    } else {
        println!("~/.shorty_aliases file not found.");
    }

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let shell_name = shell.split('/').last().unwrap_or("sh");
    let config_file = match shell_name {
        "zsh" => dirs::home_dir().expect("Could not find home directory").join(".zshrc"),
        "bash" => dirs::home_dir().expect("Could not find home directory").join(".bashrc"),
        _ => {
            println!("Unsupported shell: {}. Please manually remove 'source ~/.shorty_aliases' from your shell configuration.", shell_name);
            return Ok(());
        }
    };

    if config_file.exists() {
        let file = fs::File::open(&config_file)?;
        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
        let new_lines: Vec<String> = lines.into_iter().filter(|line| !line.contains("source ~/.shorty_aliases")).collect();

        let mut file = fs::File::create(&config_file)?;
        for line in new_lines {
            writeln!(file, "{}", line)?;
        }
        println!("Removed 'source ~/.shorty_aliases' from {}.", config_file.display());
    } else {
        println!("Shell configuration file not found: {}.", config_file.display());
    }

    println!("Uninstallation complete!");
    Ok(())
}