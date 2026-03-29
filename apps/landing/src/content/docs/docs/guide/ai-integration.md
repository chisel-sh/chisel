---
title: AI Integration
created_at: 2026-02-15T15:10:00Z
tags: [ai, machine-mode, automation]
---


Chisel is designed from the ground up to be the bridge between human developers and AI agents (like Gemini, Claude, or GPT).

## The Machine Mode Flag
Every command in Chisel accepts the `--machine` flag. This transforms the output from a terminal-formatted UI into a clean, structured YAML block.

### Example: Providing Context to an LLM
Instead of copying and pasting screenshots, you can pipe Chisel output directly into an AI prompt:

```bash
chisel docs list --machine | your-ai-tool "Which docs should I read to understand the auth system?"
```

## PROMPT.md
When you run `chisel init`, a `PROMPT.md` file is created in your `.chisel/` directory. This file contains instructions for AI agents on how to use Chisel to navigate your project.

**Recommended Workflow:**
Include the content of `.chisel/PROMPT.md` in your AI agent's "System Instructions" or "Custom Instructions" to bootstrap its awareness of your knowledge base.

## Structured Schemas
The machine mode uses stable, predictable keys. This allows you to build reliable scripts that automate spec triage, documentation auditing, or status reporting without worrying about UI changes.
