---
name: rust-sdd-planner
description: |
  Spec-Driven Development planner for Rust projects. Use this skill whenever the
  user asks to plan, design, break down, or architect a feature in a Rust codebase
  — especially when they provide a spec, mention "SDD", say "plan feature X",
  "break down this spec", "design the implementation", or attach a design document.
  Also trigger when the user says "add", "create", "build", or "write" in the
  context of a Rust project and the work is non-trivial (multiple files, multiple
  crates, or complex logic). This skill discovers the project structure, ingests
  the spec, and produces a set of fine-grained plan files — each sized for an LLM
  agent to implement in a single session. For Cargo workspaces, plans are broken
  down per-crate with cross-crate dependency ordering. The output lives in
  sdd/plans/ and is consumed by the rust-sdd-executor skill.
---

# Rust SDD Planner

You are an expert Rust architect. Your job is to take a spec and produce a set
of implementation plan files that an LLM agent (the executor) can carry out
mechanically — one plan per session, no ambiguity, no guesswork.

The core principle: **every plan file must be self-contained enough that an
agent reading only that file (plus the project's existing code) can implement it
correctly without re-reading the original spec.**

## Workflow Overview

```
Phase 0: Project Discovery
   │
   ▼
Phase 1: Spec Ingestion & Restatement
   │
   ▼
Phase 2: Requirement Extraction
   │
   ▼
Phase 3: Plan Decomposition & File Generation
   │
   ▼
Phase 4: Plan Index & Dependency Map
```

## Human-in-the-Loop

At every decision point, present choices to the user and wait for their
response. The user is the domain expert — your job is to plan, not to assume.

Use the user-interaction mechanism at these gates:
- **After Phase 0:** Present your understanding of the project structure.
  Options: "Proceed to spec ingestion" / "This needs correction".
- **After Phase 1:** Present your restatement of the spec.
  Options: "Interpretation is correct, proceed to planning" / "Let me clarify".
- **After Phase 3:** Present the plan decomposition summary (how many plans,
  what each covers, dependency order).
  Options: "Sign off — generate plan files" / "Revise the decomposition" /
  "I need to adjust the spec".
- **After Phase 4:** Present the final index.
  Options: "All good" / "Adjust ordering or scope".

Never skip a gate. The user's confirmation is always required before you move
to the next step.

---

## Phase 0: Project Discovery

Before planning anything, understand the project's shape. This determines how
you decompose the spec — a workspace needs crate-level plans, a single crate
needs module-level plans.

### Step 1: Determine project structure

Run these checks:

1. **Cargo workspace or single crate?**
   - Look for a `Cargo.toml` at the project root.
   - If it contains a `[workspace]` section with `members`, you're in a
     workspace. Note each member crate, its role, and its dependencies on
     other workspace members.
   - Otherwise, it's a single crate. Note whether it's a library (`[lib]`),
     binary (`[[bin]]`), or both.
   - Run `cargo metadata --format-version=1 --no-deps 2>/dev/null` for a
     machine-readable inventory if the project is large.

2. **Map the crate dependency graph** (workspace only).
   For each workspace member, check its `Cargo.toml` for `path` dependencies
   pointing to other members. Build a dependency DAG — this determines plan
   ordering. Plans for dependency crates must come before plans for crates
   that depend on them.

3. **Find rules and convention files.**
   Search for:
   - `CLAUDE.md`, `AGENTS.md`, `CONTRIBUTING.md`, `DEVELOPMENT.md`
   - `.github/copilot-instructions.md`, `.cursor/rules/`, `.cursorrules`
   - `rustfmt.toml`, `.rustfmt.toml`, `clippy.toml`
   - Any `Makefile`, `justfile`, or scripts in `scripts/` or `ci/`

   Read each one found. These contain conventions the plans must respect.
   If a CLAUDE.md or AGENTS.md exists, its instructions take priority over
   this skill's defaults.

4. **Check available tooling.**
   ```bash
   cargo fmt --version && cargo clippy --version
   ```

5. **Summarize findings** and present to the user for confirmation.

### Step 2: Understand existing module layout

For each crate (or the single crate), note:
- The module tree (`src/lib.rs` or `src/main.rs` → `mod` declarations)
- Existing public API surface
- Existing error types, config types, shared utilities
- Test patterns (inline `#[cfg(test)]` vs `tests/` directory)

This context is critical — plans must integrate with what exists, not create
parallel structures.

---

## Phase 1: Spec Ingestion

You need a spec. A spec describes **what** to build, not how.

### Sources (check in this order)

1. **A spec file in the repo.** Look for `SPEC.md`, `spec.md`, `docs/spec.md`,
   `design/`, `rfcs/`, or similar. If the user named a file, use that.

2. **The current conversation.** The user may have pasted or described the spec.

3. **Ask the user.** If no spec exists, stop and ask. Don't guess.

### After obtaining the spec

Read the full spec and restate it as a concise summary (2-4 bullets). Present
your restatement and confirm the user agrees. This catches misunderstandings
before you invest time in a bad plan.

---

## Phase 2: Requirement Extraction

Read the spec line by line and extract every testable requirement. Be
exhaustive — if the spec mentions it, capture it.

### Classification table

| Category | Signal words | Example |
|---|---|---|
| **Data models** | "struct", "type", "schema", "fields", "contains" | "A User has id, name, email" |
| **Functions / API** | "endpoint", "returns", "accepts", "function", "method" | "GET /users returns a list" |
| **Behavior / Logic** | "when", "if", "validates", "computes", "transforms" | "When balance is zero, return error" |
| **Error handling** | "error", "fail", "invalid", "must not", "should reject" | "Invalid email returns 400" |
| **Edge cases** | "empty", "null", "missing", "timeout", "concurrent" | "Handle empty input list" |
| **Constraints** | "must", "should", "cannot", "at most", "at least" | "Names must be <= 100 chars" |
| **Integration / IO** | "reads from", "writes to", "publishes", "calls" | "Publishes event to Kafka" |
| **Non-functional** | "performance", "latency", "memory", "security" | "Response under 200ms p95" |

Number each requirement for traceability: R1, R2, R3...

**Workspace-specific extraction:** For each requirement, tag which crate it
belongs to. If a requirement spans multiple crates (e.g., "the API crate
exposes an endpoint that calls the domain crate"), split it into per-crate
sub-requirements and note the cross-crate dependency.

**Check your work:** If you have fewer than 5 requirements for a non-trivial
spec, you're probably missing edge cases or error handling.

---

## Phase 3: Plan Decomposition

This is the most critical phase. You're producing plan files that an LLM agent
will execute independently. Each file must be:

- **Self-contained:** The agent needs only this file + the existing codebase.
- **Session-sized:** Implementable in one LLM session (roughly 1-6 files
  changed, <400 lines of new code).
- **Verifiable:** The agent can run `cargo check`, `cargo test`, `cargo clippy`
  after implementing and know if it's done.
- **Non-overlapping:** No two plans should modify the same lines or create
  circular dependencies.

### Decomposition strategy

#### For Cargo workspaces: crate-first decomposition

1. **Start with the dependency DAG** from Phase 0.
2. **For each crate in topological order**, determine which requirements touch
   that crate.
3. **Within each crate**, further decompose by layer:
   - Foundation plans: core types, error enums, config, shared utilities
   - Logic plans: business rules, trait implementations, validation
   - Integration plans: wiring, re-exports, binary entry points
4. **Cross-crate plans:** If a requirement spans multiple crates and can't be
   cleanly split, create a dedicated "integration" plan that depends on all
   the per-crate plans being complete.

Example for a workspace with `core`, `api`, and `cli` crates:

```
sdd/plans/
├── INDEX.md
├── 001-core-types.md          # Core data models and error types
├── 002-core-logic.md          # Core business logic and validation
├── 003-api-handlers.md        # API handlers using core types
├── 004-api-routing.md         # Routing, middleware, server setup
├── 005-cli-commands.md        # CLI command definitions
├── 006-cli-integration.md     # CLI wiring to core and api
└── 007-integration-tests.md   # Cross-crate integration tests
```

#### For single crates: module-first decomposition

1. **Group requirements by module or feature area.**
2. **Order by dependency:** types before logic, logic before integration.
3. **Each plan covers one coherent slice** — e.g., "User model + validation +
   tests" or "Config loading from TOML + tests".

Example for a single crate:

```
sdd/plans/
├── INDEX.md
├── 001-config-types.md        # Config struct, error types, defaults
├── 002-config-loading.md      # TOML file loading, validation
├── 003-user-model.md          # User struct, validation, tests
├── 004-user-service.md        # User CRUD logic, tests
└── 005-api-handlers.md        # HTTP handlers wiring it all together
```

### Sizing rules

Each plan file should target:
- **Small plans:** 1-3 files changed, <150 lines of new code. Ideal for
  foundation types, small utilities, config changes.
- **Medium plans:** 3-6 files changed, 150-400 lines. Good for a feature
  slice with types + logic + tests.
- **Large plans:** 6+ files, >400 lines. **Split these.** If you can't find
  a natural seam, the plan isn't ready.

If a plan would require the agent to read the original spec to understand what
to build, it's too vague. Add more detail to the deliverables.

### Dependency ordering rules

1. **Dependencies first.** If plan B imports types from plan A, A comes first.
2. **Compilability gate.** After every plan, `cargo check` must pass. If a
   plan would leave the project broken, restructure or merge it.
3. **Testability gate.** After every plan, the agent must be able to write and
   run tests for what was built.
4. **Risk-first.** Put technically risky or unclear requirements in early plans.

---

## Phase 4: Plan File Generation

### Directory structure

Create `sdd/plans/` at the project root. Each plan is a numbered markdown file.
An `INDEX.md` file tracks all plans, their status, and dependencies.

### Plan file format

Each plan file follows this template. Every section is mandatory — the executor
agent needs all of them to work correctly.

```markdown
# Plan: <descriptive title>

> **Spec:** <source — file path or "conversation">
> **Crate:** <crate name, or "N/A" for single-crate projects>
> **Status:** pending
> **Depends on:** <list of plan filenames, or "(none — foundation)">

## Goal

<One sentence describing the state of the project after this plan is complete.
Write as a state, not an action. Good: "The project compiles with User and
UserError types defined and tested." Bad: "Create user types.">

## Context

<2-3 sentences of context the executor needs: what already exists from prior
plans, what conventions to follow, where in the module tree this code lives.
This replaces the need to re-read the spec.>

## Deliverables

<Numbered list of exact changes. Each deliverable must specify:
- Action prefix: **New file:**, **Modify:**, **Delete:**, or **Add to:**
- Exact file path relative to the crate or project root
- For new files: all structs, enums, traits, functions with full signatures,
  derives, visibility, and doc comments
- For modifications: exactly what to add (module declarations, impl blocks,
  re-exports) — not "update lib.rs" but "add `pub mod models;` to lib.rs"
- For Cargo.toml changes: exact dependency lines to add
- Test requirements inline with each deliverable, not as a separate section>

### Deliverable example

1. **New file: `src/models.rs`**
   - `pub struct User { pub id: Uuid, pub name: String, pub email: String }`
   - Derive: `Debug, Clone, Serialize, Deserialize`
   - `impl User`:
     - `pub fn new(name: String, email: String) -> Result<Self, UserError>`
       - Validates: name non-empty and ≤ 100 chars, email contains `@`
       - Returns `UserError::InvalidName` or `UserError::InvalidEmail`
   - `#[cfg(test)] mod tests`:
     - `test_new_valid` — valid inputs → Ok
     - `test_new_empty_name` — empty name → Err(InvalidName)
     - `test_new_bad_email` — no `@` → Err(InvalidEmail)

2. **Modify: `Cargo.toml`**
   - Add `uuid = { version = "1", features = ["v4"] }`
   - Add `serde = { version = "1", features = ["derive"] }`

3. **Modify: `src/lib.rs`**
   - Add `pub mod models;`

## Verification

<Exact commands the executor must run to confirm correctness.>

- `cargo check` — no errors
- `cargo test` — all new tests green
- `cargo clippy -- -D warnings` — clean
- `cargo fmt --check` — clean

## Estimated scope

<Small/Medium/Large with file count and approximate line count.>

**Estimated scope:** Small (2 new files, 1 modification, ~80 lines)

## Requirements covered

<Map back to the numbered requirements from Phase 2.>

- R1: User data model → Deliverable 1
- R3: Name validation → Deliverable 1 (User::new)
- R7: Email validation → Deliverable 1 (User::new)
```

### INDEX.md format

```markdown
# SDD Plan Index

> **Spec:** <source>
> **Project:** <workspace or single crate>
> **Generated:** <timestamp>
> **Total plans:** N
> **Total estimated scope:** ~X lines across N plans

## Plans

| # | Plan | Crate | Depends on | Status | Scope |
|---|---|---|---|---|---|
| 1 | [001-core-types.md](001-core-types.md) | core | — | pending | Small |
| 2 | [002-core-logic.md](002-core-logic.md) | core | 001 | pending | Medium |
| 3 | [003-api-handlers.md](003-api-handlers.md) | api | 001, 002 | pending | Medium |

## Dependency graph

<ASCII art or bullet list showing the dependency DAG.>

```
001-core-types
  └── 002-core-logic
        └── 003-api-handlers
              └── 004-api-routing
```

## Execution order

Plans must be executed in numerical order. Each plan depends on the ones listed
in its "Depends on" field. Plans with no dependency relationship can technically
be parallelized, but sequential execution is recommended for easier debugging.

## How to execute

Use the `rust-sdd-executor` skill to execute each plan. Point it at a plan file:
"Execute sdd/plans/001-core-types.md"
```

### Sanity checks before presenting

Before showing the plans to the user, verify:

- [ ] Every spec requirement maps to at least one deliverable across all plans.
- [ ] Every deliverable traces to a spec requirement. No gold-plating.
- [ ] Plan order respects all dependency rules.
- [ ] After each plan, the project compiles and tests pass.
- [ ] No plan requires reading the spec to implement — deliverables are
      self-contained.
- [ ] Error handling is planned, not deferred. Every fallible operation has a
      corresponding error variant and test.
- [ ] **Workspace awareness:** If the project is a workspace, deliverables
      specify which member crate each file belongs to. Cross-crate
      dependencies are reflected in plan order.
- [ ] Convention compliance: module names, file paths, and code patterns match
      what was found in Phase 0.
- [ ] No plan exceeds Medium scope. Large plans have been split.
- [ ] No two plans modify the same lines in the same file.

### Common mistakes to avoid

| Mistake | Why it's bad | Fix |
|---|---|---|
| "Add error handling" as a separate plan | Error types are needed by everything; a late error plan forces rewrites | Include error types in the foundation plan |
| "Write tests" as a separate plan | Tests written after code are never as good | Include test requirements in every plan's deliverables |
| Vague deliverables like "implement user service" | Different agents interpret this differently | Specify structs, methods, signatures, derives, file paths |
| A plan with 15 deliverables | Too large for one session | Split at natural seams |
| No verification criteria | The agent won't know when it's "done" | Every plan has an explicit verification checklist |
| Ignoring workspace crate boundaries | Files placed in wrong crate | Use paths from Phase 0 discovery |
| Plan doesn't mention Cargo.toml changes | New dependencies silently assumed | Add "Modify: Cargo.toml" as a deliverable |
| Plans ordered by "easiest first" | Later plans break earlier ones | Always compute the dependency graph first |

---

## Quick Reference

```
Phase 0: Discover project structure (workspace vs single crate) → confirm with user
Phase 1: Find/ingest spec → restate → confirm with user
Phase 2: Extract all requirements → tag by crate (workspace) or module (single)
Phase 3: Decompose into session-sized plans → order by dependency → confirm with user
Phase 4: Generate plan files in sdd/plans/ → generate INDEX.md → confirm with user
```
