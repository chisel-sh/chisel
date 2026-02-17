---
title: Managing Issues
created_at: 2026-02-15T15:45:00Z
tags: [guide, issues, kanban]
---


Chisel Issues provides a text-first Kanban workflow that lives alongside your code.

## The Workflow
Issues are stored as Markdown files in `.chisel/issues/`. Each file contains a YAML frontmatter block that tracks its state.

### Statuses
- **Todo**: Tasks waiting to be started.
- **In Progress**: Active work.
- **Done / Closed**: Completed or resolved tasks.

## Key Shortcuts
- `n`: Create a new issue.
- `m`: Change the status of the selected issue (moves it between lanes).
- `p`: Cycle priority (Low, Medium, High, Critical).
- `t`: Edit the issue title.
- `x`: Delete an issue (permanently removes the file).

## Why local issues?
By keeping your tasks in Markdown files, your "To-Do" list is branched and versioned with your code. This eliminates the "Jira drift" where the issue tracker doesn't match what's actually in the repository.
