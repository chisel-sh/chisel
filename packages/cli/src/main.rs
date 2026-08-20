mod tui;
mod upgrade;

use anyhow::Result;
use chisel_docs::{DocsService, ListOptions};
use chisel_render::OutputMode;
use chisel_specs::{SpecStatus, SpecsService};
use chisel_store::{ContextItem, Store};
use clap::{Parser, Subcommand};
use dialoguer::Input;
use std::path::PathBuf;
use std::str::FromStr;
use tui::{DocsApp, SpecsApp};
use upgrade::UpgradeService;

fn format_context_xml(items: Vec<ContextItem>) -> String {
    let mut output = String::from("<context>\n");
    for item in items {
        let tag = if item.r#type == "spec" {
            "spec"
        } else {
            "file"
        };
        output.push_str(&format!("  <{} path=\"{}\">\n", tag, item.path));
        output.push_str(&item.content);
        output.push_str(&format!("\n  </{}>\n", tag));
    }
    output.push_str("</context>");
    output
}

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
    /// Manage specs
    Spec {
        #[command(subcommand)]
        command: Option<SpecCommands>,
    },
    /// Generate context for LLMs
    Context {
        #[command(subcommand)]
        command: ContextCommands,
    },
    /// Upgrade Chisel to the latest version
    Upgrade,
}

#[derive(Subcommand)]
enum ContextCommands {
    /// Create a context blob for a given query
    Create {
        /// The search query to generate context for
        query: String,
    },
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
    },
}

#[derive(Subcommand)]
enum SpecCommands {
    /// Create a new spec
    New {
        /// The title of the spec
        title: Option<String>,

        /// The area/domain
        #[arg(short, long)]
        area: Option<String>,

        /// Template to use (feature, adr)
        #[arg(short, long)]
        template: Option<String>,

        /// Spec body content; pass '-' to read from stdin
        #[arg(short, long)]
        content: Option<String>,
    },
    /// List specs
    List {
        /// Filter by status (draft, ready, in-progress, shipped, archived)
        #[arg(short, long)]
        status: Option<String>,
    },
    /// View a spec
    View {
        /// The spec slug
        slug: String,
    },
    /// Change spec status
    Status {
        /// The spec slug
        slug: String,
        /// New status (draft, ready, in-progress, shipped, archived)
        new_status: String,
    },
    /// Search specs
    Search {
        /// The search query
        query: String,
    },
    /// Edit a spec
    Edit {
        /// The spec slug
        slug: String,
    },
    /// Delete a spec
    Delete {
        /// The spec slug
        slug: String,
    },
    /// Index all specs
    Index,
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
            println!(
                "💡 A new version of Chisel is available: v{} (current: v{})",
                latest,
                env!("CARGO_PKG_VERSION")
            );
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
                Some(DocsCommands::List {
                    path,
                    all,
                    no_ignore,
                    hidden,
                }) => match mode {
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
                },
                Some(DocsCommands::Show { path }) => match mode {
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
                },
                Some(DocsCommands::Index { .. }) => {
                    service.index_all().await?;
                    if let OutputMode::Human = mode {
                        println!("Indexing complete.");
                    }
                }
                Some(DocsCommands::Search { query }) => match mode {
                    OutputMode::Machine => {
                        mode.render(service.search(&query).await?)?;
                    }
                    OutputMode::Human => {
                        run_docs_explorer(service, mode, None).await?;
                    }
                },
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
                Some(DocsCommands::Edit { path }) => match mode {
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
                },
                Some(DocsCommands::Move { path, category }) => match mode {
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
                },
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
        Commands::Spec { command } => {
            let service = SpecsService::new(root).await?;
            match command {
                Some(SpecCommands::New {
                    title,
                    area,
                    template,
                    content,
                }) => {
                    let title = match title {
                        Some(t) => t,
                        None => {
                            if let OutputMode::Machine = mode {
                                anyhow::bail!("Title is required in machine mode");
                            }
                            Input::<String>::new()
                                .with_prompt("Spec Title")
                                .interact_text()?
                        }
                    };
                    let content = match content.as_deref() {
                        Some("-") => {
                            let mut buf = String::new();
                            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
                            Some(buf)
                        }
                        other => other.map(str::to_string),
                    };
                    let spec = service
                        .create(&title, area, template.as_deref(), content.as_deref())
                        .await?;
                    match mode {
                        OutputMode::Machine => {
                            mode.render(spec)?;
                        }
                        OutputMode::Human => {
                            println!("Created spec: {} ({})", spec.title, spec.slug);
                            println!("  → {}", spec.path.display());
                        }
                    }
                }
                Some(SpecCommands::List { status }) => {
                    let status_enum =
                        status.and_then(|s| SpecStatus::from_str(&s.to_lowercase()).ok());
                    match mode {
                        OutputMode::Machine => {
                            mode.render(service.list(status_enum).await?)?;
                        }
                        OutputMode::Human => {
                            run_specs_explorer(service, mode, status_enum).await?;
                        }
                    }
                }
                Some(SpecCommands::View { slug }) => {
                    let spec = service.show(&slug).await?;
                    mode.render(spec)?;
                }
                Some(SpecCommands::Status { slug, new_status }) => {
                    let s_enum = SpecStatus::from_str(&new_status.to_lowercase())
                        .map_err(|_| anyhow::anyhow!("Invalid status: {}", new_status))?;
                    let spec = service.update_status(&slug, s_enum).await?;
                    match mode {
                        OutputMode::Machine => {
                            mode.render(spec)?;
                        }
                        OutputMode::Human => {
                            println!(
                                "Updated '{}' → {} ({})",
                                spec.slug,
                                spec.status,
                                spec.path.display()
                            );
                        }
                    }
                }
                Some(SpecCommands::Search { query }) => {
                    mode.render(service.search(&query).await?)?;
                }
                Some(SpecCommands::Edit { slug }) => {
                    let spec = service.edit(&slug).await?;
                    mode.render(spec)?;
                }
                Some(SpecCommands::Delete { slug }) => {
                    service.delete(&slug).await?;
                    if let OutputMode::Human = mode {
                        println!("Deleted spec: {}", slug);
                    }
                }
                Some(SpecCommands::Index) => {
                    service.index_all().await?;
                    if let OutputMode::Human = mode {
                        println!("Indexing complete.");
                    }
                }
                None => {
                    run_specs_explorer(service, mode, None).await?;
                }
            }
        }
        Commands::Context { command } => {
            let store = Store::new(root).await?;
            match command {
                ContextCommands::Create { query } => {
                    let items = store.fetch_context(&query).await?;
                    let output = format_context_xml(items);
                    println!("{}", output);
                }
            }
        }
    }

    Ok(())
}

async fn init_workspace(
    root: &std::path::Path,
    name: Option<String>,
    mode: OutputMode,
) -> Result<()> {
    let project_name = name.unwrap_or_else(|| {
        root.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "My Project".to_string())
    });

    if let OutputMode::Human = mode {
        println!("Initializing Chisel workspace for: {}...", project_name);
    }

    // 1. Create .chisel directory for docs + index
    let chisel_dir = root.join(".chisel");
    let docs_dir = chisel_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

    // 2. Initialize Services
    let docs_service = DocsService::new(root.to_path_buf()).await?;
    let specs_service = SpecsService::new(root.to_path_buf()).await?;

    // 3. Delegate seed generation to services
    docs_service.init(&project_name).await?;
    specs_service.init().await?;

    // 4. Generate AI Agent Prompt
    let prompt_path = chisel_dir.join("PROMPT.md");
    let prompt_content = format!(
        "# Chisel Project: {}\n\nThis project uses Chisel for documentation and specs.\n\n## Structure\n- Docs: `.chisel/docs/` (Markdown)\n- Specs: `.chisel/specs/` (Markdown with YAML frontmatter; each spec's lifecycle stage lives in its `status` field: draft, ready, in-progress, shipped, or archived)\n\n## Guidelines\nWhen performing tasks in this repo, you can use `chisel docs` and `chisel spec` with the `--machine` flag to inspect and update the project state efficiently.\n\nUse `chisel context create <query>` to gather relevant docs and specs as structured context.",
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
        println!("Success! Chisel is ready.");
        println!("Try running `chisel docs` or `chisel spec` to begin.");
    }
    Ok(())
}

async fn run_docs_explorer(
    service: DocsService,
    mode: OutputMode,
    initial_path: Option<PathBuf>,
) -> Result<()> {
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

async fn run_specs_explorer(
    service: SpecsService,
    mode: OutputMode,
    status_filter: Option<SpecStatus>,
) -> Result<()> {
    match mode {
        OutputMode::Machine => {
            mode.render(service.list(status_filter).await?)?;
        }
        OutputMode::Human => {
            let mut app = SpecsApp::new(service, status_filter).await?;
            let _ = app.run().await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_init_workspace() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        init_workspace(root, Some("Test Project".to_string()), OutputMode::Machine)
            .await
            .unwrap();

        assert!(root.join(".chisel").exists());
        assert!(root.join(".chisel/docs/welcome-to-chisel.md").exists());
        assert!(root
            .join(".chisel/docs/tutorial/working-with-docs.md")
            .exists());
        assert!(root.join(".chisel/specs").exists());
        assert!(root.join(".chisel/specs/example-feature-spec.md").exists());
        assert!(root.join(".chisel/PROMPT.md").exists());
        assert!(root.join(".gitignore").exists());

        let gitignore = std::fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(gitignore.contains(".chisel/index.db"));

        let prompt = std::fs::read_to_string(root.join(".chisel/PROMPT.md")).unwrap();
        assert!(prompt.contains("specs"));
    }

    #[test]
    fn test_format_context_xml() {
        let items = vec![
            ContextItem {
                path: "path/to/doc.md".to_string(),
                content: "# Title\nContent".to_string(),
                r#type: "document".to_string(),
            },
            ContextItem {
                path: ".chisel/specs/active/auth.md".to_string(),
                content: "Spec content".to_string(),
                r#type: "spec".to_string(),
            },
        ];

        let output = format_context_xml(items);

        assert!(output.contains("<context>"));
        assert!(output.contains("<file path=\"path/to/doc.md\">"));
        assert!(output.contains("# Title\nContent"));
        assert!(output.contains("</file>"));
        assert!(output.contains("<spec path=\".chisel/specs/active/auth.md\">"));
        assert!(output.contains("Spec content"));
        assert!(output.contains("</spec>"));
        assert!(output.contains("</context>"));
    }
}
