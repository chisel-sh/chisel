use anyhow::{Context, Result};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

pub struct Store {
    pool: SqlitePool,
}

pub struct UpdateIssueParams<'a> {
    pub id: i64,
    pub path: &'a str,
    pub title: &'a str,
    pub status: &'a str,
    pub priority: &'a str,
    pub labels: Option<&'a str>,
    pub content: &'a str,
    pub order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
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

impl Searchable for IssueRow {
    fn search_sql() -> &'static str {
        "SELECT i.id, i.path, i.title, i.status, i.priority, i.labels, i.created_at, i.updated_at, i.\"order\", 
                snippet(issues_fts, 1, '...', '...', '...', 10) as excerpt
         FROM issues i
         JOIN issues_fts f ON i.id = f.rowid
         WHERE issues_fts MATCH ?
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

    pub async fn update_issue(&self, params: UpdateIssueParams<'_>) -> Result<()> {
        sqlx::query(
            "INSERT INTO issues (id, path, title, status, priority, labels, content, \"order\", created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                path = excluded.path,
                title = excluded.title,
                status = excluded.status,
                priority = excluded.priority,
                labels = excluded.labels,
                content = excluded.content,
                \"order\" = excluded.\"order\",
                updated_at = excluded.updated_at",
        )
        .bind(params.id)
        .bind(params.path)
        .bind(params.title)
        .bind(params.status)
        .bind(params.priority)
        .bind(params.labels)
        .bind(params.content)
        .bind(params.order)
        .bind(params.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_issue(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM issues WHERE id = ?")
            .bind(id)
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

        // 2. Fetch Issues
        let issues = sqlx::query_as::<_, (String, String)>(
            "SELECT path, content
             FROM issues_fts
             WHERE issues_fts MATCH ?
             ORDER BY rank
             LIMIT 10",
        )
        .bind(query)
        .fetch_all(&self.pool)
        .await?;

        for (path, content) in issues {
            items.push(ContextItem {
                path,
                content,
                r#type: "issue".to_string(),
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

    pub async fn get_issues(&self, status: Option<&str>) -> Result<Vec<IssueRow>> {
        let query = if let Some(s) = status {
            sqlx::query_as::<_, IssueRow>(
                "SELECT id, path, title, status, priority, labels, created_at, updated_at, \"order\", SUBSTR(content, 1, 100) as excerpt
                 FROM issues
                 WHERE status = ?
                 ORDER BY \"order\" ASC, priority DESC, id DESC"
            ).bind(s)
        } else {
            sqlx::query_as::<_, IssueRow>(
                "SELECT id, path, title, status, priority, labels, created_at, updated_at, \"order\", SUBSTR(content, 1, 100) as excerpt
                 FROM issues
                 ORDER BY status, \"order\" ASC, priority DESC, id DESC"
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
    async fn test_issue_crud() {
        let store = test_store("issue_crud").await;
        let now = chrono::Utc::now();

        // Create
        store
            .update_issue(UpdateIssueParams {
                id: 1,
                path: "issues/0001.md",
                title: "Issue 1",
                status: "todo",
                priority: "high",
                labels: None,
                content: "Fix it",
                order: 0,
                created_at: now,
            })
            .await
            .unwrap();

        let issues = store.get_issues(None).await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].title, "Issue 1");

        // Update status
        store
            .update_issue(UpdateIssueParams {
                id: 1,
                path: "issues/0001.md",
                title: "Issue 1",
                status: "in-progress",
                priority: "high",
                labels: None,
                content: "Fix it",
                order: 0,
                created_at: now,
            })
            .await
            .unwrap();
        let issues = store.get_issues(Some("in-progress")).await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].status, "in-progress");

        // Delete
        store.delete_issue(1).await.unwrap();
        let issues = store.get_issues(None).await.unwrap();
        assert_eq!(issues.len(), 0);
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

        // Seed Issues
        store
            .update_issue(UpdateIssueParams {
                id: 1,
                path: "issue1.md",
                title: "Context Issue",
                status: "open",
                priority: "high",
                labels: None,
                content: "We need to fix the routing bug.",
                order: 0,
                created_at: chrono::Utc::now(),
            })
            .await
            .unwrap();

        // Search for "routing"
        let results = store.fetch_context("routing").await.unwrap();

        assert_eq!(results.len(), 2); // 1 doc + 1 issue

        let doc = results.iter().find(|i| i.r#type == "document").unwrap();
        assert_eq!(doc.path, "doc1.md");
        assert!(doc.content.contains("relevant context"));

        let issue = results.iter().find(|i| i.r#type == "issue").unwrap();
        assert_eq!(issue.path, "issue1.md");
        assert!(issue.content.contains("fix the routing bug"));
    }
}

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct IssueRow {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub labels: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub order: i32,
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
