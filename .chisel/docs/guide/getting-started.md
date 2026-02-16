---
title: Getting Started
created_at: 2026-02-15T15:35:00Z
tags: [guide, onboarding]
---

# Getting Started

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

This command creates a `.chisel/` directory and populates it with some helpful onboarding documentation and initial tasks.

## Exploration

### 1. Launch the Docs Explorer
```bash
chisel docs
```
Use `j`/`k` to navigate and `Tab` to switch between categories and the preview pane.

### 2. Open the Issue Board
```bash
chisel issues
```
Move an issue from `Todo` to `In Progress` by pressing `m`.

## Next Steps
- Learn how to [organize your knowledge](./working-with-docs.md).
- Integrate Chisel with [AI agents](./ai-integration.md).
