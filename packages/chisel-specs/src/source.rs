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

    /// Change a spec's status, rewriting its frontmatter in place.
    /// Updates the spec's status and updated fields.
    async fn set_status(&self, spec: &mut Spec, new_status: SpecStatus) -> Result<()>;

    /// Resolve the file path for a spec slug
    fn resolve_path(&self, slug: &str) -> PathBuf;

    /// Get the specs root directory
    fn root(&self) -> PathBuf;
}
