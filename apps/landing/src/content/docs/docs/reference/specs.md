---
title: Specs Reference
description: Frontmatter schema, CLI commands, and directory structure for Chisel Specs.
sidebar:
  order: 2
  label: Specs Reference
---


Chisel Specs is a lifecycle-driven spec tracker stored as local Markdown files.

## Directory Structure

Specs are organized by lifecycle state:

```
.chisel/specs/
├── user-auth-flow.md      # status: in-progress
├── api-rate-limiting.md   # status: draft
└── dark-mode.md           # status: shipped
```

All specs live directly in the specs directory; lifecycle stage is tracked by the `status` frontmatter field, not by subdirectories. Legacy workspaces using `active/`, `shipped/`, and `archived/` subdirectories are migrated to the flat layout automatically on first use.

File naming convention: `<id>_<slug>.md` (e.g., `0001_user-auth-flow.md`).

## Frontmatter Schema

```yaml
---
title: User Auth Flow
status: draft
created: 2026-03-15T10:00:00Z
updated: 2026-03-20T14:30:00Z
area: backend
related_docs:
  - docs/architecture/auth.md
open_questions:
  - Should we support OAuth2 refresh tokens?
---
```

### Fields

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `title` | string | yes | Human-readable spec title. |
| `status` | string | yes | One of `draft`, `ready`, `in-progress`, `shipped`, `archived`. |
| `created` | datetime | yes | ISO 8601 creation timestamp. |
| `updated` | datetime | no | ISO 8601 last-modified timestamp. |
| `area` | string | no | Project area or domain (e.g., `backend`, `cli`, `docs`). |
| `related_docs` | list | no | Paths to related documentation files. |
| `open_questions` | list | no | Unresolved questions to be addressed before moving to `ready`. |

## Status Values

- **draft**: Spec is being written or refined. Not yet actionable.
- **ready**: Spec is reviewed and approved. Implementation can start.
- **in-progress**: Active work is underway against this spec.
- **shipped**: Implementation is complete, merged, or deployed.
- **archived**: Spec is no longer relevant. Retained for reference.

## CLI Commands

### `chisel spec new`

Create a new spec. Accepts `--template` to select a template (`feature`, `adr`) and `--content` to provide the spec body inline; pass `--content -` to read the body from stdin. Explicit content takes precedence over a template.

### `chisel spec list`

List specs. Accepts `--status` to filter by lifecycle state.

### `chisel spec view <id>`

Display the full content of a spec.

### `chisel spec status <id> <status>`

Change a spec's status. Rewrites the `status` field in the spec's frontmatter; the file stays in place.

### `chisel spec search <query>`

Full-text search across all specs using the FTS5 index.

### `chisel spec edit <id>`

Open a spec in your `$EDITOR`.

### `chisel spec delete <id>`

Permanently remove a spec file.

### `chisel spec index`

Rebuild the spec search index.

All commands support the `--machine` flag for structured YAML output.
