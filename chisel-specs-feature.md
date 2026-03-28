# Chisel: Specs Feature & Product Refocus

**Status:** Draft  
**Created:** 2026-03-28  
**Area:** Product Strategy + Engineering  
**Author:** TBD  

---

## Summary

Remove the Issues feature from Chisel and replace it with a new **Specs** primitive — a structured, lifecycle-aware document type that sits between ephemeral issues and permanent docs. Simultaneously, refocus the marketing site away from a broad developer-tooling platform narrative toward a specific, sharp positioning aimed at solo technical founders and small teams building with LLM coding agents.

---

## Motivation

### Why Issues Isn't Working

Issues was built as a local-first Kanban board — a reasonable first attempt at handling the planning layer that docs doesn't cover. In practice it has the wrong shape for the problem:

- Issues are atomic and throwaway. They don't preserve the *why* behind decisions.
- Issues are task-oriented. They don't capture narrative, constraints, alternatives considered, or open questions.
- The gap between a closed issue and a living doc is where most organizational reasoning disappears.
- For a solo founder or small team, a Kanban board is process overhead that doesn't improve what an LLM can do with the output.

The issue tracker is the right tool for a model where humans are the primary executor of work. When LLM agents are doing significant implementation work, the unit of value shifts upstream to the artifact that gives the agent context — not the artifact that tracks human task completion.

### Why Specs Fills the Gap

A spec is the artifact that lives between an idea and a shipped feature. It has a lifecycle: rough draft → decision made → in progress → shipped/archived. Unlike an issue, it has narrative structure. Unlike a doc, it has state and is expected to evolve rapidly. Unlike both, it is the highest-value context artifact for an LLM mid-implementation.

The spec is also what engineers at senior/principal/staff level have always produced as their primary output — design docs, architecture decision records, technical RFCs. LLM agents compress the implementation layer, which means more of engineering's value moves to this planning and design tier. Chisel Specs is the infrastructure for that tier.

---

## Target Audience

### Primary: Solo Technical Founders

A solo founder using LLM coding agents (Claude Code, Cursor, Aider, etc.) as their primary implementation layer faces a specific acute problem: **their codebase grows faster than their ability to hold it in their head.** Six months in, they can't reliably remember why architectural decisions were made, what constraints existed at the time, or what the intended behavior of a system is.

This audience:
- Has strong personal motivation to maintain context artifacts (they are both author and consumer)
- Is already paying for multiple developer productivity tools
- Makes purchase decisions alone, with a short evaluation cycle
- Has no organizational politics or adoption overhead
- Will give direct, honest feedback when something isn't working

### Secondary: Small Technical Teams (2–5 engineers)

Small teams where one or two people can anchor a practice without organizational buy-in. Likely early-stage startups where the founding team is doing most engineering. The team version requires the product to demonstrate value to at least one person before asking others to change their workflow.

### Not (Yet): Mid-Sized Engineering Teams

Teams of 10+ with PMs, dedicated QA, and established tooling (Jira, Linear, Confluence) are not the target. The adoption overhead is too high and the value proposition requires behavior change across multiple roles. This is an expansion path, not the initial bet.

---

## The Spec Primitive

### What a Spec Is

A spec is a structured markdown document with a defined lifecycle and frontmatter schema. It lives in a `specs/` directory in the project, version-controlled alongside the codebase.

A spec captures:
- **Intent** — what are we building and why
- **Success criteria** — how will we know it worked
- **Constraints** — what are the technical and product boundaries
- **Alternatives considered** — what else was evaluated and why it was rejected
- **Open questions** — what is still unresolved
- **Implementation notes** — what changed during execution vs. the plan
- **Status** — where in its lifecycle the spec currently is

### Lifecycle States

```
draft → ready → in-progress → shipped → archived
```

- **draft** — being written, not ready to act on
- **ready** — decision made, ready for implementation
- **in-progress** — actively being built against
- **shipped** — implementation complete; spec reflects what actually shipped
- **archived** — superseded or abandoned, preserved for reference

### Frontmatter Schema

```yaml
---
title: "Feature name"
status: draft | ready | in-progress | shipped | archived
created: YYYY-MM-DD
updated: YYYY-MM-DD
area: auth | payments | infra | etc.
related_docs:
  - docs/architecture/auth.md
open_questions:
  - "Should we support OAuth before email/password?"
---
```

### Directory Structure

```
project/
├── specs/
│   ├── active/
│   │   ├── user-auth.md
│   │   └── payment-flow.md
│   ├── shipped/
│   │   └── onboarding-v1.md
│   └── archived/
│       └── csv-import.md
├── docs/
│   └── architecture/
└── README.md
```

Active specs are queryable by agents as high-priority context. Shipped specs are archived with their implementation notes intact — this is where organizational reasoning is preserved rather than lost.

---

## CLI Design

### New Commands

```bash
# Create a new spec from template
chisel spec new "user authentication"

# List specs, optionally filtered by status
chisel spec list
chisel spec list --status in-progress

# View a spec in the TUI
chisel spec view user-auth

# Move a spec through its lifecycle
chisel spec status user-auth in-progress

# Search across specs (FTS5)
chisel spec search "oauth"

# Machine mode output for LLM consumption
chisel spec list --machine --status in-progress
chisel spec view user-auth --machine
```

### Machine Mode Output

Machine mode for specs should emit the full spec content plus metadata in a format optimized for LLM context windows. Active specs (draft, ready, in-progress) should be surfaced as highest-priority context when an agent queries the knowledge base.

Example machine mode output:

```yaml
specs:
  - id: user-auth
    title: "User Authentication"
    status: in-progress
    area: auth
    created: 2026-03-01
    updated: 2026-03-28
    open_questions:
      - "Should we support OAuth before email/password?"
    content: |
      [full markdown content]
```

### Integration with `chisel docs`

Specs and docs are distinct primitives but share the same index. A spec in `shipped` status can be promoted to a permanent doc with:

```bash
chisel spec graduate user-auth
```

This moves the relevant content to `docs/`, strips the spec frontmatter, and archives the original spec with a reference to the permanent doc. The graduation path makes the spec-to-doc lifecycle explicit and ensures shipped specs don't silently become stale.

---

## `chisel init` Without an LLM

Chisel does not ship an LLM. Init should provide immediate value through static analysis and opinionated scaffolding:

### Static Analysis

- Parse `README.md` for project name, description, and existing structure
- Detect tech stack from `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, etc.
- Find and index any existing markdown files
- Detect existing `CLAUDE.md` or `.cursorrules` and import as context

### Scaffolded Output

Create a starter set of files even if empty:

```
docs/
  architecture.md    # Template: system overview, key decisions, tech stack
  decisions/         # ADR directory
specs/
  active/            # Empty, ready for first spec
```

The architecture template is pre-populated with whatever was derivable from static analysis — project name, detected tech stack, placeholder sections for the rest.

### BYOLLM Hook

```bash
chisel init --dump
```

Outputs a structured prompt describing everything chisel found in the repo, ready to pipe to any LLM:

```bash
chisel init --dump | claude -p "Generate starter architecture docs for this project"
```

The output of that conversation goes back into chisel as the initial docs. This makes the bring-your-own-LLM philosophy explicit and demonstrable in the first interaction.

---

## Default Templates and Conventions

Chisel should ship with opinionated defaults that teach good practice through structure, not documentation. The templates are the best practices guide.

### Feature Spec Template

```markdown
---
title: ""
status: draft
created: {{date}}
updated: {{date}}
area: ""
open_questions: []
---

## What and Why

<!-- What are we building? Why now? What problem does it solve? -->

## Success Criteria

<!-- How will we know this worked? What does good look like? -->

## Constraints

<!-- Technical constraints, product constraints, what's out of scope -->

## Approach

<!-- The plan. How will this be built? -->

## Alternatives Considered

<!-- What else was evaluated? Why was this approach chosen? -->

## Open Questions

<!-- What is still unresolved? Who can resolve it? -->

## Implementation Notes

<!-- Updated during/after implementation: what changed from the plan and why -->
```

### Architecture Decision Record Template

```markdown
---
title: ""
status: decided | superseded | deprecated
created: {{date}}
deciders: []
---

## Context

<!-- What situation prompted this decision? -->

## Decision

<!-- What was decided? -->

## Consequences

<!-- What becomes easier or harder as a result? -->
```

Templates are the guardrails. The structure teaches the practice without imposing process.

---

## What to Remove

### Chisel Issues

Remove Issues from the product entirely. This includes:
- The `chisel issues` command set
- The Kanban TUI
- The local YAML issue storage
- All references on the marketing site and documentation

Issues should be dropped cleanly rather than deprecated gradually. The thesis is that issues are the wrong primitive; keeping them alongside specs creates confusion about what to reach for.

**Migration path:** For any users currently using chisel issues, provide a one-time export command that converts open issues to draft specs:

```bash
chisel issues export-to-specs
```

### Chisel Observe (Deprioritize)

Observe is not wrong, but it is a distraction from the core positioning at this stage. Remove from the primary marketing narrative. Keep in the roadmap section if desired, but do not let it dilute the product story.

---

## Marketing Site Recommendations

### Current Problems

The current site sells the mechanism (terminal-native, local-first, Rust, 10ms latency) rather than the outcome. The comparison table positions against "legacy tools" — a developer-tooling frame that misses the actual pain. "Text-first tools for shaping information" means nothing to someone who just needs help staying the architect of their own project.

The roadmap section (Issues, Docs, Observe) makes chisel look like a platform play, which creates uncertainty about focus and raises the bar for adoption.

### Recommended Positioning

**Headline:**

> Your codebase grows faster than your memory. Chisel keeps you the architect.

Or alternatively:

> The project brain for founders building with AI agents.

**Subhead:**

> Structured specs and docs your LLM can actually use. Write once, query from anywhere.

### Revised Homepage Structure

**Section 1: The problem**  
Solo founders and small teams using LLM agents ship fast — faster than any human can track. Six months in, the codebase has outgrown the founder's ability to hold it in their head. Every new agent session starts from scratch. Decisions are made twice. Architecture drifts from intent.

**Section 2: The solution**  
Chisel is a local-first knowledge base for your project. Specs for active work. Docs for permanent knowledge. Both structured so your LLM tools can read them as context when they need it.

**Section 3: How it works**  
Short demo or animated terminal showing: creating a spec, moving it through lifecycle, querying it in machine mode, passing it to an agent.

**Section 4: The primitives**  
Two cards only — Docs and Specs. No Observe. No Issues. Each with a one-line description and the key workflow it enables.

**Section 5: Why terminal-native**  
The mechanism pitch lives here, not at the top. Local-first, version-controlled, structured for machine consumption. This is where the comparison table belongs if used at all.

### Messaging to Remove

- "Suite of terminal-native tools" — implies platform, raises adoption bar
- "Manage work" — this is the issues framing, drop it
- The three-column roadmap featuring Observe alongside Docs and Issues
- Any framing that competes with Jira or Linear — wrong audience, wrong battle

### Messaging to Add

- Explicit callout that it works with Claude Code, Cursor, and any shell-capable agent
- The BYOLLM philosophy stated plainly: chisel structures your knowledge, you bring the model
- A concrete example of the workflow: write spec → implement with agent → spec reflects what shipped
- `chisel init --dump` as the first interaction shown

---

## Open Questions

- Should specs support comment threads (async collaboration) or is that a team-tier feature?
- Does `chisel spec graduate` belong in v1 or is it premature until we see how users actually move from spec to doc?
- Is SQLite FTS5 sufficient for spec search, or does the spec layer warrant semantic/embedding-based retrieval sooner than docs does?
- What is the right behavior when a spec references a doc that doesn't exist yet?
- Should machine mode surface all active specs automatically, or require explicit opt-in per agent session?

---

## Success Criteria for This Change

- Solo founders can go from `chisel init` to first spec in under 10 minutes with a useful result
- LLM agent sessions can be bootstrapped with relevant spec context via a single machine mode command
- Shipped specs contain enough implementation notes that the reasoning is preserved and retrievable 6 months later
- The marketing site produces a meaningful increase in trial installs from the target audience segment
- Issues generates zero support or feature requests after removal (indicating no meaningful user loss)
