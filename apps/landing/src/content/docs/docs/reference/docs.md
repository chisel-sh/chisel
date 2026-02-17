---
title: Docs Reference
created_at: 2026-02-15T15:00:00Z
tags: [reference, docs]
order: 2
---


Chisel Docs is a Markdown-first knowledge base.

## Storage Structure

Documents are stored in `.chisel/docs/` as `.md` files. Subdirectories represent categories.

- `.chisel/docs/INDEX.md`: Automatically managed index of all documents.
- `.chisel/docs/category/_category.yaml`: Optional metadata for a category (label, order).

## Frontmatter

Chisel uses YAML frontmatter to store document metadata.

```yaml
---
title: Document Title
created_at: 2026-02-15T15:00:00Z
tags: [tag1, tag2]
order: 10
---
```

### Fields

- `title`: The display name of the document.
- `created_at`: ISO 8601 timestamp.
- `tags`: List of strings for categorization.
- `order`: Integer used for sorting in lists and the index.

## TUI Shortcuts

When running `chisel docs`:

- `Tab`: Cycle through Sidebar, Main List, and Preview panes.
- `j/k` or `Up/Down`: Move selection.
- `Enter`: Edit document (opens `$EDITOR`).
- `/`: Search.
- `n`: Create new document.
- `m`: Move document to a new category.
- `[` / `]`: Adjust document or category order.
- `q`: Quit.
