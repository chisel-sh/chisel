---
title: "Implement Chisel Designs CLI"
status: proposed
type: rfc
issues: [11]
created_at: 2026-02-25
---

# RFC: Chisel Designs System

## 1. Problem Statement
Chisel currently manages "Work" (Issues) and "Knowledge" (Docs), but lacks a dedicated space for "Strategy" (Planning). Developers are currently creating manual design docs in `.chisel/designs/`, but there is no CLI support for:
- Standardized templating (PRDs vs ADRs).
- Tracking the lifecycle of a design (Draft -> Approved -> Superseded).
- Cross-linking designs to the issues they implement.

## 2. Proposed Solution
Introduce a `chisel designs` subcommand that manages a new `chisel-designs` package.

### 2.1 Directory Structure
All designs will live in `.chisel/designs/` with a numbered prefix to ensure chronological order:
- `.chisel/designs/0001_initial-design.md`
- `.chisel/designs/0002_follow-up.md`

### 2.2 CLI Interface
- `chisel designs list`: Show all designs, their type, and status.
- `chisel designs show <id>`: Render the design in the TUI.
- `chisel designs new <title>`: Create a new design.
  - `--rfc`: (Default) Technical proposal.
  - `--adr`: Architectural decision record (immutable history).
  - `--prd`: Product requirements.
  - `--tech-spec`: Detailed implementation plan.
- `chisel designs status <id> <status>`: Update the status (draft, proposed, accepted, superseded).

### 2.3 Cross-Linking
The `chisel issues show <id>` command should be updated to look for any design docs that list the issue ID in their frontmatter.

## 3. Template Definitions

### RFC Template
- **Context:** Why are we doing this?
- **Proposed Changes:** Technical details.
- **Impact:** What changes for the user?
- **Alternatives:** What else did we consider?

### ADR Template
- **Title:** The decision name.
- **Date:** When it was made.
- **Status:** Proposed/Accepted/Superseded.
- **Context:** The situation leading to the decision.
- **Decision:** What we are doing.
- **Consequences:** The pros and cons of the result.

## 4. Dogfooding Goals
By using this file to plan the feature, we want to test:
1. Is the YAML frontmatter easy to maintain manually?
2. Does the "Type" distinction (RFC vs ADR) actually help organize thoughts?
3. How does it feel to "refer back" to this file while writing code?
