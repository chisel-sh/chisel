use crate::{Spec, SpecStatus};
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait SpecSource: Send + Sync {
    /// List all discoverable specs, optionally filtered by status
    async fn list(&self, status: Option<SpecStatus>) -> Result<Vec<Spec>>;

    /// Load a spec by slug
    async fn load(&self, slug: &str) -> Result<Spec>;

    /// Save a spec to the source
    async fn save(&self, spec: &Spec) -> Result<()>;

    /// Delete a spec by slug
    async fn delete(&self, slug: &str) -> Result<()>;

    /// Move a spec file to the correct directory for its new status.
    /// Updates the spec's path and status fields, rewrites frontmatter.
    async fn move_to_status_dir(&self, spec: &mut Spec, new_status: SpecStatus) -> Result<()>;

    /// Resolve the file path for a spec given slug and status
    fn resolve_path(&self, slug: &str, status: &SpecStatus) -> PathBuf;

    /// Get the specs root directory
    fn root(&self) -> PathBuf;
}
