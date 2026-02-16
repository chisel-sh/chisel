use anyhow::{Context, Result};
use std::path::{PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use crate::{Issue, IssueFrontmatter};

pub fn load_issue_from_file(path: PathBuf) -> Result<Issue> {
    let raw = fs::read_to_string(&path)?;
    let metadata = fs::metadata(&path)?;
    let updated_at: DateTime<Utc> = metadata.modified()?.into();

    if !raw.starts_with("---") {
        anyhow::bail!("Issue file missing frontmatter: {}", path.display());
    }

    let parts: Vec<&str> = raw.splitn(3, "---").collect();
    if parts.len() != 3 {
         anyhow::bail!("Invalid issue file format: {}", path.display());
    }

    let fm: IssueFrontmatter = serde_yaml::from_str(parts[1])?;
    let content = parts[2].trim().to_string();

    let filename = path.file_name().and_then(|n| n.to_str()).context("Invalid filename")?;
    let id = filename.split('_').next().context("Invalid issue ID in filename")?.parse::<i64>()?;

    Ok(Issue {
        id,
        path,
        title: fm.title,
        status: fm.status,
        priority: fm.priority,
        labels: fm.labels,
        created_at: fm.created_at,
        updated_at,
        order: fm.order,
        content,
        external_id: fm.external_id,
    })
}
