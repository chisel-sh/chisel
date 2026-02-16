---
title: Codebase Overview
created_at: 2026-02-15T15:15:00Z
tags: [architecture, internal, development]
---

# Codebase Overview

Chisel is organized as a Rust workspace with decoupled packages to ensure maintainability and testability.

## Package Map

### `packages/cli`
The entry point. Handles CLI argument parsing (via `clap`) and orchestrates the high-level services. Contains the `main.rs` and the `tui.rs` (Ratatui applications).

### `packages/chisel-docs`
Core logic for the documentation engine. Manages document lifecycle, category resolution, and delegates storage to the `DataSource` trait.

### `packages/chisel-issues`
Core logic for the issue tracker. Manages Kanban status transitions, priority handling, and delegates storage to the `IssueSource` trait.

### `packages/chisel-store`
The persistence layer. Wraps **SQLite** (via `sqlx`) to provide the FTS5 full-text search index used by both Docs and Issues.

### `packages/chisel-fs`
Low-level file system utilities, slugification, and editor spawning logic shared across the workspace.

### `packages/chisel-render`
Shared rendering traits and color tokens for ensuring a consistent look and feel across all TUIs.

## Design Patterns
- **Trait-based Adapters**: Both Docs and Issues use a "Source" trait to allow swapping the backend (e.g., local files vs. GitHub API).
- **Asynchronous by Default**: All services are built using `tokio` to handle concurrent file system and network operations efficiently.
