use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};

const REPO: &str = "chisel-sh/chisel";
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCheck {
    pub last_checked: SystemTime,
    pub latest_version: Option<String>,
}

pub struct UpgradeService {
    workspace_root: PathBuf,
}

impl UpgradeService {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    fn check_file_path(&self) -> PathBuf {
        self.workspace_root
            .join(".chisel")
            .join("update_check.json")
    }

    pub async fn check_for_updates(&self) -> Result<Option<String>> {
        let current_version = env!("CARGO_PKG_VERSION");
        let check_file = self.check_file_path();

        if let Ok(content) = std::fs::read_to_string(&check_file) {
            if let Ok(check) = serde_json::from_str::<UpdateCheck>(&content) {
                if let Ok(elapsed) = check.last_checked.elapsed() {
                    if elapsed < CHECK_INTERVAL {
                        if let Some(latest) = check.latest_version {
                            if self.is_newer(&latest, current_version) {
                                return Ok(Some(latest));
                            }
                        }
                        return Ok(None);
                    }
                }
            }
        }

        // Time to check again
        let latest = self.fetch_latest_version().await?;
        let check = UpdateCheck {
            last_checked: SystemTime::now(),
            latest_version: Some(latest.clone()),
        };

        if let Ok(json) = serde_json::to_string(&check) {
            let _ = std::fs::write(&check_file, json);
        }

        if self.is_newer(&latest, current_version) {
            Ok(Some(latest))
        } else {
            Ok(None)
        }
    }

    async fn fetch_latest_version(&self) -> Result<String> {
        let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
        let client = reqwest::Client::builder()
            .user_agent("chisel-cli")
            .build()?;

        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("Failed to fetch latest version from GitHub");
        }

        let json: serde_json::Value = resp.json().await?;
        let tag = json["tag_name"]
            .as_str()
            .context("Invalid tag_name in GitHub response")?;

        // Remove 'chisel-v' or 'v' prefix if present
        Ok(tag
            .trim_start_matches("chisel-v")
            .trim_start_matches('v')
            .to_string())
    }

    fn is_newer(&self, latest: &str, current: &str) -> bool {
        // Simple semver compare (split by .)
        let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();
        let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

        for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
            if l > c {
                return true;
            }
            if l < c {
                return false;
            }
        }
        latest_parts.len() > current_parts.len()
    }

    pub fn perform_update(&self) -> Result<()> {
        println!("🚀 Updating Chisel...");

        let status = Command::new("sh")
            .arg("-c")
            .arg("curl -sL https://install.chisel.build | sh")
            .status()
            .context("Failed to run installation script")?;

        if status.success() {
            println!("✨ Chisel updated successfully.");
        } else {
            anyhow::bail!("Update failed. Please try running the installation script manually: curl -sL https://install.chisel.build | sh");
        }

        Ok(())
    }
}
