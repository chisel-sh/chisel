use crate::{
    parsing::load_issue_from_file, source::IssueSource, Issue, IssueFrontmatter, IssueStatus,
};
use anyhow::Result;
use async_trait::async_trait;
use chisel_fs::slugify_title;
use glob::glob;
use std::fs;
use std::path::PathBuf;

pub struct DefaultIssueSource {
    pub root: PathBuf,
}

impl DefaultIssueSource {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn issues_dir(&self) -> PathBuf {
        self.root.join(".chisel").join("issues")
    }
}

#[async_trait]
impl IssueSource for DefaultIssueSource {
    async fn list(&self, status: Option<IssueStatus>) -> Result<Vec<Issue>> {
        let dir = self.issues_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut issues = Vec::new();
        for path in glob(&format!("{}/*.md", dir.display()))?.flatten() {
            if let Ok(issue) = load_issue_from_file(path) {
                if let Some(s) = &status {
                    if issue.status == *s {
                        issues.push(issue);
                    }
                } else {
                    issues.push(issue);
                }
            }
        }
        Ok(issues)
    }

    async fn load(&self, id: i64) -> Result<Issue> {
        let dir = self.issues_dir();
        for path in glob(&format!("{}/*.md", dir.display()))?.flatten() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(id_str) = name.split('_').next() {
                    if let Ok(found_id) = id_str.parse::<i64>() {
                        if found_id == id {
                            return load_issue_from_file(path);
                        }
                    }
                }
            }
        }
        anyhow::bail!("Issue #{} not found", id)
    }

    async fn save(&self, issue: &Issue) -> Result<()> {
        let fm = IssueFrontmatter {
            title: issue.title.clone(),
            status: issue.status.clone(),
            priority: issue.priority.clone(),
            labels: issue.labels.clone(),
            created_at: issue.created_at,
            order: issue.order,
            external_id: issue.external_id.clone(),
        };

        let file_content = format!(
            "---
{}---

{}",
            serde_yaml::to_string(&fm)?,
            issue.content
        );

        if let Some(parent) = issue.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&issue.path, file_content)?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<()> {
        let issue = self.load(id).await?;
        if issue.path.exists() {
            fs::remove_file(issue.path)?;
        }
        Ok(())
    }

    fn next_id(&self) -> Result<i64> {
        let dir = self.issues_dir();
        if !dir.exists() {
            return Ok(1);
        }

        let mut max_id = 0;
        for path in glob(&format!("{}/*.md", dir.display()))?.flatten() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(id_str) = name.split('_').next() {
                    if let Ok(id) = id_str.parse::<i64>() {
                        if id > max_id {
                            max_id = id;
                        }
                    }
                }
            }
        }
        Ok(max_id + 1)
    }

    fn resolve_path(&self, id: i64, title: &str) -> PathBuf {
        let dir = self.issues_dir();
        let slug = slugify_title(title);
        let filename = format!("{:04}_{}.md", id, slug);
        dir.join(filename)
    }
}
