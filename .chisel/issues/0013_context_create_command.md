---
title: Implement 'context create' command for LLM context generation
status: done
priority: high
labels:
- feature
- cli
- llm
created_at: 2024-05-24T10:00:00Z
order: 0
external_id: null
---

# Implement 'context create' command

The goal is to add a new command to the Chisel CLI that generates a blob of context relevant to a specific topic or query, optimized for use with LLMs.

## User Story
As a user working with an LLM, I want to quickly gather relevant documentation and issues related to a specific term (e.g., "app routing") so that I can paste it into the LLM's context window.

## Proposed Command
`chisel context create <query>`

## Requirements
1.  Search the `chisel-store` (SQLite) using the existing FTS index for both documents and issues.
2.  Retrieve the full content of the most relevant matches.
3.  Format the output in a structured way (e.g., XML tags or Markdown blocks) that clearly delimits file paths and content.
4.  Output to stdout so it can be piped or copied.

## Implementation Steps
1.  Add `context` subcommand to `main.rs`.
2.  Implement search logic in `chisel-store` to retrieve full content for top N matches.
3.  Format the results using XML tags.
