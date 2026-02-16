---
title: Issues Reference
created_at: 2026-02-15T15:00:00Z
tags: [reference, issues]
order: 3
---

# Chisel Issues Reference

Chisel Issues is a text-first issue tracker.

## Storage Structure

Issues are stored in `.chisel/issues/` as Markdown files with a numeric ID prefix.

- Example: `.chisel/issues/0001_fix-bug.md`

## Frontmatter

Issues use a strict YAML schema for status and priority tracking.

```yaml
---
title: Fix the login bug
status: todo
priority: high
labels: [bug, auth]
created_at: 2026-02-15T15:00:00Z
order: 0
---
```

### Fields

- `status`: `todo`, `in-progress`, `done`, `closed`, `cancelled`.
- `priority`: `low`, `medium`, `high`, `critical`.
- `labels`: List of strings.
- `order`: Integer for Kanban board sorting.

## TUI Shortcuts

When running `chisel issues`:

- `Tab` / `Shift+Tab`: Move between Kanban columns (Todo -> In Progress -> Done).
- `j/k` or `Up/Down`: Move selection within a column.
- `Enter`: Edit issue description (opens `$EDITOR`).
- `n`: Create new issue.
- `t`: Edit title.
- `p`: Change priority.
- `m`: Move issue (change status).
- `x`: Delete issue.
- `[` / `]`: Adjust issue order.
- `q`: Quit.
