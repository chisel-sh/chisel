use serde::{Serialize, Deserialize};
use anyhow::{Context, Result};
use std::path::{PathBuf, Path};
use std::fs;
use chrono::{DateTime, Utc};
use chisel_store::{Store, SearchResult};
use chisel_fs::{slugify_title, spawn_editor, config::ChiselConfig};
use chisel_render::Renderable;

pub mod source;
pub mod parsing;
pub mod default_source;
pub mod starlight_source;

use source::DataSource;
use default_source::DefaultSource;
use starlight_source::StarlightSource;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Doc {
    pub name: String,
    pub path: PathBuf,
    pub category: Option<String>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmatter: Option<DocFrontmatter>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<DocSection>,
}

impl chisel_render::MachineOutput for Doc {}

impl Renderable for Doc {
    fn render_human(&self) -> Result<()> {
        if let Some(content) = &self.content {
            println!("{}", content);
        } else {
            println!("Document: {} ({})", self.name, self.path.display());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocList(pub Vec<Doc>);

impl chisel_render::MachineOutput for DocList {}

impl Renderable for DocList {
    fn render_human(&self) -> Result<()> {
        if self.0.is_empty() {
            println!("No documents found.");
            return Ok(());
        }

        println!("DOCUMENTS:");
        let mut current_cat = None;
        let mut sorted = self.0.clone();
        DocList::sort_docs(&mut sorted);

        for doc in sorted {
            if doc.category != current_cat {
                current_cat = doc.category.clone();
                println!("\n[{}]", current_cat.as_deref().unwrap_or("GENERAL").to_uppercase());
            }
            println!("  - {} ({})", doc.name, doc.path.display());
        }
        Ok(())
    }
}

impl DocList {
    pub fn sort_docs(docs: &mut [Doc]) {
        docs.sort_by(|a, b| {
            let a_order = a.frontmatter.as_ref().and_then(|f| f.order).unwrap_or(i32::MAX);
            let b_order = b.frontmatter.as_ref().and_then(|f| f.order).unwrap_or(i32::MAX);
            a_order.cmp(&b_order).then_with(|| a.name.cmp(&b.name))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_sorting() {
        let mut docs = vec![
            Doc {
                name: "z.md".to_string(),
                path: PathBuf::from("z.md"),
                category: None,
                updated_at: Utc::now(),
                content: None,
                frontmatter: Some(DocFrontmatter { title: "Z".to_string(), order: Some(10), ..Default::default() }),
                sections: vec![],
            },
            Doc {
                name: "a.md".to_string(),
                path: PathBuf::from("a.md"),
                category: None,
                updated_at: Utc::now(),
                content: None,
                frontmatter: Some(DocFrontmatter { title: "A".to_string(), order: Some(1), ..Default::default() }),
                sections: vec![],
            },
            Doc {
                name: "b.md".to_string(),
                path: PathBuf::from("b.md"),
                category: None,
                updated_at: Utc::now(),
                content: None,
                frontmatter: Some(DocFrontmatter { title: "B".to_string(), order: Some(1), ..Default::default() }),
                sections: vec![],
            },
        ];

        DocList::sort_docs(&mut docs);

        assert_eq!(docs[0].name, "a.md");
        assert_eq!(docs[1].name, "b.md");
        assert_eq!(docs[2].name, "z.md");
    }

    #[tokio::test]
    async fn test_docs_service_flow() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        
        // Initialize service
        let service = DocsService::new(root.clone()).await.unwrap();
        service.init("Test Project").await.unwrap();
        
        // Check initial seed docs
        let docs = service.list(ListOptions::default()).await.unwrap();
        assert!(docs.0.len() >= 2);
        
        // Create new doc
        let new_doc = service.create("My New Doc", Some("ideas".to_string())).await.unwrap();
        assert_eq!(new_doc.name, "my-new-doc.md");
        assert!(root.join(".chisel/docs/ideas/my-new-doc.md").exists());
        
        // Re-list and verify
        let docs = service.list(ListOptions::default()).await.unwrap();
        assert!(docs.0.iter().any(|d| d.name == "my-new-doc.md"));
    }
}




#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceOverview {
    pub root: PathBuf,
    pub stats: WorkspaceStats,
    pub topology: Vec<CategorySummary>,
    pub recent_changes: Vec<DocSummary>,
}

impl chisel_render::MachineOutput for WorkspaceOverview {}

impl Renderable for WorkspaceOverview {
    fn render_human(&self) -> Result<()> {
        println!("WORKSPACE: {}", self.root.display());
        println!("STATS: {} docs in {} categories", self.stats.total_documents, self.stats.categories);
        
        println!("\nTOPOLOGY:");
        for cat in &self.topology {
            println!("  {:<20} {}", cat.category, cat.count);
        }

        println!("\nRECENT CHANGES:");
        for doc in &self.recent_changes {
            println!("  {:<30} {}", doc.name, doc.updated_at.format("%Y-%m-%d %H:%M"));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceStats {
    pub total_documents: usize,
    pub categories: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategorySummary {
    pub category: String,
    pub count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocSummary {
    pub name: String,
    pub path: PathBuf,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocSection {
    pub level: u8,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DocFrontmatter {
    pub title: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub order: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CategoryMetadata {
    pub label: Option<String>,
    pub order: Option<i32>,
}

#[derive(Default)]
pub struct ListOptions {
    pub root: PathBuf,
    pub use_gitignore: bool,
    pub include_hidden: bool,
}

#[derive(Serialize)]
pub struct SearchResults(pub Vec<SearchResult>);

impl chisel_render::MachineOutput for SearchResults {}

impl Renderable for SearchResults {
    fn render_human(&self) -> Result<()> {
        if self.0.is_empty() {
            println!("No results found.");
        } else {
            for res in &self.0 {
                println!("- {} ({})", res.name, res.path);
                println!("  {}", res.excerpt);
            }
        }
        Ok(())
    }
}

pub struct DocsService {
    pub store: Option<Store>,
    pub workspace_root: PathBuf,
    pub source: Box<dyn DataSource>,
}

impl DocsService {
    pub async fn new(root: PathBuf) -> Result<Self> {
        let store = Store::new(root.clone()).await?;
        let config = ChiselConfig::load(&root).unwrap_or_default();
        
        let starlight_path = config.docs.as_ref()
            .and_then(|d| d.source.clone())
            .map(|s| root.join(s))
            .filter(|p| p.exists())
            .unwrap_or_else(|| root.join("src").join("content").join("docs"));

        let source: Box<dyn DataSource> = if starlight_path.exists() {
            Box::new(StarlightSource::new(starlight_path))
        } else {
            let chisel_docs = root.join(".chisel").join("docs");
            Box::new(DefaultSource::new(chisel_docs))
        };

        Ok(Self { 
            store: Some(store), 
            workspace_root: root,
            source
        })
    }

    pub async fn list(&self, _options: ListOptions) -> Result<DocList> {
        let mut docs: Vec<Doc> = self.source.list().await?;
        DocList::sort_docs(&mut docs);
        Ok(DocList(docs))
    }

    pub async fn search(&self, query: &str) -> Result<SearchResults> {
        let store = self.store.as_ref().context("Store not initialized")?;
        Ok(SearchResults(store.search_fts(query).await?))
    }

    pub async fn show(&self, path: PathBuf) -> Result<Doc> {
        self.source.load(path).await
    }

    pub async fn overview(&self) -> Result<WorkspaceOverview> {
        let docs: Vec<Doc> = self.source.list().await?;
        
        let mut cat_counts = std::collections::BTreeMap::new();
        for doc in &docs {
            let cat = doc.category.as_deref().unwrap_or("GENERAL").to_string();
            *cat_counts.entry(cat).or_insert(0) += 1;
        }

        let topology: Vec<CategorySummary> = cat_counts.into_iter()
            .map(|(category, count)| CategorySummary { category, count })
            .collect();

        let mut recent = docs.clone();
        recent.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        let recent_changes = recent.into_iter()
            .take(5)
            .map(|d| DocSummary {
                name: d.name,
                path: d.path,
                updated_at: d.updated_at,
            })
            .collect();

        Ok(WorkspaceOverview {
            root: self.workspace_root.clone(),
            stats: WorkspaceStats {
                total_documents: docs.len(),
                categories: topology.len(),
            },
            topology,
            recent_changes,
        })
    }

    pub async fn create(&self, title: &str, category: Option<String>) -> Result<Doc> {
        let slug = slugify_title(title);
        let path = self.source.resolve_path(category.clone(), slug);
        
        let fm = DocFrontmatter {
            title: title.to_string(),
            created_at: Utc::now(),
            tags: Vec::new(),
            order: None,
        };
        
        let content = format!(
            "---\n{}---\n\n# {}\n\nStart writing here...",
            serde_yaml::to_string(&fm)?,
            title
        );
        
        let doc = Doc {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            path: path.clone(),
            updated_at: Utc::now(),
            content: Some(content),
            frontmatter: Some(fm),
            sections: vec![],
            category, 
        };
        
        self.save_and_sync(&doc).await?;
        
        Ok(doc)
    }

    async fn save_and_sync(&self, doc: &Doc) -> Result<()> {
        self.source.save(doc).await?;
        self.index_doc_internal(doc).await?;
        self.rebuild_index().await
    }

    async fn index_doc_internal(&self, doc: &Doc) -> Result<()> {
        if let Some(store) = &self.store {
             if let Some(content) = &doc.content {
                let tags = doc.frontmatter.as_ref().map(|f| f.tags.join(", "));
                let title = doc.frontmatter.as_ref().map(|f| f.title.as_str());
                let created_at = doc.frontmatter.as_ref().map(|f| f.created_at);

                store.update_doc(
                    &doc.path.to_string_lossy(),
                    &doc.name,
                    title,
                    tags.as_deref(),
                    content,
                    created_at,
                ).await?;
            }
        }
        Ok(())
    }

    pub async fn edit(&self, path: PathBuf) -> Result<Doc> {
        let doc = self.source.load(path).await?;
        spawn_editor(&doc.path)?;
        
        let updated = self.source.load(doc.path.clone()).await?;
        self.save_and_sync(&updated).await?;
        
        Ok(updated)
    }

    pub async fn move_doc(&self, path: PathBuf, new_category: Option<String>) -> Result<Doc> {
        let mut doc = self.source.load(path).await?;
        let old_path = doc.path.clone();
        
        let slug = doc.path.file_stem().context("Invalid path")?.to_string_lossy().into_owned();
        let new_path = self.source.resolve_path(new_category.clone(), slug);
        
        doc.path = new_path;
        doc.category = new_category;
        
        // Move is a bit special as it involves delete
        self.source.save(&doc).await?;
        self.source.delete(old_path).await?;
        
        self.index_doc_internal(&doc).await?;
        self.rebuild_index().await?;
        
        Ok(doc)
    }

    pub async fn update_doc_order(&self, path: PathBuf, order: i32) -> Result<Doc> {
        let mut doc = self.source.load(path).await?;
        
        let mut fm = doc.frontmatter.clone().unwrap_or_default();
        fm.order = Some(order);
        doc.frontmatter = Some(fm);

        self.save_and_sync(&doc).await?;
        
        Ok(doc)
    }

    pub fn update_category_order(&self, category: &str, order: i32) -> Result<()> {
        if category == "[ALL]" || category == "GENERAL" {
            return Ok(());
        }
        
        let mut meta = get_category_metadata(&self.source.root(), category);
        meta.order = Some(order);
        
        let path = self.source.root().join(category).join("_category.yaml");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_yaml::to_string(&meta)?)?;
        Ok(())
    }

    pub async fn fuzzy_search(&self, query: &str) -> Result<SearchResults> {
        let store = self.store.as_ref().context("Store not initialized")?;
        Ok(SearchResults(store.fuzzy_search(query).await?))
    }

    pub async fn delete(&self, path: PathBuf) -> Result<()> {
        self.source.delete(path).await?;
        self.rebuild_index().await?;
        Ok(())
    }

    pub async fn index_all(&self) -> Result<()> {
        let docs = self.source.list().await?;
        for doc in docs {
            if let Ok(full_doc) = self.source.load(doc.path).await {
                let _ = self.index_doc_internal(&full_doc).await;
            }
        }
        self.rebuild_index().await?;
        Ok(())
    }

    pub async fn rebuild_index(&self) -> Result<()> {
        let mut docs = self.source.list().await?;
        
        let mut categories_meta = std::collections::HashMap::new();
        for doc in &docs {
            let cat_id = doc.category.as_deref().unwrap_or("GENERAL");
            if !categories_meta.contains_key(cat_id) {
                categories_meta.insert(cat_id.to_string(), get_category_metadata(&self.source.root(), cat_id));
            }
        }

        docs.sort_by(|a, b| {
            let a_cat_id = a.category.as_deref().unwrap_or("GENERAL");
            let b_cat_id = b.category.as_deref().unwrap_or("GENERAL");
            
            if a_cat_id == b_cat_id {
                let a_order = a.frontmatter.as_ref().and_then(|f| f.order).unwrap_or(i32::MAX);
                let b_order = b.frontmatter.as_ref().and_then(|f| f.order).unwrap_or(i32::MAX);
                a_order.cmp(&b_order).then_with(|| a.name.cmp(&b.name))
            } else {
                let a_meta = &categories_meta[a_cat_id];
                let b_meta = &categories_meta[b_cat_id];
                a_meta.order.unwrap_or(i32::MAX).cmp(&b_meta.order.unwrap_or(i32::MAX))
                    .then_with(|| a_cat_id.cmp(b_cat_id))
            }
        });

        let mut index_content = String::new();

        let mut current_category = None;

        for doc in docs {
            if doc.category != current_category {
                current_category = doc.category.clone();
                let cat_label = current_category.as_deref().unwrap_or("General");
                index_content.push_str(&format!("\n### {}\n", cat_label.to_uppercase()));
            }

            let name = doc.path.file_stem().map_or("Unknown", |s| s.to_str().unwrap_or(""));
            let link = self.source.index_link(&doc);
            index_content.push_str(&format!("- [{}]({})\n", name, link));
        }

        let mut final_content = String::from("---\ntitle: Chisel Docs\n---\n\n# Chisel Docs\n\n");
        final_content.push_str("Automatically managed by Chisel.\n\n");
        final_content.push_str(&index_content);

        let index_path = self.source.root().join("index.md");
        fs::write(index_path, final_content)?;
        Ok(())
    }

    pub async fn init(&self, _project_name: &str) -> Result<()> {
        // 1. Welcome
        let welcome_path = self.source.resolve_path(None, "welcome-to-chisel".to_string());
        if !welcome_path.exists() {
             let fm = DocFrontmatter {
                title: "Welcome to Chisel".to_string(),
                created_at: Utc::now(),
                tags: vec!["chisel".to_string(), "onboarding".to_string()],
                order: None,
            };
            let content = "Chisel is a text-first toolkit for shaping information. Everything you see here is stored as plain Markdown files in your project.\n\n### Quick Start\n- Use **Docs** to manage your project's knowledge base.\n- Use **Issues** to track tasks in a local Kanban board.\n\n### Navigation\n- `Tab`: Switch between Category, Document, and Preview panes.\n- `j`/`k`: Move selection up and down.\n- `q`: Exit the TUI and return to your shell.";
            
            let doc = Doc {
                name: "welcome-to-chisel.md".to_string(),
                path: welcome_path,
                updated_at: Utc::now(),
                content: Some(content.to_string()),
                frontmatter: Some(fm),
                sections: vec![],
                category: None,
            };
            self.source.save(&doc).await?;
        }

        // 2. Chisel Docs Tutorial
        let tutorial_dir = "tutorial".to_string();
        let working_path = self.source.resolve_path(Some(tutorial_dir.clone()), "working-with-docs".to_string());
        if !working_path.exists() {
             let fm = DocFrontmatter {
                title: "Working with Docs".to_string(),
                created_at: Utc::now(),
                tags: vec!["tutorial".to_string(), "docs".to_string()],
                order: None,
            };
            let content = "Docs are stored in `.chisel/docs/`. You can organize them into categories by creating subdirectories.\n\n### Key Actions\n- `n`: Create a new document.\n- `e`: Edit the selected document in your `$EDITOR`.\n- `m`: Move a document to a different category.\n- `/`: Search all documents (uses local SQLite index).";
            
            let doc = Doc {
                name: "working-with-docs.md".to_string(),
                path: working_path,
                updated_at: Utc::now(),
                content: Some(content.to_string()),
                frontmatter: Some(fm),
                sections: vec![],
                category: Some(tutorial_dir.clone()),
            };
            self.source.save(&doc).await?;
        }

        self.index_all().await?;
        Ok(())
    }
}

pub async fn index_docs(root: PathBuf, store: &Store) -> Result<()> {
    let chisel_docs = root.join(".chisel").join("docs");
    let source = DefaultSource::new(chisel_docs);
    let docs = source.list().await?;
    for doc in docs {
        if let Ok(full_doc) = source.load(doc.path).await {
             if let Some(content) = &full_doc.content {
                let tags = full_doc.frontmatter.as_ref().map(|f| f.tags.join(", "));
                let title = full_doc.frontmatter.as_ref().map(|f| f.title.as_str());
                let created_at = full_doc.frontmatter.as_ref().map(|f| f.created_at);

                store.update_doc(
                    &full_doc.path.to_string_lossy(),
                    &full_doc.name,
                    title,
                    tags.as_deref(),
                    content,
                    created_at,
                ).await?;
            }
        }
    }
    Ok(())
}

pub async fn search_docs(query: &str, store: &Store) -> Result<Vec<SearchResult>> {
    store.search_fts(query).await
}

pub fn get_category_metadata(docs_root: &Path, category: &str) -> CategoryMetadata {
    if category == "[ALL]" {
        return CategoryMetadata { label: Some("[ALL]".to_string()), order: Some(-1) };
    }
    let path = docs_root.join(category).join("_category.yaml");
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(meta) = serde_yaml::from_str::<CategoryMetadata>(&content) {
                return meta;
            }
        }
    }
    CategoryMetadata {
        label: Some(category.to_string()),
        order: None,
    }
}
