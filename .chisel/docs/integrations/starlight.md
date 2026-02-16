---
title: Starlight Integration
created_at: 2026-02-15T15:50:00Z
tags: [integrations, starlight, astro]
---

# Astro Starlight Integration

Chisel can automatically adapt to projects using the **Astro Starlight** documentation framework.

## How it works
Chisel detects the presence of `src/content/docs/` in your repository. If found, it will bypass its default `.chisel/docs/` folder and use your Starlight directory as the source of truth.

## Features
- **Zero Config**: Just run `chisel docs` in a Starlight project.
- **Metadata Mapping**: Chisel maps Starlight's `sidebar.order` field to the TUI's internal sorting logic.
- **MDX Support**: Chisel can read and preview `.mdx` files natively.

## Frontmatter Compatibility

Chisel respects Starlight's metadata schema while adding its own when necessary.

**Starlight Frontmatter:**
```yaml
title: My Doc
sidebar:
  order: 1
```

**Chisel Mapping:**
- Chisel's `order` field maps to Starlight's `sidebar.order`.
- Chisel adds `created_at` which Starlight safely ignores.

## Manual Configuration

If your Starlight docs are in a non-standard location, or if you want to force Chisel to use a specific directory, you can currently use a symlink:

```bash
# Force Chisel to use a specific directory
ln -s custom/docs/path .chisel/docs
```

*Note: Automated configuration via `chisel.toml` is planned for a future release.*
