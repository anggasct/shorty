use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const GITHUB_API_URL: &str = "https://api.github.com/repos/anggasct/shorty/releases/latest";
const USER_AGENT: &str = concat!("shorty/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize, Serialize)]
pub struct Release {
    pub tag_name: String,
    pub body: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

pub fn get_latest_release(timeout_secs: u64) -> Result<Release> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(USER_AGENT)
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(GITHUB_API_URL)
        .send()
        .context("Failed to fetch latest release from GitHub")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "GitHub API returned error: {}",
            response.status()
        ));
    }

    let release: Release = response
        .json()
        .context("Failed to parse GitHub API response")?;

    Ok(release)
}

pub fn compare_versions(current: &str, latest: &str) -> VersionComparison {
    let current_clean = current.trim_start_matches('v');
    let latest_clean = latest.trim_start_matches('v');

    match current_clean.cmp(latest_clean) {
        std::cmp::Ordering::Less => VersionComparison::UpdateAvailable,
        std::cmp::Ordering::Equal => VersionComparison::UpToDate,
        std::cmp::Ordering::Greater => VersionComparison::Ahead,
    }
}

#[derive(Debug, PartialEq)]
pub enum VersionComparison {
    UpdateAvailable,
    UpToDate,
    Ahead,
}

pub fn get_platform_binary_name() -> &'static str {
    #[cfg(target_os = "linux")]
    return "shorty-linux";

    #[cfg(target_os = "macos")]
    return "shorty-macos";

    #[cfg(target_os = "windows")]
    return "shorty-windows.exe";

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    compile_error!("Unsupported platform");
}

pub fn find_asset_url(release: &Release) -> Result<String> {
    let binary_name = get_platform_binary_name();

    release
        .assets
        .iter()
        .find(|asset| asset.name == binary_name)
        .map(|asset| asset.browser_download_url.clone())
        .ok_or_else(|| anyhow!("No binary found for platform: {}", binary_name))
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
