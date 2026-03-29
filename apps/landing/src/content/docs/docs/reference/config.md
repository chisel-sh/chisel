---
title: Configuration
description: Reference for the chisel.toml configuration file.
sidebar:
  order: 4
  label: Configuration
---

Chisel is configured via an optional `chisel.toml` file in the project root. If the file is absent, Chisel uses sensible defaults.

## Example

```toml
[docs]
source = "apps/landing/src/content/docs"

[specs]
source = "design/specs"
```

## `[docs]`

Configuration for Chisel Docs.

### `source`

- **Type:** path (string)
- **Default:** `.chisel/docs/`

Override the directory Chisel uses as the docs source. This is useful when your documentation lives outside `.chisel/`, for example in an Astro Starlight project.

```toml
[docs]
source = "apps/landing/src/content/docs"
```

When set, Chisel reads and indexes Markdown files from this directory instead of `.chisel/docs/`. The path is relative to the project root.

See [Starlight Integration](/docs/integrations/starlight/) for a full walkthrough.

## `[specs]`

Configuration for Chisel Specs.

### `source`

- **Type:** path (string)
- **Default:** `.chisel/specs/`

Override the directory Chisel uses as the specs source. Specs are organized into `active/`, `shipped/`, and `archived/` subdirectories within this path.

```toml
[specs]
source = "design/specs"
```

When set, Chisel reads and writes specs to this directory instead of `.chisel/specs/`. The path is relative to the project root.

## File Location

Chisel looks for `chisel.toml` in the current working directory. It does not search parent directories.
