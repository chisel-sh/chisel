use anyhow::{Context, Result};
use chisel_fs::slugify_title;
use chisel_fs::spawn_editor;
use chisel_render::Renderable;
use chisel_store::Store;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

pub mod default_source;
pub mod parsing;
pub mod source;

use default_source::DefaultSpecSource;
use source::SpecSource;

// --- Core Types ---

#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Display, EnumString,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SpecStatus {
    Draft,
    Ready,
    InProgress,
    Shipped,
    Archived,
}

impl SpecStatus {
    pub fn directory(&self) -> &'static str {
        match self {
            SpecStatus::Draft | SpecStatus::Ready | SpecStatus::InProgress => "active",
            SpecStatus::Shipped => "shipped",
            SpecStatus::Archived => "archived",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpecFrontmatter {
    pub title: String,
    pub status: SpecStatus,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_docs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_questions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Spec {
    pub slug: String,
    pub path: PathBuf,
    pub title: String,
    pub status: SpecStatus,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    pub area: Option<String>,
    pub related_docs: Vec<String>,
    pub open_questions: Vec<String>,
    pub content: String,
}

impl chisel_render::MachineOutput for Spec {}

impl Renderable for Spec {
    fn render_human(&self) -> Result<()> {
        println!("{}", self.title);
        println!("Slug:    {}", self.slug);
        println!("Status:  {}", self.status);
        if let Some(ref area) = self.area {
            println!("Area:    {}", area);
        }
        println!("Created: {}", self.created);
        println!("Updated: {}", self.updated);
        if !self.open_questions.is_empty() {
            println!("\nOpen Questions:");
            for q in &self.open_questions {
                println!("  - {}", q);
            }
        }
        if !self.related_docs.is_empty() {
            println!("\nRelated Docs:");
            for d in &self.related_docs {
                println!("  - {}", d);
            }
        }
        println!("\n---\n\n{}", self.content);
        Ok(())
    }
}

// --- List Types ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpecList(pub Vec<chisel_store::SpecRow>);

impl chisel_render::MachineOutput for SpecList {
    fn to_machine_string(&self) -> Result<String> {
        let mut sorted = self.0.clone();
        SpecList::sort_specs(&mut sorted);
        Ok(serde_yaml::to_string(&sorted)?)
    }
}

impl SpecList {
    pub fn sort_specs(specs: &mut [chisel_store::SpecRow]) {
        specs.sort_by(|a, b| {
            let a_status = SpecStatus::from_str(&a.status).unwrap_or(SpecStatus::Draft);
            let b_status = SpecStatus::from_str(&b.status).unwrap_or(SpecStatus::Draft);
            a_status
                .cmp(&b_status)
                .then_with(|| b.updated.cmp(&a.updated))
        });
    }
}

impl Renderable for SpecList {
    fn render_human(&self) -> Result<()> {
        if self.0.is_empty() {
            println!("No specs found.");
            return Ok(());
        }

        let mut sorted = self.0.clone();
        SpecList::sort_specs(&mut sorted);

        println!(
            "{:<25} {:<30} {:<14} {:<10}",
            "SLUG", "TITLE", "STATUS", "AREA"
        );
        println!("{}", "-".repeat(79));
        for spec in sorted {
            println!(
                "{:<25} {:<30} {:<14} {:<10}",
                chisel_fs::truncate_with_ellipsis(&spec.slug, 23),
                chisel_fs::truncate_with_ellipsis(&spec.title, 28),
                spec.status,
                spec.area.as_deref().unwrap_or("-"),
            );
        }
        Ok(())
    }
}

// --- Templates ---

pub enum SpecTemplate {
    Feature,
    Adr,
    Minimal,
}

impl SpecTemplate {
    pub fn content(&self) -> &'static str {
        match self {
            SpecTemplate::Feature => "\
## What and Why

<!-- What are we building? Why now? What problem does it solve? -->

## Success Criteria

<!-- How will we know this worked? What does good look like? -->

## Constraints

<!-- Technical constraints, product constraints, what's out of scope -->

## Approach

<!-- The plan. How will this be built? -->

## Alternatives Considered

<!-- What else was evaluated? Why was this approach chosen? -->

## Open Questions

<!-- What is still unresolved? Who can resolve it? -->

## Implementation Notes

<!-- Updated during/after implementation: what changed from the plan and why -->",
            SpecTemplate::Adr => "\
## Context

<!-- What situation prompted this decision? -->

## Decision

<!-- What was decided? -->

## Consequences

<!-- What becomes easier or harder as a result? -->",
            SpecTemplate::Minimal => "\
## Summary

<!-- Brief description of this spec -->",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "feature" => Some(SpecTemplate::Feature),
            "adr" => Some(SpecTemplate::Adr),
            _ => None,
        }
    }
}

// --- Service ---

pub struct SpecsService {
    pub store: Option<Store>,
    pub root: PathBuf,
    pub source: Box<dyn SpecSource>,
}

impl SpecsService {
    pub async fn new(project_root: PathBuf) -> Result<Self> {
        let store = Store::new(project_root.clone()).await?;
        let config = chisel_fs::config::ChiselConfig::load(&project_root).unwrap_or_default();

        let specs_dir = config
            .specs
            .as_ref()
            .and_then(|s| s.source.clone())
            .map(|s| project_root.join(s))
            .unwrap_or_else(|| project_root.join(".chisel").join("specs"));

        let source = Box::new(DefaultSpecSource::new(specs_dir));
        Ok(Self {
            store: Some(store),
            root: project_root,
            source,
        })
    }

    pub async fn create(
        &self,
        title: &str,
        area: Option<String>,
        template: Option<&str>,
        content: Option<&str>,
    ) -> Result<Spec> {
        let slug = slugify_title(title);
        let today = chrono::Local::now().date_naive();
        let path = self.source.resolve_path(&slug, &SpecStatus::Draft);

        let template_content = template
            .and_then(SpecTemplate::from_name)
            .map(|t| t.content().to_string())
            .unwrap_or_else(|| {
                content
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| SpecTemplate::Feature.content().to_string())
            });

        let spec = Spec {
            slug,
            path,
            title: title.to_string(),
            status: SpecStatus::Draft,
            created: today,
            updated: today,
            area,
            related_docs: Vec::new(),
            open_questions: Vec::new(),
            content: template_content,
        };

        self.save_and_sync(&spec).await?;
        Ok(spec)
    }

    async fn save_and_sync(&self, spec: &Spec) -> Result<()> {
        self.source.save(spec).await?;
        self.index_spec(spec).await
    }

    pub async fn list(&self, status: Option<SpecStatus>) -> Result<SpecList> {
        let specs = self.source.list(status).await?;
        let rows = specs
            .into_iter()
            .map(|s| chisel_store::SpecRow {
                slug: s.slug,
                path: s.path.to_string_lossy().to_string(),
                title: s.title,
                status: s.status.to_string(),
                area: s.area,
                created: s.created,
                updated: s.updated,
                excerpt: chisel_fs::truncate_with_ellipsis(&s.content, 100),
            })
            .collect();

        let mut list = SpecList(rows);
        SpecList::sort_specs(&mut list.0);
        Ok(list)
    }

    pub async fn show(&self, slug: &str) -> Result<Spec> {
        self.source.load(slug).await
    }

    pub async fn update_status(&self, slug: &str, new_status: SpecStatus) -> Result<Spec> {
        let mut spec = self.source.load(slug).await?;
        self.source.move_to_status_dir(&mut spec, new_status).await?;
        self.index_spec(&spec).await?;
        Ok(spec)
    }

    pub async fn edit(&self, slug: &str) -> Result<Spec> {
        let spec = self.source.load(slug).await?;

        let mut temp_path = std::env::temp_dir();
        temp_path.push(format!("chisel_spec_{}.md", slug));

        // Write full file (frontmatter + content) for editing
        let fm = SpecFrontmatter {
            title: spec.title.clone(),
            status: spec.status.clone(),
            created: spec.created,
            updated: spec.updated,
            area: spec.area.clone(),
            related_docs: spec.related_docs.clone(),
            open_questions: spec.open_questions.clone(),
        };
        let file_content = format!("---\n{}---\n\n{}", serde_yaml::to_string(&fm)?, spec.content);
        fs::write(&temp_path, file_content)?;

        spawn_editor(&temp_path)?;

        // Re-parse the edited file
        let edited = parsing::load_spec_from_file(temp_path.clone())?;
        let _ = fs::remove_file(&temp_path);

        // Build the updated spec preserving the real path
        let mut updated = Spec {
            slug: spec.slug,
            path: spec.path,
            title: edited.title,
            status: edited.status,
            created: edited.created,
            updated: chrono::Local::now().date_naive(),
            area: edited.area,
            related_docs: edited.related_docs,
            open_questions: edited.open_questions,
            content: edited.content,
        };

        // If status changed, handle directory move
        if updated.status != spec.status {
            let new_status = updated.status.clone();
            self.source
                .move_to_status_dir(&mut updated, new_status)
                .await?;
        } else {
            self.save_and_sync(&updated).await?;
        }

        Ok(updated)
    }

    pub async fn delete(&self, slug: &str) -> Result<()> {
        self.source.delete(slug).await?;
        if let Some(store) = &self.store {
            store.delete_spec(slug).await?;
        }
        Ok(())
    }

    pub async fn search(&self, query: &str) -> Result<SpecList> {
        let store = self.store.as_ref().context("Store not initialized")?;
        let rows = store.search_fts::<chisel_store::SpecRow>(query).await?;
        Ok(SpecList(rows))
    }

    pub async fn index_spec(&self, spec: &Spec) -> Result<()> {
        if let Some(store) = &self.store {
            store
                .update_spec(chisel_store::UpdateSpecParams {
                    slug: &spec.slug,
                    path: &spec.path.to_string_lossy(),
                    title: &spec.title,
                    status: &spec.status.to_string(),
                    area: spec.area.as_deref(),
                    content: &spec.content,
                    created: spec.created,
                    updated: spec.updated,
                })
                .await?;
        }
        Ok(())
    }

    pub async fn index_all(&self) -> Result<()> {
        let specs = self.source.list(None).await?;
        for spec in specs {
            let _ = self.index_spec(&spec).await;
        }
        Ok(())
    }

    pub async fn init(&self) -> Result<()> {
        // Create subdirectories
        let root = self.source.root();
        fs::create_dir_all(root.join("active"))?;
        fs::create_dir_all(root.join("shipped"))?;
        fs::create_dir_all(root.join("archived"))?;

        let existing = self.source.list(None).await?;
        if existing.is_empty() {
            self.create(
                "Example Feature Spec",
                Some("onboarding".to_string()),
                Some("feature"),
                None,
            )
            .await?;
        }

        self.index_all().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_status_directory() {
        assert_eq!(SpecStatus::Draft.directory(), "active");
        assert_eq!(SpecStatus::Ready.directory(), "active");
        assert_eq!(SpecStatus::InProgress.directory(), "active");
        assert_eq!(SpecStatus::Shipped.directory(), "shipped");
        assert_eq!(SpecStatus::Archived.directory(), "archived");
    }

    #[test]
    fn test_spec_sorting() {
        let mut specs = vec![
            chisel_store::SpecRow {
                slug: "a".to_string(),
                path: "".to_string(),
                title: "A".to_string(),
                status: "shipped".to_string(),
                area: None,
                created: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                updated: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                excerpt: "".to_string(),
            },
            chisel_store::SpecRow {
                slug: "b".to_string(),
                path: "".to_string(),
                title: "B".to_string(),
                status: "draft".to_string(),
                area: None,
                created: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                updated: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                excerpt: "".to_string(),
            },
            chisel_store::SpecRow {
                slug: "c".to_string(),
                path: "".to_string(),
                title: "C".to_string(),
                status: "draft".to_string(),
                area: None,
                created: NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(),
                updated: NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(),
                excerpt: "".to_string(),
            },
        ];

        SpecList::sort_specs(&mut specs);

        // Draft comes before shipped; within draft, newer first
        assert_eq!(specs[0].slug, "c");
        assert_eq!(specs[1].slug, "b");
        assert_eq!(specs[2].slug, "a");
    }

    #[tokio::test]
    async fn test_specs_service_flow() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();

        let service = SpecsService::new(root.clone()).await.unwrap();
        service.init().await.unwrap();

        let list = service.list(None).await.unwrap();
        assert_eq!(list.0.len(), 1); // Seed spec

        // Create
        let spec = service
            .create("User Auth", Some("auth".to_string()), Some("feature"), None)
            .await
            .unwrap();
        assert_eq!(spec.slug, "user-auth");
        assert_eq!(spec.status, SpecStatus::Draft);
        assert!(spec.path.to_string_lossy().contains("active"));

        // List
        let list = service.list(None).await.unwrap();
        assert_eq!(list.0.len(), 2);

        // Update status to in-progress (stays in active/)
        let updated = service
            .update_status("user-auth", SpecStatus::InProgress)
            .await
            .unwrap();
        assert_eq!(updated.status, SpecStatus::InProgress);
        assert!(updated.path.to_string_lossy().contains("active"));

        // Update status to shipped (moves to shipped/)
        let shipped = service
            .update_status("user-auth", SpecStatus::Shipped)
            .await
            .unwrap();
        assert_eq!(shipped.status, SpecStatus::Shipped);
        assert!(shipped.path.to_string_lossy().contains("shipped"));

        // Old path should not exist
        assert!(!root.join(".chisel/specs/active/user-auth.md").exists());
        assert!(root.join(".chisel/specs/shipped/user-auth.md").exists());

        // Delete
        service.delete("user-auth").await.unwrap();
        assert!(service.show("user-auth").await.is_err());
    }

    #[tokio::test]
    async fn test_em_dash_in_title_and_content() {
        let temp = tempfile::tempdir().unwrap();
        let service = SpecsService::new(temp.path().to_path_buf()).await.unwrap();

        // Em dashes straddle the truncation cutoffs (title byte 25, excerpt
        // byte 97) that the old byte-index slicing panicked on.
        let title = format!("{}\u{2014} and more title text", "x".repeat(24));
        let content = format!("{}\u{2014}{}", "y".repeat(96), " body tail".repeat(3));

        let spec = service
            .create(&title, None, None, Some(&content))
            .await
            .unwrap();
        assert_eq!(spec.title, title);

        // Excerpt building (crashes both machine and human modes on old code)
        let list = service.list(None).await.unwrap();
        assert_eq!(list.0.len(), 1);
        assert!(list.0[0].excerpt.ends_with("..."));

        // Human table rendering (title truncation path)
        list.render_human().unwrap();
    }
}
