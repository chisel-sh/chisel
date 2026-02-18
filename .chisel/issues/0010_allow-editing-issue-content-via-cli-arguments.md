---
title: Allow editing issue content via CLI arguments
status: todo
priority: medium
labels:
- cli
- enhancement
created_at: 2026-02-18T23:58:14.789231418Z
order: 0
external_id: null
---

Currently `chisel issues edit` opens an interactive editor. There is no way to update the content of an issue via CLI flags or machine mode.

Expected behavior:
- `chisel issues edit <ID> --content "New content"`
- Support for piping content to stdin for updates.
