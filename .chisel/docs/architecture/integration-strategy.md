# Architecture: Integration & Adapter API

## Motivation
Chisel is designed to be decoupled from specific storage conventions. To support tools like **Astro Starlight**, **Obsidian**, or **Logseq**, we use an "Adapter Pattern" that separates core logic (TUI, Indexing, LLM Search) from the data source (File System, Metadata conventions).

## Core Abstraction: The `DataSource` Trait

The `DataSource` trait defines how Chisel interacts with content backends.

```rust
#[async_trait]
pub trait DataSource: Send + Sync {
    /// Scan the source and return a flat list of discoverable items
    async fn list(&self) -> Result<Vec<Doc>>;

    /// Parse a specific file into Chisel's internal domain model
    async fn load(&self, path: PathBuf) -> Result<Doc>;

    /// Persist changes back to the source
    async fn save(&self, doc: &Doc) -> Result<()>;
    
    /// Remove an item from the source
    async fn delete(&self, path: PathBuf) -> Result<()>;
}
```

## Decision: The Internal Adapter Pattern

We have adopted an architecture similar to **Helix's LSP integration** or **Cargo's Registry Source**. We implement strict Rust traits for different backends within the Chisel codebase.

### Why Internal Adapters?
1.  **Latency:** Indexing thousands of markdown files requires tight loops. IPC/WASM overhead is avoided for core operations.
2.  **Type Safety:** Native Rust implementations provide the best developer experience and reliability.
3.  **Future-Proof:** This design prepares us for a future WASM-based plugin system; we can simply implement a `WasmSource` that wraps a runtime.

## Case Study: Astro Starlight

Integrating Chisel with Starlight allows a single set of Markdown files to serve as both a web documentation site and a terminal-accessible knowledge base.

| Feature | Compatibility |
| :--- | :--- |
| **Content Root** | `src/content/docs/` vs `.chisel/docs/` |
| **File Format** | `.md`, `.mdx` support |
| **Ordering** | Maps Chisel `order` to Starlight `sidebar.order` |

## Future Evolution: Plugin API

If we expose the `DataSource` trait via **WASM** or a **JSON-RPC** interface, community members could write external plugins such as:
*   `chisel-plugin-notion` (Syncs Notion pages to Chisel TUI)
*   `chisel-plugin-linear` (Syncs Linear issues to Chisel TUI)
