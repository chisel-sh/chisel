---
title: Allow passing content to chisel issues new
status: todo
priority: medium
labels:
- cli
- enhancement
created_at: 2026-02-18T23:57:55.742128776Z
order: 0
external_id: null
---

We should add a new issue in our issue tracker - `chisel issues new` can't be passed the content of the issue, or `chisel issue new --machine` can't be used after creation to edit the content.

Expected behavior:
- `chisel issues new --content "My issue content"`
- OR allow piping content to stdin?
- Provide a way to edit content via CLI without opening an editor.
