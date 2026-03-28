use crate::{Spec, SpecFrontmatter};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn load_spec_from_file(path: PathBuf) -> Result<Spec> {
    let raw = fs::read_to_string(&path)?;

    if !raw.starts_with("---") {
        anyhow::bail!("Spec file missing frontmatter: {}", path.display());
    }

    let parts: Vec<&str> = raw.splitn(3, "---").collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid spec file format: {}", path.display());
    }

    let fm: SpecFrontmatter = serde_yaml::from_str(parts[1])?;
    let content = parts[2].trim().to_string();

    let slug = path
        .file_stem()
        .and_then(|n| n.to_str())
        .context("Invalid spec filename")?
        .to_string();

    Ok(Spec {
        slug,
        path,
        title: fm.title,
        status: fm.status,
        created: fm.created,
        updated: fm.updated,
        area: fm.area,
        related_docs: fm.related_docs,
        open_questions: fm.open_questions,
        content,
    })
}
