mod tui;
mod upgrade;

use anyhow::Result;
use chisel_render::OutputMode;
use chisel_docs::{ListOptions, DocsService};
use chisel_issues::{IssuesService, IssuePriority, IssueStatus};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;
use tui::{DocsApp, IssuesApp};
use dialoguer::{Input, Select};
use upgrade::UpgradeService;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output in machine-readable format (YAML)
    #[arg(short, long, global = true)]
    machine: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Chisel workspace
    Init {
        /// The project name (defaults to current directory name)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Manage documentation
    Docs {
        #[command(subcommand)]
        command: Option<DocsCommands>,
    },
    /// Manage issues
    Issues {
        #[command(subcommand)]
        command: Option<IssuesCommands>,
    },
    /// Upgrade Chisel to the latest version
    Upgrade,
}

#[derive(Subcommand)]
enum DocsCommands {
    /// Show workspace overview
    Overview,
    /// List available documents
    List {
        /// The root directory to search in
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Include ignored and hidden files
        #[arg(short, long)]
        all: bool,

        /// Disable .gitignore processing
        #[arg(long)]
        no_ignore: bool,

        /// Include hidden files
        #[arg(long)]
        hidden: bool,
    },
    /// Show document content
    Show {
        /// The path to the document (optional, launches explorer if missing)
        path: Option<PathBuf>,
    },
    /// Index documents for search
    Index {
        /// The root directory to index
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Search documents
    Search {
        /// The search query
        query: String,
    },
    /// Fuzzy search documents
    FuzzySearch {
        /// The search query
        query: String,
    },
    /// Reorder a document
    Reorder {
        /// The path to the document
        path: PathBuf,
        /// The new order (integer)
        order: i32,
    },
    /// Reorder a category
    ReorderCategory {
        /// The category name
        category: String,
        /// The new order (integer)
        order: i32,
    },
    /// Create a new document
    New {
        /// The title of the document (optional, prompts if missing)
        title: Option<String>,

        /// The category (folder) for the document
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Edit a document
    Edit {
        /// The path to the document (optional, launches explorer if missing)
        path: Option<PathBuf>,
    },
    /// Move a document to a new category
    Move {
        /// The path to the document
        path: Option<PathBuf>,
        /// The new category (folder)
        category: Option<String>,
    },
    /// Delete a document
    Delete {
        /// The path to the document
        path: PathBuf,
    }
}

#[derive(Subcommand)]
enum IssuesCommands {
    /// Show issues overview
    Overview,
    /// List available issues
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show issue details
    Show {
        /// The issue ID
        id: i64,
    },
    /// Create a new issue
    New {
        /// The title of the issue
        #[arg(short, long)]
        title: Option<String>,
        
        /// The priority (low, medium, high, critical)
        #[arg(short, long)]
        priority: Option<String>,

        /// The labels (comma-separated)
        #[arg(short, long)]
        labels: Option<String>,
    },
    /// Edit an issue
    Edit {
        /// The issue ID
        id: i64,
    },
    /// Close an issue
    Close {
        /// The issue ID
        id: i64,
    },
    /// Delete an issue
    Delete {
        /// The issue ID
        id: i64,
    },
    /// Reorder an issue
    Reorder {
        /// The issue ID
        id: i64,
        /// The new order (integer)
        order: i32,
    },
    /// Update issue priority
    UpdatePriority {
        /// The issue ID
        id: i64,
        /// The new priority
        priority: String,
    },
    /// Update issue title
    UpdateTitle {
        /// The issue ID
        id: i64,
        /// The new title
        title: String,
    },
    /// Update issue status
    UpdateStatus {
        /// The issue ID
        id: i64,
        /// The new status
        status: String,
    },
    /// Update issue labels
    UpdateLabels {
        /// The issue ID
        id: i64,
        /// The new labels (comma-separated)
        labels: String,
    },
    /// Index all issues
    Index,
    /// Search issues
    Search {
        /// The search query
        query: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mode = if cli.machine {
        OutputMode::Machine
    } else {
        OutputMode::Human
    };

    let root = std::env::current_dir()?;
    let upgrade_service = UpgradeService::new(root.clone());

    // Background update check (only in human mode)
    if let OutputMode::Human = mode {
        if let Ok(Some(latest)) = upgrade_service.check_for_updates().await {
            println!("💡 A new version of Chisel is available: v{} (current: v{})", latest, env!("CARGO_PKG_VERSION"));
            println!("👉 Run `chisel upgrade` to update.\n");
        }
    }

    match cli.command {
        Commands::Init { name } => {
            init_workspace(&root, name, mode).await?;
        }
        Commands::Upgrade => {
            upgrade_service.perform_update()?;
        }
        Commands::Docs { command } => {
            let service = DocsService::new(root).await?;
            match command {
                Some(DocsCommands::Overview) => {
                    mode.render(service.overview().await?)?;
                }
                Some(DocsCommands::List { path, all, no_ignore, hidden }) => {
                    match mode {
                        OutputMode::Machine => {
                            let options = ListOptions {
                                root: path,
                                use_gitignore: !all && !no_ignore,
                                include_hidden: all || hidden,
                            };
                            mode.render(service.list(options).await?)?;
                        }
                        OutputMode::Human => {
                            run_docs_explorer(service, mode, None).await?;
                        }
                    }
                }
                Some(DocsCommands::Show { path }) => {
                    match mode {
                        OutputMode::Machine => {
                            if let Some(p) = path {
                                mode.render(service.show(p).await?)?;
                            } else {
                                mode.render(service.overview().await?)?;
                            }
                        }
                        OutputMode::Human => {
                            run_docs_explorer(service, mode, path).await?;
                        }
                    }
                }
                Some(DocsCommands::Index { .. }) => {
                    service.index_all().await?;
                    if let OutputMode::Human = mode {
                        println!("Indexing complete.");
                    }
                }
                Some(DocsCommands::Search { query }) => {
                    match mode {
                        OutputMode::Machine => {
                            mode.render(service.search(&query).await?)?;
                        }
                        OutputMode::Human => {
                            run_docs_explorer(service, mode, None).await?;
                        }
                    }
                }
                Some(DocsCommands::FuzzySearch { query }) => {
                    mode.render(service.fuzzy_search(&query).await?)?;
                }
                Some(DocsCommands::Reorder { path, order }) => {
                    let doc = service.update_doc_order(path, order).await?;
                    mode.render(doc)?;
                }
                Some(DocsCommands::ReorderCategory { category, order }) => {
                    service.update_category_order(&category, order)?;
                    if let OutputMode::Human = mode {
                        println!("Updated order for category: {}", category);
                    }
                }
                Some(DocsCommands::New { title, category }) => {
                    let title = match title {
                        Some(t) => t,
                        None => {
                            if let OutputMode::Machine = mode {
                                anyhow::bail!("Title is required in machine mode");
                            }
                            Input::<String>::new()
                                .with_prompt("Document Title")
                                .interact_text()?
                        }
                    };
                    let doc = service.create(&title, category).await?;
                    match mode {
                        OutputMode::Machine => {
                            mode.render(doc)?;
                        }
                        OutputMode::Human => {
                            run_docs_explorer(service, mode, Some(doc.path)).await?;
                        }
                    }
                }
                Some(DocsCommands::Edit { path }) => {
                    match mode {
                        OutputMode::Machine => {
                            if let Some(p) = path {
                                mode.render(service.edit(p).await?)?;
                            } else {
                                anyhow::bail!("Path is required in machine mode");
                            }
                        }
                        OutputMode::Human => {
                            run_docs_explorer(service, mode, path).await?;
                        }
                    }
                }
                Some(DocsCommands::Move { path, category }) => {
                    match mode {
                        OutputMode::Machine => {
                            if let Some(p) = path {
                                mode.render(service.move_doc(p, category).await?)?;
                            } else {
                                anyhow::bail!("Path is required in machine mode");
                            }
                        }
                        OutputMode::Human => {
                            run_docs_explorer(service, mode, path).await?;
                        }
                    }
                }
                Some(DocsCommands::Delete { path }) => {
                    service.delete(path.clone()).await?;
                    if let OutputMode::Human = mode {
                        println!("Deleted document: {}", path.display());
                    }
                }
                None => {
                    run_docs_explorer(service, mode, None).await?;
                }
            }
        }
        Commands::Issues { command } => {
            let service = IssuesService::new(root).await?;
            match command {
                Some(IssuesCommands::Overview) => {
                    mode.render(service.list(None).await?)?;
                }
                Some(IssuesCommands::List { status }) => {
                    let status_enum = status.and_then(|s| IssueStatus::from_str(&s.to_lowercase()).ok());
                    match mode {
                        OutputMode::Machine => {
                            mode.render(service.list(status_enum).await?)?;
                        }
                        OutputMode::Human => {
                            run_issues_explorer(service, mode, status_enum, None).await?;
                        }
                    }
                }
                Some(IssuesCommands::Show { id }) => {
                    match mode {
                        OutputMode::Machine => {
                            mode.render(service.show(id).await?)?;
                        }
                        OutputMode::Human => {
                            run_issues_explorer(service, mode, None, Some(id)).await?;
                        }
                    }
                }
                Some(IssuesCommands::New { title, priority, labels }) => {
                    let title = match title {
                        Some(t) => t,
                        None => {
                            if let OutputMode::Machine = mode {
                                anyhow::bail!("Title is required in machine mode");
                            }
                            Input::<String>::new().with_prompt("Issue Title").interact_text()?
                        }
                    };
                    let priority = parse_priority(priority, mode)?;
                    
                    let label_vec = labels.map(|l| {
                        l.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    }).unwrap_or_default();

                    let content = if let OutputMode::Human = mode {
                        "Enter description here..."
                    } else {
                        ""
                    };

                    let issue = service.create(&title, priority, label_vec, content).await?;
                    match mode {
                        OutputMode::Machine => {
                            mode.render(issue)?;
                        }
                        OutputMode::Human => {
                            run_issues_explorer(service, mode, None, Some(issue.id)).await?;
                        }
                    }
                }
                Some(IssuesCommands::Edit { id }) => {
                    match mode {
                        OutputMode::Machine => {
                            mode.render(service.edit(id).await?)?;
                        }
                        OutputMode::Human => {
                            run_issues_explorer(service, mode, None, Some(id)).await?;
                        }
                    }
                }
                Some(IssuesCommands::Close { id }) => {
                    let issue = service.update_status(id, IssueStatus::Closed).await?;
                    mode.render(issue)?;
                }
                Some(IssuesCommands::Delete { id }) => {
                    service.delete(id).await?;
                    if let OutputMode::Human = mode {
                        println!("Deleted issue #{}", id);
                    }
                }
                Some(IssuesCommands::Reorder { id, order }) => {
                    let issue = service.update_order(id, order).await?;
                    mode.render(issue)?;
                }
                Some(IssuesCommands::UpdatePriority { id, priority }) => {
                    let p_enum = IssuePriority::from_str(&priority.to_lowercase())
                        .map_err(|_| anyhow::anyhow!("Invalid priority: {}", priority))?;
                    let issue = service.update_priority(id, p_enum).await?;
                    mode.render(issue)?;
                }
                Some(IssuesCommands::UpdateTitle { id, title }) => {
                    let issue = service.update_title(id, title).await?;
                    mode.render(issue)?;
                }
                Some(IssuesCommands::UpdateStatus { id, status }) => {
                    let s_enum = IssueStatus::from_str(&status.to_lowercase())
                        .map_err(|_| anyhow::anyhow!("Invalid status: {}", status))?;
                    let issue = service.update_status(id, s_enum).await?;
                    mode.render(issue)?;
                }
                Some(IssuesCommands::UpdateLabels { id, labels }) => {
                    let label_vec = labels.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let issue = service.update_labels(id, label_vec).await?;
                    mode.render(issue)?;
                }
                Some(IssuesCommands::Index) => {
                    service.index_all().await?;
                    if let OutputMode::Human = mode {
                        println!("Indexing complete.");
                    }
                }
                Some(IssuesCommands::Search { query }) => {
                    mode.render(service.search(&query).await?)?;
                }
                None => {
                    run_issues_explorer(service, mode, None, None).await?;
                }
            }
        }
    }

    Ok(())
}

async fn init_workspace(root: &std::path::Path, name: Option<String>, mode: OutputMode) -> Result<()> {
    let project_name = name.unwrap_or_else(|| {
        root.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "My Project".to_string())
    });

    if let OutputMode::Human = mode {
        println!("Initializing Chisel workspace for: {}...", project_name);
    }

    // 1. Create .chisel directory
    let chisel_dir = root.join(".chisel");
    let docs_dir = chisel_dir.join("docs");
    let issues_dir = chisel_dir.join("issues");
    std::fs::create_dir_all(&docs_dir)?;
    std::fs::create_dir_all(&issues_dir)?;

    // 2. Initialize Services
    let docs_service = DocsService::new(root.to_path_buf()).await?;
    let issues_service = IssuesService::new(root.to_path_buf()).await?;

    // 3. Delegate seed generation to services
    docs_service.init(&project_name).await?;
    issues_service.init().await?;

    // 4. Generate AI Agent Prompt
    let prompt_path = chisel_dir.join("PROMPT.md");
    let prompt_content = format!(
        "# Chisel Project: {}\n\nThis project uses Chisel for its documentation and issue tracking.\n\n## Structure\n- Docs: `.chisel/docs/` (Markdown)\n- Issues: `.chisel/issues/` (Markdown with YAML frontmatter)\n\n## Guidelines\nWhen performing tasks in this repo, you can use `chisel docs` and `chisel issues` with the `--machine` flag to inspect and update the project state efficiently.",
        project_name
    );
    std::fs::write(&prompt_path, prompt_content)?;

    // 5. Update .gitignore
    let gitignore_path = root.join(".gitignore");
    let gitignore_entry = "\n# Chisel Cache\n.chisel/index.db\n";
    if gitignore_path.exists() {
        let mut content = std::fs::read_to_string(&gitignore_path)?;
        if !content.contains(".chisel/index.db") {
            content.push_str(gitignore_entry);
            std::fs::write(&gitignore_path, content)?;
        }
    } else {
        std::fs::write(&gitignore_path, gitignore_entry)?;
    }

    if let OutputMode::Human = mode {
        println!("Success! Chisel is ready at .chisel/");
        println!("Try running `chisel docs` or `chisel issues` to begin.");
    }
    Ok(())
}

async fn run_docs_explorer(service: DocsService, mode: OutputMode, initial_path: Option<PathBuf>) -> Result<()> {
    if !service.workspace_root.join(".chisel").exists() {
        if let OutputMode::Human = mode {
            println!("This directory is not a Chisel workspace.");
            println!("Run `chisel init` to get started.");
            return Ok(());
        }
    }

    match mode {
        OutputMode::Machine => {
            mode.render(service.overview().await?)?;
        }
        OutputMode::Human => {
            let mut app = DocsApp::new(service, initial_path).await?;
            let _ = app.run().await?;
        }
    }
    Ok(())
}

async fn run_issues_explorer(service: IssuesService, mode: OutputMode, status_filter: Option<IssueStatus>, initial_id: Option<i64>) -> Result<()> {
    if !service.root.join(".chisel").exists() {
        if let OutputMode::Human = mode {
            println!("This directory is not a Chisel workspace.");
            println!("Run `chisel init` to get started.");
            return Ok(());
        }
    }

    match mode {
        OutputMode::Machine => {
            mode.render(service.list(status_filter).await?)?;
        }
        OutputMode::Human => {
            let mut app = IssuesApp::new(service, status_filter, initial_id).await?;
            let _ = app.run().await?;
        }
    }
    Ok(())
}

fn parse_priority(priority: Option<String>, mode: OutputMode) -> Result<IssuePriority> {
    match priority {
        Some(p) => {
            IssuePriority::from_str(&p.to_lowercase())
                .or_else(|_| {
                    if p.to_lowercase() == "med" {
                        Ok(IssuePriority::Medium)
                    } else {
                        Err(anyhow::anyhow!("Invalid priority: {}", p))
                    }
                })
        }
        None => {
            if let OutputMode::Machine = mode {
                Ok(IssuePriority::Medium)
            } else {
                let options = vec!["Low", "Medium", "High", "Critical"];
                let selection = Select::new()
                    .with_prompt("Priority")
                    .items(&options)
                    .default(1)
                    .interact()?;
                Ok(match selection {
                    0 => IssuePriority::Low,
                    1 => IssuePriority::Medium,
                    2 => IssuePriority::High,
                    3 => IssuePriority::Critical,
                    _ => IssuePriority::Medium,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_init_workspace() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        
        init_workspace(root, Some("Test Project".to_string()), OutputMode::Machine).await.unwrap();
        
        assert!(root.join(".chisel").exists());
        assert!(root.join(".chisel/docs/welcome-to-chisel.md").exists());
        assert!(root.join(".chisel/docs/tutorial/working-with-docs.md").exists());
        assert!(root.join(".chisel/issues").exists());
        assert!(root.join(".chisel/PROMPT.md").exists());
        assert!(root.join(".gitignore").exists());
        
        let gitignore = std::fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(gitignore.contains(".chisel/index.db"));
    }
}
