use crate::utils::get_aliases_path;
use std::fs;

pub fn list_aliases(tag: Option<&str>) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;

    if let Some(tag) = tag {
        let filtered: Vec<&str> = contents
            .lines()
            .filter(|line| line.contains(&format!("#tags:{tag}")))
            .collect();

        if filtered.is_empty() {
            println!("No aliases found with tag: {tag}");
        } else {
            for alias in filtered {
                println!("{alias}");
            }
        }
    } else {
        println!("{contents}");
    }

    Ok(())
}
