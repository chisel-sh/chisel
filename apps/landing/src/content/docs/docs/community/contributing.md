---
title: Contributing
created_at: 2026-02-15T15:25:00Z
tags:
- community
- contributing
order: null
---


We welcome contributions of all kinds: bug fixes, feature requests, documentation, and new adapters!

## Workflow

1. **Fork the Repo**: Create your own copy of the repository on GitHub.
2. **Create a Branch**: Use a descriptive name like `feat/new-adapter` or `fix/tui-crash`.
3. **Commit often**: We use **Conventional Commits** (`feat:`, `fix:`, `chore:`, etc.). This helps our automated release tools (`release-plz`) generate accurate changelogs.
4. **Run Tests**: Ensure all tests pass before submitting.
   ```bash
   cargo test
   ```
5. **Open a PR**: Submit your changes for review.

## Coding Standards
- **Rustfmt**: Run `cargo fmt` to ensure consistent styling.
- **Clippy**: Run `cargo clippy` to catch common mistakes.
- **Documentation**: If you add a feature, please add a corresponding doc in `.chisel/docs/`.

## Specs
We use Chisel to manage Chisel! You can find our current roadmap and specs by running `chisel spec list` in the root of this repository.
