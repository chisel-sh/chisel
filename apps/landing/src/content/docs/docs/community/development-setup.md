---
title: Development Setup
created_at: 2026-02-15T15:30:00Z
tags: [community, development, setup]
---


Follow these steps to set up Chisel for local development.

## Prerequisites
- **Rust**: Latest stable version (use [rustup](https://rustup.rs/)).
- **Node.js**: Required for building the landing page (Astro).
- **VHS**: Required for generating demo videos (`go install github.com/charmbracelet/vhs@latest`).

## Building the CLI
```bash
cargo build --release --bin chisel
```
The binary will be available at `target/release/chisel`.

## Running the Landing Page
```bash
cd apps/landing
npm install
npm run dev
```

## Generating Demo Assets
We use a script to ensure demos are recorded in a clean environment:
```bash
./generate_demos.sh
```

## Internal Testing
Chisel includes a suite of integration tests for the workspace initialization and service logic:
```bash
cargo test
```
