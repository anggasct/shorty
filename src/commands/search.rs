use std::fs;
use crate::utils::get_aliases_path;

pub fn search_aliases(query: &str) -> anyhow::Result<()> {
    let aliases_path = get_aliases_path();
    let contents = fs::read_to_string(&aliases_path)?;

    let results: Vec<&str> = contents
        .lines()
        .filter(|line| line.contains(query))
        .collect();

    if results.is_empty() {
        println!("No aliases found matching: {}", query);
    } else {
        for alias in results {
            println!("{}", alias);
        }
    }

    Ok(())
}
