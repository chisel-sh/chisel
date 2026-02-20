---
title: Design Workspace Versioning & Migrations System
status: todo
priority: low
labels:
- architecture
- enhancement
created_at: 2026-02-19T16:27:23.291634276Z
order: 0
external_id: null
---

We need a robust strategy for handling workspace changes (migrations) as Chisel evolves.

Currently, we handle some things ad-hoc, but we should formalize:
- A `version` field in `.chisel/chisel.toml`.
- A mechanism to detect if the CLI version is incompatible with the workspace version.
- A `chisel migrate` command to apply pending migrations.
- Versioning the `.chisel/` directory structure itself.

See discussion in #8 for initial thoughts on a JIT vs Explicit approach.
