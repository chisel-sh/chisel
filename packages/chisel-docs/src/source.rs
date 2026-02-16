use async_trait::async_trait;
use anyhow::Result;
use std::path::PathBuf;
use crate::Doc;

#[async_trait]
pub trait DataSource: Send + Sync {
    /// List all discoverable documents in the source
    async fn list(&self) -> Result<Vec<Doc>>;

    /// Load a specific document by path
    async fn load(&self, path: PathBuf) -> Result<Doc>;

    /// Save a document back to the source
    async fn save(&self, doc: &Doc) -> Result<()>;

    /// Delete a document
    async fn delete(&self, path: PathBuf) -> Result<()>;

    /// Resolve target path for a new document
    fn resolve_path(&self, category: Option<String>, slug: String) -> PathBuf;

    /// Get the root directory of the source
    fn root(&self) -> PathBuf;
}
