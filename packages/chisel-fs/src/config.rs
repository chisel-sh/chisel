use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChiselConfig {
    pub docs: Option<DocsConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocsConfig {
    pub source: Option<PathBuf>,
}

impl ChiselConfig {
    pub fn load(root: &Path) -> Result<Self> {
        let config_path = root.join("chisel.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
}
