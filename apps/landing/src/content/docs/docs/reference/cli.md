---
title: CLI Reference
sidebar:
  order: 1
  label: CLI Reference
---


Chisel is a suite of terminal-native tools. All commands support a `--machine` flag for LLM-friendly output.

## Global Flags

- `-h, --help`: Show help information.
- `-V, --version`: Show version information.
- `-m, --machine`: Output in machine-readable format (YAML).

## Commands

### `chisel init`
Initialize a new Chisel workspace in the current directory.
- Creates `.chisel/` directory.
- Sets up default documentation and spec tracking.
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

### `chisel spec`
Manage your project's specs.
- `chisel spec list`: List specs (filter by `--status`).
- `chisel spec view <id>`: Show spec details.
- `chisel spec new`: Create a new spec (supports `--template`).
- `chisel spec edit <id>`: Open a spec in your `$EDITOR`.
- `chisel spec status <id> <status>`: Change a spec's lifecycle status.
- `chisel spec search <query>`: Full-text search across specs.
- `chisel spec delete <id>`: Remove a spec.
- `chisel spec index`: Rebuild the spec search index.
