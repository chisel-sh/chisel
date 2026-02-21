use anyhow::{Context, Result};
use chisel_fs::spawn_editor;
use chisel_render::Renderable;
use chisel_store::{IssueRow, Store};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

pub mod default_source;
pub mod parsing;
pub mod source;

use default_source::DefaultIssueSource;
use source::IssueSource;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Issue {
    pub id: i64,
    pub path: PathBuf,
    pub title: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub order: i32,
    pub content: String,
    pub external_id: Option<String>,
}

impl chisel_render::MachineOutput for Issue {}

#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Display, EnumString,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum IssueStatus {
    Todo,
    InProgress,
    Done,
    Closed,
    Cancelled,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Display, EnumString,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum IssuePriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueFrontmatter {
    pub title: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub order: i32,
    pub external_id: Option<String>,
}

impl Renderable for Issue {
    fn render_human(&self) -> Result<()> {
        println!("#{} {}", self.id, self.title);
        println!("Status:   {}", self.status);
        println!("Priority: {}", self.priority);
        if !self.labels.is_empty() {
            println!("Labels:   {}", self.labels.join(", "));
        }
        println!("\n---\n\n{}", self.content);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssueList(pub Vec<IssueRow>);

impl chisel_render::MachineOutput for IssueList {
    fn to_machine_string(&self) -> Result<String> {
        let mut sorted = self.0.clone();
        IssueList::sort_issues(&mut sorted);
        Ok(serde_yaml::to_string(&sorted)?)
    }
}

impl IssueList {
    pub fn sort_issues(issues: &mut [IssueRow]) {
        issues.sort_by(|a, b| {
            let a_status = IssueStatus::from_str(&a.status).unwrap_or(IssueStatus::Todo);
            let b_status = IssueStatus::from_str(&b.status).unwrap_or(IssueStatus::Todo);

            a_status
                .cmp(&b_status)
                .then_with(|| {
                    let a_prio =
                        IssuePriority::from_str(&a.priority).unwrap_or(IssuePriority::Medium);
                    let b_prio =
                        IssuePriority::from_str(&b.priority).unwrap_or(IssuePriority::Medium);
                    b_prio.cmp(&a_prio) // Higher priority first
                })
                .then_with(|| b.id.cmp(&a.id))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_sorting() {
        let mut issues = vec![
            IssueRow {
                id: 1,
                status: "done".to_string(),
                priority: "low".to_string(),
                ..mock_issue_row()
            },
            IssueRow {
                id: 2,
                status: "todo".to_string(),
                priority: "critical".to_string(),
                ..mock_issue_row()
            },
            IssueRow {
                id: 3,
                status: "todo".to_string(),
                priority: "low".to_string(),
                ..mock_issue_row()
            },
        ];

        IssueList::sort_issues(&mut issues);

        assert_eq!(issues[0].id, 2); // todo, critical
        assert_eq!(issues[1].id, 3); // todo, low
        assert_eq!(issues[2].id, 1); // done, low
    }

    #[tokio::test]
    async fn test_issues_service_flow() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();

        let service = IssuesService::new(root.clone()).await.unwrap();
        service.init().await.unwrap();

        let list = service.list(None).await.unwrap();
        assert_eq!(list.0.len(), 2);

        // Create
        let issue = service
            .create("Bug", IssuePriority::High, vec![], "Fix it")
            .await
            .unwrap();
        assert_eq!(issue.id, 3);

        // Update Status
        service
            .update_status(3, IssueStatus::InProgress)
            .await
            .unwrap();
        let updated = service.show(3).await.unwrap();
        assert_eq!(updated.status, IssueStatus::InProgress);

        // Delete
        service.delete(3).await.unwrap();
        assert!(service.show(3).await.is_err());
    }

    fn mock_issue_row() -> IssueRow {
        IssueRow {
            id: 0,
            path: "".to_string(),
            title: "".to_string(),
            status: "".to_string(),
            priority: "".to_string(),
            labels: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            excerpt: "".to_string(),
        }
    }
}

impl Renderable for IssueList {
    fn render_human(&self) -> Result<()> {
        if self.0.is_empty() {
            println!("No issues found.");
            return Ok(());
        }

        let mut sorted = self.0.clone();
        IssueList::sort_issues(&mut sorted);

        println!(
            "{:<6} {:<35} {:<12} {:<10}",
            "ID", "TITLE", "STATUS", "PRIORITY"
        );
        println!("{}", "-".repeat(68));
        for issue in sorted {
            println!(
                "{:<6} {:<35} {:<12} {:<10}",
                format!("#{}", issue.id),
                if issue.title.len() > 33 {
                    format!("{}...", &issue.title[..30])
                } else {
                    issue.title.clone()
                },
                issue.status,
                issue.priority,
            );
        }
        Ok(())
    }
}

pub struct IssuesService {
    pub store: Option<Store>,
    pub root: PathBuf,
    pub source: Box<dyn IssueSource>,
}

impl IssuesService {
    pub async fn new(root: PathBuf) -> Result<Self> {
        let store = Store::new(root.clone()).await?;
        let source = Box::new(DefaultIssueSource::new(root.clone()));
        Ok(Self {
            store: Some(store),
            root,
            source,
        })
    }

    pub async fn list(&self, status: Option<IssueStatus>) -> Result<IssueList> {
        let issues = self.source.list(status).await?;
        // For performance, we might want to use the store if available
        // but for trait purity we use source for now.
        // Actually, store is useful for fast queries in TUI.
        let rows = issues
            .into_iter()
            .map(|i| IssueRow {
                id: i.id,
                path: i.path.to_string_lossy().to_string(),
                title: i.title,
                status: i.status.to_string(),
                priority: i.priority.to_string(),
                labels: if i.labels.is_empty() {
                    None
                } else {
                    Some(i.labels.join(","))
                },
                order: i.order,
                excerpt: if i.content.len() > 100 {
                    format!("{}...", &i.content[..97])
                } else {
                    i.content.clone()
                },
                created_at: i.created_at,
                updated_at: i.updated_at,
            })
            .collect();

        let mut list = IssueList(rows);
        IssueList::sort_issues(&mut list.0);
        Ok(list)
    }

    pub async fn show(&self, id: i64) -> Result<Issue> {
        self.source.load(id).await
    }

    pub async fn create(
        &self,
        title: &str,
        priority: IssuePriority,
        labels: Vec<String>,
        content: &str,
    ) -> Result<Issue> {
        let id = self.source.next_id()?;
        let path = self.source.resolve_path(id, title);

        let issue = Issue {
            id,
            path,
            title: title.to_string(),
            status: IssueStatus::Todo,
            priority,
            labels,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            order: 0,
            content: content.to_string(),
            external_id: None,
        };

        self.save_and_sync(&issue).await?;

        Ok(issue)
    }

    async fn save_and_sync(&self, issue: &Issue) -> Result<()> {
        self.source.save(issue).await?;
        self.index_issue(issue).await
    }

    pub async fn edit(&self, id: i64) -> Result<Issue> {
        let mut issue = self.source.load(id).await?;

        let mut temp_path = std::env::temp_dir();
        temp_path.push(format!("chisel_issue_{}.md", id));
        fs::write(&temp_path, &issue.content)?;

        spawn_editor(&temp_path)?;

        let new_content = fs::read_to_string(&temp_path)?;
        issue.content = new_content.trim().to_string();
        let _ = fs::remove_file(&temp_path);

        self.save_and_sync(&issue).await?;
        Ok(issue)
    }

    pub async fn update_title(&self, id: i64, title: String) -> Result<Issue> {
        let mut issue = self.source.load(id).await?;
        issue.title = title;
        self.save_and_sync(&issue).await?;
        Ok(issue)
    }

    pub async fn update_priority(&self, id: i64, priority: IssuePriority) -> Result<Issue> {
        let mut issue = self.source.load(id).await?;
        issue.priority = priority;
        self.save_and_sync(&issue).await?;
        Ok(issue)
    }

    pub async fn update_status(&self, id: i64, status: IssueStatus) -> Result<Issue> {
        let mut issue = self.source.load(id).await?;
        issue.status = status;
        self.save_and_sync(&issue).await?;
        Ok(issue)
    }

    pub async fn update_labels(&self, id: i64, labels: Vec<String>) -> Result<Issue> {
        let mut issue = self.source.load(id).await?;
        issue.labels = labels;
        self.save_and_sync(&issue).await?;
        Ok(issue)
    }

    pub async fn update_order(&self, id: i64, order: i32) -> Result<Issue> {
        let mut issue = self.source.load(id).await?;
        issue.order = order;
        self.save_and_sync(&issue).await?;
        Ok(issue)
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        self.source.delete(id).await?;
        if let Some(store) = &self.store {
            store.delete_issue(id).await?;
        }
        Ok(())
    }

    pub async fn index_issue(&self, issue: &Issue) -> Result<()> {
        if let Some(store) = &self.store {
            let labels = if issue.labels.is_empty() {
                None
            } else {
                Some(issue.labels.join(","))
            };

            store
                .update_issue(chisel_store::UpdateIssueParams {
                    id: issue.id,
                    path: &issue.path.to_string_lossy(),
                    title: &issue.title,
                    status: &issue.status.to_string(),
                    priority: &issue.priority.to_string(),
                    labels: labels.as_deref(),
                    content: &issue.content,
                    order: issue.order,
                    created_at: issue.created_at,
                })
                .await?;
        }
        Ok(())
    }

    pub async fn index_all(&self) -> Result<()> {
        let issues = self.source.list(None).await?;
        for issue in issues {
            let _ = self.index_issue(&issue).await;
        }
        Ok(())
    }

    pub async fn search(&self, query: &str) -> Result<IssueList> {
        let store = self.store.as_ref().context("Store not initialized")?;
        let rows = store.search_fts::<chisel_store::IssueRow>(query).await?;
        Ok(IssueList(rows))
    }

    pub async fn init(&self) -> Result<()> {
        let existing = self.source.list(None).await?;
        if existing.is_empty() {
            // Initial setup for DefaultSource
            self.create(
                "Try moving this issue",
                IssuePriority::High,
                vec!["tutorial".to_string()],
                "Select this issue in the `Todo` lane and press `m`. Move it to `In Progress` to see the board update."
            ).await?;

            self.create(
                "Delete onboarding content",
                IssuePriority::Low,
                vec!["cleanup".to_string()],
                "Once you are comfortable with Chisel, you can delete these seed files. \n\n1. Press `x` on an issue to delete it.\n2. Delete doc files manually or via future TUI actions.\n\nYour workspace is your own!"
            ).await?;
        }

        self.index_all().await?;
        Ok(())
    }
}
