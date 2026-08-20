use anyhow::{Context, Result};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

pub struct Store {
    pool: SqlitePool,
}

pub struct UpdateSpecParams<'a> {
    pub slug: &'a str,
    pub path: &'a str,
    pub title: &'a str,
    pub status: &'a str,
    pub area: Option<&'a str>,
    pub content: &'a str,
    pub created: chrono::NaiveDate,
    pub updated: chrono::NaiveDate,
}

pub trait Searchable: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin {
    fn search_sql() -> &'static str;
}

impl Searchable for SearchResult {
    fn search_sql() -> &'static str {
        "SELECT path, name, snippet(documents_fts, 3, '...', '...', '...', 10) as excerpt
         FROM documents_fts
         WHERE documents_fts MATCH ?
         ORDER BY rank"
    }
}

impl Searchable for SpecRow {
    fn search_sql() -> &'static str {
        "SELECT s.slug, s.path, s.title, s.status, s.area, s.created, s.updated,
                snippet(specs_fts, 1, '...', '...', '...', 10) as excerpt
         FROM specs s
         JOIN specs_fts f ON s.rowid = f.rowid
         WHERE specs_fts MATCH ?
         ORDER BY rank"
    }
}

impl Store {
    pub async fn new(root: PathBuf) -> Result<Self> {
        let db_path = root.join(".chisel").join("index.db");
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).context("Failed to create chisel data directory")?;
        }
        let url = format!("sqlite:{}", db_path.display());
        Self::new_with_url(&url).await
    }

    pub async fn new_with_url(url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(url)
            .map_err(|e| anyhow::anyhow!("Invalid database URL: {}", e))?
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options)
            .await
            .context("Failed to connect to SQLite")?;

        let store = Self { pool };
        store.migrate().await?;

        Ok(store)
    }

    async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("Failed to run database migrations")?;
        Ok(())
    }

    pub async fn update_doc(
        &self,
        path: &str,
        name: &str,
        title: Option<&str>,
        tags: Option<&str>,
        content: &str,
        created_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO documents (path, name, title, tags, content, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                title = excluded.title,
                tags = excluded.tags,
                content = excluded.content,
                created_at = COALESCE(documents.created_at, excluded.created_at),
                updated_at = excluded.updated_at",
        )
        .bind(path)
        .bind(name)
        .bind(title)
        .bind(tags)
        .bind(content)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_spec(&self, params: UpdateSpecParams<'_>) -> Result<()> {
        sqlx::query(
            "INSERT INTO specs (slug, path, title, status, area, content, created, updated)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(slug) DO UPDATE SET
                path = excluded.path,
                title = excluded.title,
                status = excluded.status,
                area = excluded.area,
                content = excluded.content,
                updated = excluded.updated",
        )
        .bind(params.slug)
        .bind(params.path)
        .bind(params.title)
        .bind(params.status)
        .bind(params.area)
        .bind(params.content)
        .bind(params.created)
        .bind(params.updated)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_doc(&self, path: &str) -> Result<()> {
        sqlx::query("DELETE FROM documents WHERE path = ?")
            .bind(path)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_doc_paths(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT path FROM documents")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|(p,)| p).collect())
    }

    pub async fn delete_spec(&self, slug: &str) -> Result<()> {
        sqlx::query("DELETE FROM specs WHERE slug = ?")
            .bind(slug)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn search_fts<T: Searchable>(&self, query: &str) -> Result<Vec<T>> {
        let results = sqlx::query_as::<_, T>(T::search_sql())
            .bind(query)
            .fetch_all(&self.pool)
            .await?;
        Ok(results)
    }

    pub async fn fuzzy_search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let sql_query = format!(
            "%{}%",
            query
                .chars()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("%")
        );
        let results = sqlx::query_as::<_, SearchResult>(
            "SELECT path, name, SUBSTR(content, 1, 100) as excerpt
             FROM documents
             WHERE name LIKE ? OR title LIKE ? OR content LIKE ?
             ORDER BY updated_at DESC
             LIMIT 20",
        )
        .bind(&sql_query)
        .bind(&sql_query)
        .bind(&sql_query)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    pub async fn fetch_context(&self, query: &str) -> Result<Vec<ContextItem>> {
        let mut items = Vec::new();

        // 1. Fetch Docs
        let docs = sqlx::query_as::<_, (String, String)>(
            "SELECT path, content
             FROM documents_fts
             WHERE documents_fts MATCH ?
             ORDER BY rank
             LIMIT 10",
        )
        .bind(query)
        .fetch_all(&self.pool)
        .await?;

        for (path, content) in docs {
            items.push(ContextItem {
                path,
                content,
                r#type: "document".to_string(),
            });
        }

        // 2. Fetch Specs
        let specs = sqlx::query_as::<_, (String, String)>(
            "SELECT path, content
             FROM specs_fts
             WHERE specs_fts MATCH ?
             ORDER BY rank
             LIMIT 10",
        )
        .bind(query)
        .fetch_all(&self.pool)
        .await?;

        for (path, content) in specs {
            items.push(ContextItem {
                path,
                content,
                r#type: "spec".to_string(),
            });
        }

        Ok(items)
    }

    pub async fn get_all_results(&self) -> Result<Vec<SearchResult>> {
        let results = sqlx::query_as::<_, SearchResult>(
            "SELECT path, name, SUBSTR(content, 1, 50) as excerpt
             FROM documents
             ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    pub async fn get_specs(&self, status: Option<&str>) -> Result<Vec<SpecRow>> {
        let query = if let Some(s) = status {
            sqlx::query_as::<_, SpecRow>(
                "SELECT slug, path, title, status, area, created, updated, SUBSTR(content, 1, 100) as excerpt
                 FROM specs
                 WHERE status = ?
                 ORDER BY status, updated DESC"
            ).bind(s)
        } else {
            sqlx::query_as::<_, SpecRow>(
                "SELECT slug, path, title, status, area, created, updated, SUBSTR(content, 1, 100) as excerpt
                 FROM specs
                 ORDER BY status, updated DESC"
            )
        };

        let results = query.fetch_all(&self.pool).await?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_store(name: &str) -> Store {
        let url = format!("sqlite:file:{}?mode=memory&cache=shared", name);
        Store::new_with_url(&url)
            .await
            .expect("Failed to create in-memory store")
    }

    #[tokio::test]
    async fn test_doc_crud() {
        let store = test_store("doc_crud").await;

        // Create
        store
            .update_doc(
                "test.md",
                "test",
                Some("Test Title"),
                Some("tag1, tag2"),
                "Content here",
                None,
            )
            .await
            .unwrap();

        let results = store.get_all_results().await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test");

        // Update
        store
            .update_doc(
                "test.md",
                "test",
                Some("New Title"),
                None,
                "New Content",
                None,
            )
            .await
            .unwrap();
        let results = store.get_all_results().await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].excerpt, "New Content");
    }

    #[tokio::test]
    async fn test_spec_crud() {
        let store = test_store("spec_crud").await;
        let today = chrono::NaiveDate::from_ymd_opt(2026, 3, 28).unwrap();

        // Create
        store
            .update_spec(UpdateSpecParams {
                slug: "user-auth",
                path: ".chisel/specs/active/user-auth.md",
                title: "User Auth",
                status: "draft",
                area: Some("auth"),
                content: "Implement user authentication",
                created: today,
                updated: today,
            })
            .await
            .unwrap();

        let specs = store.get_specs(None).await.unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].title, "User Auth");

        // Update status
        store
            .update_spec(UpdateSpecParams {
                slug: "user-auth",
                path: ".chisel/specs/active/user-auth.md",
                title: "User Auth",
                status: "in-progress",
                area: Some("auth"),
                content: "Implement user authentication",
                created: today,
                updated: today,
            })
            .await
            .unwrap();
        let specs = store.get_specs(Some("in-progress")).await.unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].status, "in-progress");

        // Delete
        store.delete_spec("user-auth").await.unwrap();
        let specs = store.get_specs(None).await.unwrap();
        assert_eq!(specs.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_doc() {
        let store = test_store("delete_doc").await;

        store
            .update_doc("a.md", "a", Some("Alpha"), None, "alpha content", None)
            .await
            .unwrap();
        store
            .update_doc("b.md", "b", Some("Beta"), None, "beta content", None)
            .await
            .unwrap();

        // Fires the documents_ad FTS trigger, which was malformed before
        // migration 003 and made any DELETE on documents fail to prepare
        store.delete_doc("a.md").await.unwrap();

        assert_eq!(store.get_doc_paths().await.unwrap(), vec!["b.md"]);

        let results = store.search_fts::<SearchResult>("alpha").await.unwrap();
        assert!(results.is_empty());
        let results = store.search_fts::<SearchResult>("beta").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_search() {
        let store = test_store("search").await;

        store
            .update_doc(
                "a.md",
                "a",
                Some("Rust"),
                None,
                "Rust is a systems programming language.",
                None,
            )
            .await
            .unwrap();
        store
            .update_doc(
                "b.md",
                "b",
                Some("Go"),
                None,
                "Go is an open source programming language.",
                None,
            )
            .await
            .unwrap();

        let rust_results = store.search_fts::<SearchResult>("Rust").await.unwrap();
        assert_eq!(rust_results.len(), 1);
        assert_eq!(rust_results[0].name, "a");

        let prog_results = store
            .search_fts::<SearchResult>("programming")
            .await
            .unwrap();
        assert_eq!(prog_results.len(), 2);

        // Fuzzy search
        let fuzzy_results = store.fuzzy_search("rs").await.unwrap();
        assert_eq!(fuzzy_results.len(), 1);
        assert_eq!(fuzzy_results[0].name, "a");
    }

    #[tokio::test]
    async fn test_fetch_context() {
        let store = test_store("context").await;
        let today = chrono::NaiveDate::from_ymd_opt(2026, 3, 28).unwrap();

        // Seed Docs
        store
            .update_doc(
                "doc1.md",
                "doc1",
                Some("Context Doc"),
                None,
                "This is relevant context about routing.",
                None,
            )
            .await
            .unwrap();
        store
            .update_doc(
                "doc2.md",
                "doc2",
                Some("Other Doc"),
                None,
                "This is irrelevant.",
                None,
            )
            .await
            .unwrap();

        // Seed Specs
        store
            .update_spec(UpdateSpecParams {
                slug: "routing-fix",
                path: ".chisel/specs/active/routing-fix.md",
                title: "Routing Fix",
                status: "in-progress",
                area: Some("infra"),
                content: "We need to fix the routing bug.",
                created: today,
                updated: today,
            })
            .await
            .unwrap();

        // Search for "routing"
        let results = store.fetch_context("routing").await.unwrap();

        assert_eq!(results.len(), 2); // 1 doc + 1 spec

        let doc = results.iter().find(|i| i.r#type == "document").unwrap();
        assert_eq!(doc.path, "doc1.md");
        assert!(doc.content.contains("relevant context"));

        let spec = results.iter().find(|i| i.r#type == "spec").unwrap();
        assert_eq!(spec.path, ".chisel/specs/active/routing-fix.md");
        assert!(spec.content.contains("fix the routing bug"));
    }
}

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SpecRow {
    pub slug: String,
    pub path: String,
    pub title: String,
    pub status: String,
    pub area: Option<String>,
    pub created: chrono::NaiveDate,
    pub updated: chrono::NaiveDate,
    pub excerpt: String,
}

#[derive(sqlx::FromRow, serde::Serialize, Debug)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub excerpt: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ContextItem {
    pub path: String,
    pub content: String,
    pub r#type: String,
}
