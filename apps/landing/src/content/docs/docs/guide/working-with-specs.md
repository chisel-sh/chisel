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

You will be prompted for a title and template. Chisel creates a new Markdown file in `.chisel/specs/active/` with the appropriate frontmatter.

### Templates

- **feature**: For new features or enhancements. Includes sections for motivation, design, and acceptance criteria.
- **adr**: Architecture Decision Record. Includes context, decision, and consequences sections.

```bash
chisel spec new --template adr
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

When a spec moves to `shipped`, Chisel relocates its file from `.chisel/specs/active/` to `.chisel/specs/shipped/`. When archived, it moves to `.chisel/specs/archived/`.

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
├── active/          # Draft, Ready, and InProgress specs
│   ├── 0001_user-auth-flow.md
│   └── 0002_api-rate-limiting.md
├── shipped/         # Completed specs
│   └── 0003_dark-mode.md
└── archived/        # Historical specs
    └── 0004_deprecated-endpoint.md
```

## Why Local Specs?

By keeping specs as versioned Markdown files, your planning artifacts follow your code through branches, reverts, and history. There is no drift between your tracker and your repository.
