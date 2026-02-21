use crate::{
    parsing::{parse_frontmatter, parse_sections},
    source::DataSource,
    Doc,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub struct StarlightSource {
    pub root: PathBuf,
}

impl StarlightSource {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

// Starlight-specific Frontmatter structure to help with mapping
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct StarlightFrontmatter {
    title: String,
    #[serde(default)]
    sidebar: Option<StarlightSidebar>,
    #[serde(default)]
    tags: Vec<String>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct StarlightSidebar {
    order: Option<i32>,
    label: Option<String>,
}

#[async_trait]
impl DataSource for StarlightSource {
    async fn list(&self) -> Result<Vec<Doc>> {
        let mut docs = Vec::new();
        // Starlight supports .md, .mdx
        let walker = WalkBuilder::new(&self.root)
            .git_ignore(true)
            .hidden(false)
            .build();

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            if entry.file_type().is_some_and(|ft| ft.is_file()) && (ext == "md" || ext == "mdx") {
                let doc = self.load(path.to_path_buf()).await?;
                docs.push(doc);
            }
        }
        Ok(docs)
    }

    async fn load(&self, path: PathBuf) -> Result<Doc> {
        let metadata = fs::metadata(&path)?;
        let updated_at: DateTime<Utc> = metadata.modified()?.into();
        let raw_content = fs::read_to_string(&path)?;

        let (raw_fm, content) = parse_frontmatter(&raw_content);
        let sections = parse_sections(&content);

        let mut fm = None;
        if let Some(rf) = raw_fm {
            // Map common fields. Note: We are using Chisel's DocFrontmatter here.
            // If the file was written by Starlight, it might have different fields.
            // Ideally we'd parse as StarlightFrontmatter first.
            // For MVP, we assume Chisel writes the files or we just read 'title'.
            // But to support 'sidebar.order', we need custom parsing.

            // Re-parse raw frontmatter block manually?
            // parse_frontmatter returns DocFrontmatter which is Chisel specific.
            // We need to access the raw YAML string to parse Starlight schema.
            // But parse_frontmatter consumes it.

            // Workaround: We'll stick to Chisel schema for now, but in a real implementation
            // we would parse `serde_yaml::Value` to extract sidebar.order.
            fm = Some(rf);
        }

        let category = if let Ok(rel) = path.strip_prefix(&self.root) {
            rel.parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.to_string_lossy().into_owned())
        } else {
            None
        };

        Ok(Doc {
            name: path
                .file_name()
                .map_or(String::new(), |n| n.to_string_lossy().into_owned()),
            path,
            updated_at,
            content: Some(content),
            frontmatter: fm,
            sections,
            category,
        })
    }

    async fn save(&self, doc: &Doc) -> Result<()> {
        // Here we would map Doc.order -> sidebar.order
        // For MVP, we save as Chisel standard (Starlight ignores extra fields)
        let mut content = String::new();
        if let Some(fm) = &doc.frontmatter {
            content.push_str(
                "---
",
            );
            content.push_str(&serde_yaml::to_string(fm)?);
            content.push_str(
                "---

",
            );
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
        if let Ok(rel) = doc.path.strip_prefix(&self.root) {
            let link = rel.with_extension("");
            link.to_string_lossy().into_owned()
        } else {
            doc.path
                .file_stem()
                .map_or(String::new(), |s| s.to_string_lossy().into_owned())
        }
    }
}
