use async_trait::async_trait;
use anyhow::Result;
use std::path::PathBuf;
use crate::{Issue, IssueStatus};

#[async_trait]
pub trait IssueSource: Send + Sync {
    /// List all discoverable issues in the source
    async fn list(&self, status: Option<IssueStatus>) -> Result<Vec<Issue>>;

    /// Load a specific issue by ID or path
    async fn load(&self, id: i64) -> Result<Issue>;

    /// Save an issue back to the source
    async fn save(&self, issue: &Issue) -> Result<()>;

    /// Delete an issue
    async fn delete(&self, id: i64) -> Result<()>;

    /// Get the next available ID
    fn next_id(&self) -> Result<i64>;

    /// Resolve target path for a new issue
    fn resolve_path(&self, id: i64, title: &str) -> PathBuf;
}
