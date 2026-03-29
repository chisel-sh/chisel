---
title: "Technical Specification: Chisel Designs Package"
status: proposed
type: tech-spec
issues: [11]
designs: [2] # links to the RFC
created_at: 2026-02-25
---

# Tech Spec: Chisel Designs Package

## 1. Package Structure
We will create a new package `packages/chisel-designs` that mirrors the structure of `chisel-docs` and `chisel-issues`.

- `packages/chisel-designs/src/lib.rs`: The main `DesignsService`.
- `packages/chisel-designs/src/source.rs`: Trait for loading/saving designs.
- `packages/chisel-designs/src/default_source.rs`: Local filesystem implementation.

## 2. Design Model
```rust
pub struct Design {
    pub id: i64,
    pub path: PathBuf,
    pub title: String,
    pub status: DesignStatus,
    pub design_type: DesignType,
    pub issues: Vec<i64>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum DesignStatus {
    Draft,
    Proposed,
    Accepted,
    Superseded,
    Deprecated,
}

pub enum DesignType {
    Rfc,
    Adr,
    Prd,
    TechSpec,
}
```

## 3. CLI Integration
We will update `packages/cli/src/main.rs` to include a `Designs` subcommand.

```rust
#[derive(Subcommand)]
enum DesignsCommands {
    New {
        title: String,
        #[arg(long)]
        rfc: bool,
        #[arg(long)]
        adr: bool,
        #[arg(long)]
        prd: bool,
        #[arg(long)]
        tech_spec: bool,
    },
    List,
    Show { id: i64 },
    UpdateStatus { id: i64, status: String },
}
```

## 4. Shared Utilities
We will move `slugify_title` and `spawn_editor` to a shared utility module if they aren't already available to `chisel-designs`.

## 5. Persistence
The SQLite store in `chisel-store` will need a new table `designs` and FTS support.
- `designs` table: id, path, title, status, type, issues, created_at, updated_at, content.
- FTS support for `chisel designs search`.

## 6. Implementation Plan
1. [ ] Create `packages/chisel-designs` crate.
2. [ ] Implement `Design` model and basic file I/O.
3. [ ] Update `chisel-store` with migrations for the `designs` table.
4. [ ] Implement `DesignsService::list`, `create`, `show`.
5. [ ] Add `chisel designs` command to the CLI.
6. [ ] Update TUI to include a designs view (optional, can start with CLI only).
