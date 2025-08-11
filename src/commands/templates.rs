use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub parameters: Vec<TemplateParameter>,
    pub category: String,
    pub created_at: String,
    pub usage_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
    pub validation_pattern: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TemplatesData {
    version: String,
    templates: Vec<Template>,
}

pub fn add_template(
    name: &str,
    pattern: &str,
    description: Option<&str>,
    category: Option<&str>,
) -> anyhow::Result<()> {
    let mut templates = load_templates()?;

    if templates.iter().any(|t| t.name == name) {
        anyhow::bail!("Template '{}' already exists. Use a different name or remove the existing template first.", name);
    }
    let parameters = extract_parameters_from_pattern(pattern);

    let template = Template {
        name: name.to_string(),
        description: description.unwrap_or("No description").to_string(),
        pattern: pattern.to_string(),
        parameters,
        category: category.unwrap_or("general").to_string(),
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        usage_count: 0,
    };

    let template_params = template.parameters.clone();
    templates.push(template);
    save_templates(&templates)?;

    println!("Template '{}' added successfully", name);
    println!("Pattern: {}", pattern);
    if !template_params.is_empty() {
        println!("Parameters found:");
        for param in &template_params {
            println!("  • {} - {}", param.name, param.description);
        }
    }

    Ok(())
}

pub fn list_templates(category: Option<&str>) -> anyhow::Result<()> {
    let templates = load_templates()?;

    if templates.is_empty() {
        println!("No templates found. Create your first template with 'shorty template add'");
        return Ok(());
    }

    let filtered_templates: Vec<_> = if let Some(cat) = category {
        templates.iter().filter(|t| t.category == cat).collect()
    } else {
        templates.iter().collect()
    };

    if filtered_templates.is_empty() {
        println!(
            "No templates found in category '{}'",
            category.unwrap_or("all")
        );
        return Ok(());
    }

    println!("Available Templates:");

    let mut categories: HashMap<String, Vec<&Template>> = HashMap::new();
    for template in &filtered_templates {
        categories
            .entry(template.category.clone())
            .or_default()
            .push(template);
    }

    for (cat, templates_in_cat) in categories {
        println!("\n{}", cat.to_uppercase());

        for template in templates_in_cat {
            println!("  {} (used {}x)", template.name, template.usage_count);
            println!("     {}", template.description);
            println!("     Pattern: {}", template.pattern);

            if !template.parameters.is_empty() {
                let param_names: Vec<String> =
                    template.parameters.iter().map(|p| p.name.clone()).collect();
                println!("     Parameters: {}", param_names.join(", "));
            }
            println!();
        }
    }

    println!("Use 'shorty template use <name>' to create an alias from a template");
    Ok(())
}

pub fn use_template(
    name: &str,
    params: &HashMap<String, String>,
    alias_name: Option<&str>,
) -> anyhow::Result<()> {
    let mut templates = load_templates()?;

    let template = templates
        .iter_mut()
        .find(|t| t.name == name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))?;

    for param in &template.parameters {
        if param.required && !params.contains_key(&param.name) {
            anyhow::bail!(
                "Required parameter '{}' is missing. Description: {}",
                param.name,
                param.description
            );
        }
    }

    let mut command = template.pattern.clone();
    for param in &template.parameters {
        let placeholder = format!("{{{}}}", param.name);

        let value = if let Some(provided_value) = params.get(&param.name) {
            provided_value.clone()
        } else if let Some(default) = &param.default_value {
            default.clone()
        } else {
            continue;
        };

        if let Some(pattern) = &param.validation_pattern {
            let regex = regex::Regex::new(pattern)?;
            if !regex.is_match(&value) {
                anyhow::bail!(
                    "Parameter '{}' value '{}' doesn't match pattern '{}'",
                    param.name,
                    value,
                    pattern
                );
            }
        }

        command = command.replace(&placeholder, &value);
    }

    let remaining_params = extract_parameters_from_pattern(&command);
    if !remaining_params.is_empty() {
        let param_names: Vec<String> = remaining_params.iter().map(|p| p.name.clone()).collect();
        anyhow::bail!("Missing values for parameters: {}", param_names.join(", "));
    }

    let final_alias_name = if let Some(name) = alias_name {
        name.to_string()
    } else {
        let mut auto_name = template.name.clone();
        if let Some(first_param) = template.parameters.first() {
            if let Some(value) = params.get(&first_param.name) {
                auto_name = format!("{}_{}", auto_name, sanitize_alias_name(value));
            }
        }
        auto_name
    };

    crate::commands::add::add_alias(
        &final_alias_name,
        &command,
        &Some(format!("Generated from template: {}", template.name)),
        &vec![template.category.clone(), "template".to_string()],
    )?;

    let template_name = template.name.clone();
    template.usage_count += 1;
    save_templates(&templates)?;

    println!(
        "Alias '{}' created from template '{}'",
        final_alias_name, template_name
    );
    println!("Command: {}", command);

    Ok(())
}

pub fn remove_template(name: &str) -> anyhow::Result<()> {
    let mut templates = load_templates()?;

    let initial_count = templates.len();
    templates.retain(|t| t.name != name);

    if templates.len() == initial_count {
        anyhow::bail!("Template '{}' not found", name);
    }

    save_templates(&templates)?;
    println!("Template '{}' removed successfully", name);

    Ok(())
}

pub fn show_template(name: &str) -> anyhow::Result<()> {
    let templates = load_templates()?;

    let template = templates
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))?;

    println!("Template: {}", template.name);
    println!("Description: {}", template.description);
    println!("Category: {}", template.category);
    println!("Usage count: {}", template.usage_count);
    println!("Created: {}", template.created_at);
    println!("\nPattern:");
    println!("  {}", template.pattern);

    if !template.parameters.is_empty() {
        println!("\nParameters:");
        for param in &template.parameters {
            println!(
                "  • {} {}",
                param.name,
                if param.required {
                    "(required)"
                } else {
                    "(optional)"
                }
            );
            println!("    {}", param.description);

            if let Some(default) = &param.default_value {
                println!("    Default: {}", default);
            }

            if let Some(pattern) = &param.validation_pattern {
                println!("    Pattern: {}", pattern);
            }
        }

        println!("\nUsage example:");
        let example_params: Vec<String> = template
            .parameters
            .iter()
            .map(|p: &TemplateParameter| {
                format!(
                    "{}={}",
                    p.name,
                    p.default_value.as_deref().unwrap_or("value")
                )
            })
            .collect();
        println!(
            "  shorty template use {} --params {}",
            template.name,
            example_params.join(",")
        );
    }

    Ok(())
}

pub fn update_template(
    name: &str,
    new_pattern: Option<&str>,
    new_description: Option<&str>,
    new_category: Option<&str>,
) -> anyhow::Result<()> {
    let mut templates = load_templates()?;

    let template = templates
        .iter_mut()
        .find(|t| t.name == name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))?;

    let mut changes = Vec::new();

    if let Some(pattern) = new_pattern {
        template.pattern = pattern.to_string();
        template.parameters = extract_parameters_from_pattern(pattern);
        changes.push("pattern");
    }

    if let Some(description) = new_description {
        template.description = description.to_string();
        changes.push("description");
    }

    if let Some(category) = new_category {
        template.category = category.to_string();
        changes.push("category");
    }

    if changes.is_empty() {
        println!("No changes specified. Use --pattern, --description, or --category");
        return Ok(());
    }

    save_templates(&templates)?;

    println!("Template '{}' updated ({})", name, changes.join(", "));

    Ok(())
}

fn load_templates() -> anyhow::Result<Vec<Template>> {
    let templates_path = get_templates_path()?;

    if !templates_path.exists() {
        let default_templates = create_default_templates();
        save_templates(&default_templates)?;
        return Ok(default_templates);
    }

    let content = fs::read_to_string(&templates_path)?;
    let data: TemplatesData = toml::from_str(&content)?;

    Ok(data.templates)
}

fn save_templates(templates: &[Template]) -> anyhow::Result<()> {
    let templates_path = get_templates_path()?;

    if let Some(parent) = templates_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = TemplatesData {
        version: "1.0".to_string(),
        templates: templates.to_vec(),
    };

    let content = toml::to_string_pretty(&data)?;
    fs::write(&templates_path, content)?;

    Ok(())
}

fn get_templates_path() -> anyhow::Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    Ok(home_dir.join(".shorty").join("templates.toml"))
}

fn extract_parameters_from_pattern(pattern: &str) -> Vec<TemplateParameter> {
    let mut parameters = Vec::new();
    let regex = regex::Regex::new(r"\{(\w+)\}").unwrap();

    for cap in regex.captures_iter(pattern) {
        let param_name = &cap[1];

        if !parameters
            .iter()
            .any(|p: &TemplateParameter| p.name == param_name)
        {
            parameters.push(TemplateParameter {
                name: param_name.to_string(),
                description: format!("Parameter for {}", param_name),
                default_value: None,
                required: true,
                validation_pattern: None,
            });
        }
    }

    parameters
}

fn sanitize_alias_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

fn create_default_templates() -> Vec<Template> {
    vec![
        Template {
            name: "git_clone".to_string(),
            description: "Clone a Git repository".to_string(),
            pattern: "git clone {url} {directory}".to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "url".to_string(),
                    description: "Git repository URL".to_string(),
                    default_value: None,
                    required: true,
                    validation_pattern: Some(r"^https?://.*\.git$|^git@.*\.git$".to_string()),
                },
                TemplateParameter {
                    name: "directory".to_string(),
                    description: "Local directory name".to_string(),
                    default_value: Some(".".to_string()),
                    required: false,
                    validation_pattern: None,
                },
            ],
            category: "git".to_string(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            usage_count: 0,
        },
        Template {
            name: "docker_run".to_string(),
            description: "Run a Docker container".to_string(),
            pattern: "docker run -it --rm {options} {image} {command}".to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "options".to_string(),
                    description: "Docker run options (e.g., -p 8080:80)".to_string(),
                    default_value: Some("".to_string()),
                    required: false,
                    validation_pattern: None,
                },
                TemplateParameter {
                    name: "image".to_string(),
                    description: "Docker image name".to_string(),
                    default_value: None,
                    required: true,
                    validation_pattern: None,
                },
                TemplateParameter {
                    name: "command".to_string(),
                    description: "Command to run in container".to_string(),
                    default_value: Some("/bin/bash".to_string()),
                    required: false,
                    validation_pattern: None,
                },
            ],
            category: "docker".to_string(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            usage_count: 0,
        },
        Template {
            name: "npm_script".to_string(),
            description: "Run npm script with environment".to_string(),
            pattern: "NODE_ENV={env} npm run {script}".to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "env".to_string(),
                    description: "Node environment (development, production, test)".to_string(),
                    default_value: Some("development".to_string()),
                    required: false,
                    validation_pattern: Some(r"^(development|production|test)$".to_string()),
                },
                TemplateParameter {
                    name: "script".to_string(),
                    description: "npm script name".to_string(),
                    default_value: None,
                    required: true,
                    validation_pattern: None,
                },
            ],
            category: "nodejs".to_string(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            usage_count: 0,
        },
        Template {
            name: "ssh_tunnel".to_string(),
            description: "Create SSH tunnel".to_string(),
            pattern: "ssh -L {local_port}:localhost:{remote_port} {user}@{host} -N".to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "local_port".to_string(),
                    description: "Local port number".to_string(),
                    default_value: None,
                    required: true,
                    validation_pattern: Some(r"^\d+$".to_string()),
                },
                TemplateParameter {
                    name: "remote_port".to_string(),
                    description: "Remote port number".to_string(),
                    default_value: None,
                    required: true,
                    validation_pattern: Some(r"^\d+$".to_string()),
                },
                TemplateParameter {
                    name: "user".to_string(),
                    description: "SSH username".to_string(),
                    default_value: None,
                    required: true,
                    validation_pattern: None,
                },
                TemplateParameter {
                    name: "host".to_string(),
                    description: "SSH host".to_string(),
                    default_value: None,
                    required: true,
                    validation_pattern: None,
                },
            ],
            category: "network".to_string(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            usage_count: 0,
        },
    ]
}
