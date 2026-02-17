---
title: Adapter Pattern
created_at: 2026-02-15T15:20:00Z
tags:
- architecture
- integration
- traits
order: null
---


To remain "Source Agnostic," Chisel relies on the **Adapter Pattern** implemented via Rust traits. This allows Chisel to manage documentation and issues regardless of where they are stored.

## DataSource Trait (Docs)
Defined in `packages/chisel-docs/src/source.rs`. Any implementation can power the Chisel Docs TUI.

- **`DefaultSource`**: Manages Markdown files in `.chisel/docs/`.
- **`StarlightSource`**: Adapts to an existing Astro Starlight structure in `src/content/docs/`.

## IssueSource Trait (Issues)
Defined in `packages/chisel-issues/src/source.rs`.

- **`DefaultIssueSource`**: Manages local Markdown issues in `.chisel/issues/`.
- **Future Adapters**: Potential implementations for `GitHubIssueSource` or `LinearSource`.

## Why this matters
This architecture allows Chisel to be a **unified interface** for engineering information. You can use the same TUI shortcuts and AI machine-mode schemas to interact with local files, cloud providers, or custom documentation sites.
