use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
    pub executable: String,
    pub commands: Vec<PluginCommand>,
    pub hooks: Vec<String>,
    pub config: HashMap<String, serde_json::Value>,
    pub installed_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginRegistry {
    version: String,
    plugins: Vec<Plugin>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub executable: String,
    pub commands: Vec<PluginCommand>,
    pub hooks: Vec<String>,
    pub dependencies: Vec<String>,
    pub config_schema: HashMap<String, serde_json::Value>,
}

pub fn list_plugins(show_all: bool) -> anyhow::Result<()> {
    let plugins = load_plugins()?;

    if plugins.is_empty() {
        println!("No plugins installed");
        println!("Install plugins with 'shorty plugin install <name>'");
        return Ok(());
    }

    let filtered_plugins: Vec<_> = if show_all {
        plugins.iter().collect()
    } else {
        plugins.iter().filter(|p| p.enabled).collect()
    };

    if filtered_plugins.is_empty() {
        println!(
            "No {} plugins found",
            if show_all { "installed" } else { "enabled" }
        );
        return Ok(());
    }

    println!(
        "{} Plugins:\n",
        if show_all { "Installed" } else { "Enabled" }
    );

    for plugin in &filtered_plugins {
        let status_icon = if plugin.enabled {
            "[ENABLED]"
        } else {
            "[DISABLED]"
        };
        println!("{} {} v{}", status_icon, plugin.name, plugin.version);
        println!("   {}", plugin.description);
        println!("   Author: {}", plugin.author);

        if !plugin.commands.is_empty() {
            println!(
                "   Commands: {}",
                plugin
                    .commands
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if !plugin.hooks.is_empty() {
            println!("   Hooks: {}", plugin.hooks.join(", "));
        }

        println!("   Installed: {}", plugin.installed_at);
        println!();
    }

    println!("Use 'shorty plugin show <name>' for detailed information");

    Ok(())
}

pub fn install_plugin(name_or_path: &str) -> anyhow::Result<()> {
    println!("Installing plugin: {name_or_path}");

    let plugin_dir = get_plugins_dir()?;
    fs::create_dir_all(&plugin_dir)?;

    let plugin = if Path::new(name_or_path).exists() {
        install_from_path(name_or_path)?
    } else if name_or_path.starts_with("http") {
        install_from_url(name_or_path)?
    } else {
        install_from_registry(name_or_path)?
    };

    let mut plugins = load_plugins()?;

    if let Some(existing) = plugins.iter().position(|p| p.name == plugin.name) {
        plugins[existing] = plugin.clone();
        println!("Updated existing plugin: {}", plugin.name);
    } else {
        plugins.push(plugin.clone());
        println!("Installed new plugin: {}", plugin.name);
    }

    save_plugins(&plugins)?;

    if let Err(e) = validate_plugin(&plugin) {
        println!("Plugin validation warning: {e}");
    }

    println!("Plugin '{}' installed successfully", plugin.name);
    println!("Enable it with 'shorty plugin enable {}'", plugin.name);

    Ok(())
}

pub fn remove_plugin(name: &str) -> anyhow::Result<()> {
    let mut plugins = load_plugins()?;

    let plugin_index = plugins
        .iter()
        .position(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

    let plugin = &plugins[plugin_index];

    let plugin_path = get_plugin_path(&plugin.name)?;
    if plugin_path.exists() {
        fs::remove_dir_all(&plugin_path)?;
    }

    plugins.remove(plugin_index);
    save_plugins(&plugins)?;

    println!("Plugin '{name}' removed successfully");

    Ok(())
}

pub fn enable_plugin(name: &str) -> anyhow::Result<()> {
    let mut plugins = load_plugins()?;

    let plugin = plugins
        .iter_mut()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

    if plugin.enabled {
        println!("Plugin '{name}' is already enabled");
        return Ok(());
    }

    validate_plugin(plugin)?;

    plugin.enabled = true;
    save_plugins(&plugins)?;

    println!("Plugin '{name}' enabled");

    Ok(())
}

pub fn disable_plugin(name: &str) -> anyhow::Result<()> {
    let mut plugins = load_plugins()?;

    let plugin = plugins
        .iter_mut()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

    if !plugin.enabled {
        println!("Plugin '{name}' is already disabled");
        return Ok(());
    }

    plugin.enabled = false;
    save_plugins(&plugins)?;

    println!("Plugin '{name}' disabled");

    Ok(())
}

pub fn show_plugin(name: &str) -> anyhow::Result<()> {
    let plugins = load_plugins()?;

    let plugin = plugins
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", name))?;

    println!("Plugin: {}", plugin.name);
    println!("Version: {}", plugin.version);
    println!("Description: {}", plugin.description);
    println!("Author: {}", plugin.author);
    println!(
        "Status: {}",
        if plugin.enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!("Executable: {}", plugin.executable);
    println!("Installed: {}", plugin.installed_at);

    if !plugin.commands.is_empty() {
        println!("\nCommands:");
        for cmd in &plugin.commands {
            println!("  • {} - {}", cmd.name, cmd.description);
            println!("    Usage: {}", cmd.usage);
        }
    }

    if !plugin.hooks.is_empty() {
        println!("\nHooks:");
        for hook in &plugin.hooks {
            println!("  • {hook}");
        }
    }

    if !plugin.config.is_empty() {
        println!("\nConfiguration:");
        for (key, value) in &plugin.config {
            println!("  • {key}: {value}");
        }
    }

    let plugin_path = get_plugin_path(&plugin.name)?;
    println!("\nLocation: {}", plugin_path.display());

    Ok(())
}

pub fn execute_plugin_command(
    plugin_name: &str,
    command: &str,
    args: &[String],
) -> anyhow::Result<()> {
    let plugins = load_plugins()?;

    let plugin = plugins
        .iter()
        .find(|p| p.name == plugin_name && p.enabled)
        .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found or not enabled", plugin_name))?;

    let _plugin_command = plugin
        .commands
        .iter()
        .find(|c| c.name == command)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Command '{}' not found in plugin '{}'",
                command,
                plugin_name
            )
        })?;

    println!("Executing plugin command: {plugin_name} {command}");

    let plugin_path = get_plugin_path(&plugin.name)?;
    let executable_path = plugin_path.join(&plugin.executable);

    if !executable_path.exists() {
        anyhow::bail!("Plugin executable not found: {}", executable_path.display());
    }

    let mut cmd = Command::new(&executable_path);
    cmd.arg(command);
    cmd.args(args);

    cmd.env("SHORTY_PLUGIN_NAME", &plugin.name);
    cmd.env("SHORTY_PLUGIN_VERSION", &plugin.version);
    cmd.env(
        "SHORTY_ALIASES_PATH",
        get_aliases_path().display().to_string(),
    );

    let output = cmd.output()?;

    if !output.status.success() {
        println!("Plugin command failed:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("{stdout}");
    }

    println!("Plugin command completed successfully");

    Ok(())
}

#[allow(dead_code)]
pub fn run_plugin_hooks(hook_name: &str, context: &HashMap<String, String>) -> anyhow::Result<()> {
    let plugins = load_plugins()?;

    let hook_plugins: Vec<_> = plugins
        .iter()
        .filter(|p| p.enabled && p.hooks.contains(&hook_name.to_string()))
        .collect();

    if hook_plugins.is_empty() {
        return Ok(());
    }

    for plugin in hook_plugins {
        if let Err(e) = execute_plugin_hook(plugin, hook_name, context) {
            eprintln!("Hook execution failed for plugin '{}': {}", plugin.name, e);
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn execute_plugin_hook(
    plugin: &Plugin,
    hook_name: &str,
    context: &HashMap<String, String>,
) -> anyhow::Result<()> {
    let plugin_path = get_plugin_path(&plugin.name)?;
    let executable_path = plugin_path.join(&plugin.executable);

    if !executable_path.exists() {
        anyhow::bail!("Plugin executable not found: {}", executable_path.display());
    }

    let mut cmd = Command::new(&executable_path);
    cmd.arg("--hook");
    cmd.arg(hook_name);

    cmd.env("SHORTY_PLUGIN_NAME", &plugin.name);
    cmd.env("SHORTY_PLUGIN_VERSION", &plugin.version);
    cmd.env(
        "SHORTY_ALIASES_PATH",
        get_aliases_path().display().to_string(),
    );

    for (key, value) in context {
        cmd.env(format!("SHORTY_HOOK_{}", key.to_uppercase()), value);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Hook execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

fn install_from_path(path: &str) -> anyhow::Result<Plugin> {
    let source_path = Path::new(path);

    if !source_path.exists() {
        anyhow::bail!("Plugin path does not exist: {}", path);
    }

    let manifest_path = source_path.join("plugin.toml");
    if !manifest_path.exists() {
        anyhow::bail!("Plugin manifest not found: plugin.toml");
    }

    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: PluginManifest = toml::from_str(&manifest_content)?;

    validate_manifest(&manifest)?;

    let plugin_dir = get_plugin_path(&manifest.name)?;
    if plugin_dir.exists() {
        fs::remove_dir_all(&plugin_dir)?;
    }

    copy_directory(source_path, &plugin_dir)?;

    let plugin = Plugin {
        name: manifest.name,
        version: manifest.version,
        description: manifest.description,
        author: manifest.author,
        enabled: false,
        executable: manifest.executable,
        commands: manifest.commands,
        hooks: manifest.hooks,
        config: HashMap::new(),
        installed_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    Ok(plugin)
}

fn install_from_url(_url: &str) -> anyhow::Result<Plugin> {
    anyhow::bail!("URL-based plugin installation not yet implemented");
}

fn install_from_registry(_name: &str) -> anyhow::Result<Plugin> {
    anyhow::bail!(
        "Registry-based plugin installation not yet implemented. Use local path for now."
    );
}

fn validate_plugin(plugin: &Plugin) -> anyhow::Result<()> {
    let plugin_path = get_plugin_path(&plugin.name)?;
    let executable_path = plugin_path.join(&plugin.executable);

    if !executable_path.exists() {
        anyhow::bail!("Plugin executable not found: {}", executable_path.display());
    }

    if !is_executable(&executable_path)? {
        anyhow::bail!(
            "Plugin file is not executable: {}",
            executable_path.display()
        );
    }

    let output = Command::new(&executable_path).arg("--help").output();

    match output {
        Ok(out) if out.status.success() => {}
        _ => {
            anyhow::bail!("Plugin does not respond to --help command");
        }
    }

    Ok(())
}

fn validate_manifest(manifest: &PluginManifest) -> anyhow::Result<()> {
    if manifest.name.is_empty() {
        anyhow::bail!("Plugin name cannot be empty");
    }

    if manifest.version.is_empty() {
        anyhow::bail!("Plugin version cannot be empty");
    }

    if manifest.executable.is_empty() {
        anyhow::bail!("Plugin executable cannot be empty");
    }

    for cmd in &manifest.commands {
        if cmd.name.is_empty() {
            anyhow::bail!("Command name cannot be empty");
        }

        if cmd.name.contains(' ') {
            anyhow::bail!("Command name cannot contain spaces: {}", cmd.name);
        }
    }

    Ok(())
}

fn load_plugins() -> anyhow::Result<Vec<Plugin>> {
    let registry_path = get_plugins_registry_path()?;

    if !registry_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&registry_path)?;
    let registry: PluginRegistry = toml::from_str(&content)?;

    Ok(registry.plugins)
}

fn save_plugins(plugins: &[Plugin]) -> anyhow::Result<()> {
    let registry_path = get_plugins_registry_path()?;

    if let Some(parent) = registry_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let registry = PluginRegistry {
        version: "1.0".to_string(),
        plugins: plugins.to_vec(),
    };

    let content = toml::to_string_pretty(&registry)?;
    fs::write(&registry_path, content)?;

    Ok(())
}

fn get_plugins_dir() -> anyhow::Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    Ok(home_dir.join(".shorty").join("plugins"))
}

fn get_plugin_path(name: &str) -> anyhow::Result<PathBuf> {
    let plugins_dir = get_plugins_dir()?;
    Ok(plugins_dir.join(name))
}

fn get_plugins_registry_path() -> anyhow::Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    Ok(home_dir.join(".shorty").join("plugins.toml"))
}

fn get_aliases_path() -> PathBuf {
    crate::utils::get_aliases_path()
}

fn copy_directory(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn is_executable(path: &Path) -> anyhow::Result<bool> {
    let metadata = fs::metadata(path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        Ok(permissions.mode() & 0o111 != 0)
    }

    #[cfg(not(unix))]
    {
        Ok(path.extension().is_some_and(|ext| ext == "exe"))
    }
}
