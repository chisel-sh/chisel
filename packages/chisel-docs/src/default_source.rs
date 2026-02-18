use async_trait::async_trait;
use anyhow::Result;
use std::path::{PathBuf, Path};
use std::fs;
use ignore::WalkBuilder;
use chrono::{DateTime, Utc};
use crate::{Doc, parsing::{parse_frontmatter, parse_sections}, source::DataSource};

pub struct DefaultSource {
    pub root: PathBuf,
}

impl DefaultSource {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn resolve_category(&self, path: &Path) -> Option<String> {
        if let Ok(rel) = path.strip_prefix(&self.root) {
            return rel.parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.to_string_lossy().into_owned());
        }
        None
    }
}

#[async_trait]
impl DataSource for DefaultSource {
    async fn list(&self) -> Result<Vec<Doc>> {
        let mut docs = Vec::new();
        let walker = WalkBuilder::new(&self.root)
            .git_ignore(true)
            .hidden(false)
            .build();

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_some_and(|ft| ft.is_file()) 
               && entry.path().extension().is_some_and(|ext| ext == "md") 
               && entry.file_name() != "INDEX.md"
               && !entry.file_name().to_string_lossy().starts_with("_")
            {
                let path = entry.path().to_path_buf();
                let doc = self.load(path).await?;
                docs.push(doc);
            }
        }
        Ok(docs)
    }

    async fn load(&self, path: PathBuf) -> Result<Doc> {
        let metadata = fs::metadata(&path)?;
        let updated_at: DateTime<Utc> = metadata.modified()?.into();
        let raw_content = fs::read_to_string(&path)?;
        
        let (frontmatter, content) = parse_frontmatter(&raw_content);
        let sections = parse_sections(&content);
        let category = self.resolve_category(&path);

        Ok(Doc {
            name: path.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned()),
            path,
            updated_at,
            content: Some(content),
            frontmatter,
            sections,
            category,
        })
    }

    async fn save(&self, doc: &Doc) -> Result<()> {
        let mut content = String::new();
        if let Some(fm) = &doc.frontmatter {
            content.push_str("---
");
            content.push_str(&serde_yaml::to_string(fm)?);
            content.push_str("---

");
        }
        if let Some(body) = &doc.content {
            content.push_str(body);
        }

        if let Some(parent) = doc.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&doc.path, content)?;
        Ok(())
    }

    async fn delete(&self, path: PathBuf) -> Result<()> {
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn resolve_path(&self, category: Option<String>, slug: String) -> PathBuf {
        let mut path = self.root.clone();
        if let Some(cat) = category {
            path.push(cat);
        }
        path.push(format!("{}.md", slug));
        path
    }

    fn root(&self) -> PathBuf {
        self.root.clone()
    }

    fn index_link(&self, doc: &Doc) -> String {
        doc.name.clone()
    }
}
