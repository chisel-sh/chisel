---
title: Working with Specs
description: Create and manage lifecycle specs to track features, decisions, and work items.
sidebar:
  order: 3
  label: Working with Specs
---


Chisel Specs provides a lifecycle-driven workflow for tracking features, architectural decisions, and work items as Markdown files alongside your code.

## Creating a Spec

```bash
chisel spec new
```

You will be prompted for a title and template. Chisel creates a new Markdown file in `.chisel/specs/` with the appropriate frontmatter.

### Templates

- **feature**: For new features or enhancements. Includes sections for motivation, design, and acceptance criteria.
- **adr**: Architecture Decision Record. Includes context, decision, and consequences sections.

```bash
chisel spec new --template adr
```

### Providing Content Inline

Skip the template and supply the spec body in one command — handy for scripts and LLM agents that already have the content:

```bash
chisel spec new "API Rate Limiting" --content "## Summary\n\nThrottle requests per API key."
cat design-notes.md | chisel spec new "API Rate Limiting" --content -
```

## Lifecycle States

Every spec moves through a defined lifecycle:

1. **Draft** — Initial idea or proposal. Still being shaped.
2. **Ready** — Spec is complete and reviewed. Work can begin.
3. **InProgress** — Active implementation underway.
4. **Shipped** — Work is complete and deployed/merged.
5. **Archived** — No longer relevant. Kept for historical reference.

## Moving Specs Through Status

```bash
chisel spec status <id> <new-status>
```

Examples:

```bash
chisel spec status 0001 ready
chisel spec status 0001 in-progress
chisel spec status 0001 shipped
```

Status changes update the `status` field in the spec's frontmatter — the file itself never moves, so links and paths stay stable across the entire lifecycle.

## Listing and Searching

```bash
chisel spec list                  # List all active specs
chisel spec list --status draft   # Filter by status
chisel spec search "auth"         # Full-text search across specs
chisel spec view <id>             # View a specific spec
```

## Machine Mode

All spec commands support `--machine` for structured YAML output, suitable for LLM context windows and script automation.

```bash
chisel spec list --machine
```

## Directory Structure

```
.chisel/specs/
├── user-auth-flow.md      # status: in-progress
├── api-rate-limiting.md   # status: draft
├── dark-mode.md           # status: shipped
└── deprecated-endpoint.md # status: archived
```

All specs live in a single flat directory; each file's lifecycle stage is the `status` field in its frontmatter. Workspaces created before this layout (with `active/`, `shipped/`, and `archived/` subdirectories) are migrated automatically the first time any `chisel spec` command runs — files move into `.chisel/specs/` and a summary of the moves is printed.

## Why Local Specs?

By keeping specs as versioned Markdown files, your planning artifacts follow your code through branches, reverts, and history. There is no drift between your tracker and your repository.
