---
title: Adapter Pattern
created_at: 2026-02-15T15:20:00Z
tags:
- architecture
- integration
- traits
order: null
---


To remain "Source Agnostic," Chisel relies on the **Adapter Pattern** implemented via Rust traits. This allows Chisel to manage documentation and specs regardless of where they are stored.

## DataSource Trait (Docs)
Defined in `packages/chisel-docs/src/source.rs`. Any implementation can power the Chisel Docs TUI.

- **`DefaultSource`**: Manages Markdown files in `.chisel/docs/`.
- **`StarlightSource`**: Adapts to an existing Astro Starlight structure in `src/content/docs/`.

## SpecSource Trait (Specs)
Defined in `packages/chisel-specs/src/source.rs`.

- **`DefaultSpecSource`**: Manages local Markdown specs in `specs/{active,shipped,archived}/`.
- **Future Adapters**: Potential implementations for `GitHubSpecSource` or `LinearSource`.

## Why this matters
This architecture allows Chisel to be a **unified interface** for engineering information. You can use the same TUI shortcuts and AI machine-mode schemas to interact with local files, cloud providers, or custom documentation sites.
