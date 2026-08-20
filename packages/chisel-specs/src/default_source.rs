use crate::{parsing::load_spec_from_file, source::SpecSource, SpecFrontmatter, Spec, SpecStatus};
use anyhow::Result;
use async_trait::async_trait;
use glob::glob;
use std::fs;
use std::path::PathBuf;

pub struct DefaultSpecSource {
    pub root: PathBuf,
}

/// Subdirectories used by the legacy status-based layout
const LEGACY_STATUS_DIRS: [&str; 3] = ["active", "shipped", "archived"];

impl DefaultSpecSource {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Move spec files out of the legacy active/shipped/archived
    /// subdirectories into the flat layout, where status lives only in
    /// frontmatter. Slug collisions across status directories get a
    /// `-{dir}` suffix (then `-{dir}-2`, ...) since flat slugs must be
    /// unique. Emptied legacy directories are removed. Returns the
    /// relocations performed as (from, to) pairs.
    pub fn migrate_legacy_layout(&self) -> Result<Vec<(PathBuf, PathBuf)>> {
        let mut moves = Vec::new();
        for dir_name in LEGACY_STATUS_DIRS {
            let dir = self.root.join(dir_name);
            if !dir.exists() {
                continue;
            }
            for path in glob(&format!("{}/*.md", dir.display()))?.flatten() {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                let mut target = self.root.join(format!("{}.md", stem));
                if target.exists() {
                    target = self.root.join(format!("{}-{}.md", stem, dir_name));
                }
                let mut n = 2;
                while target.exists() {
                    target = self.root.join(format!("{}-{}-{}.md", stem, dir_name, n));
                    n += 1;
                }
                fs::rename(&path, &target)?;
                moves.push((path, target));
            }
            // Remove the legacy directory once it holds nothing else
            let _ = fs::remove_dir(&dir);
        }
        Ok(moves)
    }
}

#[async_trait]
impl SpecSource for DefaultSpecSource {
    async fn list(&self, status: Option<SpecStatus>) -> Result<Vec<Spec>> {
        let mut specs = Vec::new();
        if self.root.exists() {
            for path in glob(&format!("{}/*.md", self.root.display()))?.flatten() {
                if let Ok(spec) = load_spec_from_file(path) {
                    if status.as_ref().is_none_or(|s| *s == spec.status) {
                        specs.push(spec);
                    }
                }
            }
        }

        // Sort by status order, then by updated date descending
        specs.sort_by(|a, b| {
            a.status
                .cmp(&b.status)
                .then_with(|| b.updated.cmp(&a.updated))
        });

        Ok(specs)
    }

    async fn load(&self, slug: &str) -> Result<Spec> {
        let path = self.resolve_path(slug);
        if path.exists() {
            return load_spec_from_file(path);
        }
        anyhow::bail!("Spec '{}' not found", slug)
    }

    async fn save(&self, spec: &Spec) -> Result<()> {
        let fm = SpecFrontmatter {
            title: spec.title.clone(),
            status: spec.status.clone(),
            created: spec.created,
            updated: spec.updated,
            area: spec.area.clone(),
            related_docs: spec.related_docs.clone(),
            open_questions: spec.open_questions.clone(),
        };

        let file_content = format!(
            "---\n{}---\n\n{}",
            serde_yaml::to_string(&fm)?,
            spec.content
        );

        if let Some(parent) = spec.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&spec.path, file_content)?;
        Ok(())
    }

    async fn delete(&self, slug: &str) -> Result<()> {
        let spec = self.load(slug).await?;
        if spec.path.exists() {
            fs::remove_file(spec.path)?;
        }
        Ok(())
    }

    async fn set_status(&self, spec: &mut Spec, new_status: SpecStatus) -> Result<()> {
        spec.status = new_status;
        spec.updated = chrono::Local::now().date_naive();
        self.save(spec).await
    }

    fn resolve_path(&self, slug: &str) -> PathBuf {
        self.root.join(format!("{}.md", slug))
    }

    fn root(&self) -> PathBuf {
        self.root.clone()
    }
}
