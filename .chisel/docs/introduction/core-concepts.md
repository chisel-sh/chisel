---
title: Core Concepts
created_at: 2026-02-15T15:05:00Z
tags: [concepts, fundamentals]
---

# Core Concepts

Understanding how Chisel structures information will help you get the most out of the tool suite.

## The .chisel Workspace
Every Chisel project revolves around the `.chisel/` directory. This is where your knowledge base and issues live. Keeping this data in your repo ensures it follows your code's lifecycle (branches, reverts, and history).

## Text-as-Source-of-Truth
Unlike tools that hide your data in a database, Chisel uses standard files:
- **Docs**: Nested Markdown files in `.chisel/docs/`.
- **Issues**: Markdown files with YAML frontmatter in `.chisel/issues/`.

## The Hybrid Engine
While the source of truth is text, Chisel maintains a local **SQLite FTS5** index (`.chisel/index.db`).
- **Performance**: Instant search results even with thousands of documents.
- **AI Readiness**: The index provides the foundation for semantic search and RAG (Retrieval-Augmented Generation) workflows.

## Human vs Machine Modes
Every Chisel command supports two outputs:
1. **Human (Default)**: A rich, interactive TUI optimized for developer speed.
2. **Machine (`--machine`)**: A token-dense YAML output optimized for LLM context windows and script automation.
