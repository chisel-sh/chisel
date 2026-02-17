---
title: Working with Docs
created_at: 2026-02-15T15:40:00Z
tags: [guide, docs, organization]
---


Chisel Docs is more than just a file browser; it's a structured knowledge base designed for high-speed navigation.

## Organization
Docs are stored in `.chisel/docs/`. You can organize them into categories by creating subdirectories.

- `.chisel/docs/api/endpoints.md` -> Category: **API**
- `.chisel/docs/guides/auth.md` -> Category: **GUIDES**

## Key Shortcuts
- `/`: Search all documents (uses the SQLite FTS5 index).
- `n`: Create a new document. You'll be prompted for a title and category.
- `e`: Open the selected document in your default `$EDITOR`.
- `m`: Move a document to a different category.
- `Tab`: Cycle focus between Sidebar, Document List, and Preview.

## Indexing
Chisel automatically generates an `INDEX.md` file in your docs root. This provides a human-readable table of contents that is also easily parsed by Git-based documentation tools.
