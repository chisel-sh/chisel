use crate::{
    parsing::load_spec_from_file, source::SpecSource, Spec, SpecFrontmatter, SpecStatus,
};
use anyhow::Result;
use async_trait::async_trait;
use glob::glob;
use std::fs;
use std::path::PathBuf;

pub struct DefaultSpecSource {
    pub root: PathBuf,
}

impl DefaultSpecSource {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn subdirs(&self) -> [PathBuf; 3] {
        [
            self.root.join("active"),
            self.root.join("shipped"),
            self.root.join("archived"),
        ]
    }
}

#[async_trait]
impl SpecSource for DefaultSpecSource {
    async fn list(&self, status: Option<SpecStatus>) -> Result<Vec<Spec>> {
        let dirs = match &status {
            Some(s) => vec![self.root.join(s.directory())],
            None => self.subdirs().to_vec(),
        };

        let mut specs = Vec::new();
        for dir in dirs {
            if !dir.exists() {
                continue;
            }
            for path in glob(&format!("{}/*.md", dir.display()))?.flatten() {
                if let Ok(spec) = load_spec_from_file(path) {
                    specs.push(spec);
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
        let filename = format!("{}.md", slug);
        for dir in &self.subdirs() {
            let path = dir.join(&filename);
            if path.exists() {
                return load_spec_from_file(path);
            }
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

    async fn move_to_status_dir(&self, spec: &mut Spec, new_status: SpecStatus) -> Result<()> {
        let old_dir = spec.status.directory();
        let new_dir = new_status.directory();

        // Update spec fields
        spec.status = new_status.clone();
        spec.updated = chrono::Local::now().date_naive();

        if old_dir != new_dir {
            let new_path = self.resolve_path(&spec.slug, &new_status);
            if let Some(parent) = new_path.parent() {
                fs::create_dir_all(parent)?;
            }
            // Write to new location first, then remove old
            let old_path = spec.path.clone();
            spec.path = new_path;
            self.save(spec).await?;
            if old_path.exists() {
                fs::remove_file(old_path)?;
            }
        } else {
            // Same directory, just update frontmatter in place
            self.save(spec).await?;
        }

        Ok(())
    }

    fn resolve_path(&self, slug: &str, status: &SpecStatus) -> PathBuf {
        self.root
            .join(status.directory())
            .join(format!("{}.md", slug))
    }

    fn root(&self) -> PathBuf {
        self.root.clone()
    }
}
