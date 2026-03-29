---
title: Getting Started
created_at: 2026-02-15T15:35:00Z
tags: [guide, onboarding]
---


Welcome to Chisel! This guide will get you up and running in less than 5 minutes.

## Installation

Chisel is distributed as a single binary. Install it using our shell script:

```bash
curl -sL https://install.chisel.build | sh
```

## Initializing your first Workspace

Navigate to any project (e.g., a Rust repo, a web app, or a folder of notes) and run:

```bash
chisel init
```

This command creates a `.chisel/` directory with docs and specs subdirectories, and populates them with helpful onboarding documentation and an initial spec.

## Exploration

### 1. Launch the Docs Explorer
```bash
chisel docs
```
Use `j`/`k` to navigate and `Tab` to switch between categories and the preview pane.

### 2. View Your Specs
```bash
chisel spec list
```
Create a new spec with `chisel spec new` or move one through its lifecycle with `chisel spec status`.

## Next Steps
- Learn how to [organize your knowledge](./working-with-docs.md).
- Learn how to [work with specs](./working-with-specs.md).
- Integrate Chisel with [AI agents](./ai-integration.md).
