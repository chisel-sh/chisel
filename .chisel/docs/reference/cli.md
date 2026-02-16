---
title: CLI Reference
created_at: 2026-02-15T15:00:00Z
tags: [reference, cli]
order: 1
---

# CLI Reference

Chisel is a suite of terminal-native tools. All commands support a `--machine` flag for LLM-friendly output.

## Global Flags

- `-h, --help`: Show help information.
- `-V, --version`: Show version information.
- `-m, --machine`: Output in machine-readable format (YAML).

## Commands

### `chisel init`
Initialize a new Chisel workspace in the current directory.
- Creates `.chisel/` directory.
- Sets up default documentation and issue tracking.
- Generates `PROMPT.md` for AI agents.

### `chisel docs`
Manage your project's knowledge base.
- `chisel docs overview`: Show workspace stats and recent changes.
- `chisel docs list`: List available documents.
- `chisel docs show <path>`: Display document content.
- `chisel docs index`: Rebuild the local SQLite search index.
- `chisel docs search <query>`: Search documents using FTS5.
- `chisel docs new`: Create a new document.
- `chisel docs edit <path>`: Open a document in your `$EDITOR`.

### `chisel issues`
Manage your project's tasks.
- `chisel issues overview`: Show issues by status.
- `chisel issues list`: List available issues (filter by `--status`).
- `chisel issues show <id>`: Show issue details.
- `chisel issues new`: Create a new issue.
- `chisel issues edit <id>`: Open an issue in your `$EDITOR`.
- `chisel issues close <id>`: Mark an issue as closed.
