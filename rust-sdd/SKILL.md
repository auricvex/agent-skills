---
name: rust-sdd
description: |
  Spec-Driven Development for Rust projects. Use this skill whenever the user
  asks to implement a feature, build something, or make changes in a Rust
  codebase — especially when they provide a spec, mention "SDD", say "implement
  feature X", or attach a design document. Also trigger when the user says
  "add", "create", "build", or "write" in the context of a Rust project, or
  when they say "continue", "resume", "go on", "proceed" mid-workflow.
  This skill enforces a structured workflow: discover the project → ingest the
  spec → produce a phased plan → implement phase by phase with verification →
  final review for zero gaps. It ensures every Rust change is traceable to a
  spec requirement and passes clippy/rustfmt/tests. The workflow is designed to
  survive session interruptions — when the user says "continue", the skill
  detects in-progress state via sdd/plan.md and resumes exactly where it left
  off.
---

# Rust Spec-Driven Development (SDD)

You are an expert Rust engineer following Spec-Driven Development. The core
principle is simple: **every line of code must trace back to a spec
requirement, and every spec requirement must be implemented.** Nothing slips
through the cracks.

This workflow spans multiple sessions. When the user says "continue" or
"resume" (or the conversation restarts mid-implementation), pick up exactly
where you left off — do not restart from Phase 0.

## Human-in-the-Loop: Gate at Every Phase

This workflow is designed for collaboration, not automation. **At every
decision point, use the `AskUserQuestion` tool to present choices to the user
and wait for their response before proceeding.** The user is the domain
expert — your job is to execute, not to assume. Never make a decision the
user should make.

Use `AskUserQuestion` at these gates (all are mandatory):
- **After Phase 0 (discovery):** Present your understanding of the project
  structure, conventions, and tooling. Options: "Proceed to spec ingestion" /
  "This needs correction". If the user says their understanding differs,
  ask for the correction and re-present.
- **After Phase 1 (spec restatement):** Present your restatement of the spec.
  Options: "Interpretation is correct, proceed to planning" / "Let me clarify".
- **After Phase 2 (plan creation):** Present the phased plan summary.
  Options: "Sign off — implement now" / "Revise the plan" / "I need to adjust
  the spec". This is the most expensive gate to get wrong — the user must
  explicitly approve before any code is written.
- **Between each phase in Phase 3:** Report what was completed and what's
  next with options: "Continue to next phase" / "Pause here" / "Adjust plan".
  The user may want to inspect changes, reorder phases, or stop.
- **On any design issue or verification failure:** Present the problem,
  explain options, and use `AskUserQuestion` to let the user choose the path
  forward. Do not unilaterally redesign or paper over problems.
- **After Phase 4 (gap report):** Present findings with options: "Accept gaps
  as-is" / "Add phases to address gaps" / "File follow-up issues" / "All
  clear, mark complete".

Never skip a gate. The `AskUserQuestion` response is the only valid
confirmation — do not proceed without it. The user's confirmation is always
required before you move to the next step.

## Session Entry: Detect and Resume

Before entering any phase, check whether an SDD session is already in
progress. This determines whether you start fresh or resume.

### Check for an existing plan

Look for `sdd/plan.md` at the project root. If it exists:
- Read it. Find the first phase with `Status: in_progress` or `Status: pending`.
- If there's an `in_progress` phase, that's where you resume. Look at the
  phase's deliverables and the state of the working tree (`git status`, `git
  diff`) to understand what's been done vs. what's left.
- If all phases are `pending`, start from Phase 2 (you already did Phase 0
  and Phase 1 previously) — the spec and plan are already set.
- If all phases are `completed`, go straight to Phase 4 (zero-gap review).
- Skip Phase 0/1/2 when resuming — the project structure, spec, and plan are
  already known and documented.

If `sdd/plan.md` does NOT exist, proceed from Phase 0 as normal.

### When "continue" is the user's message

If the user's message is just "continue", "resume", "go on", or similar,
this is a resume signal. Follow the session entry logic above. If
`sdd/plan.md` doesn't exist, reply: "I don't see an in-progress SDD session
(sdd/plan.md is missing). What would you like me to work on?"

## Phase 0: Project Discovery

Before touching any code, understand the project's shape. This isn't busywork —
it tells you where new code belongs, what conventions to follow, and what guard
rails already exist.

### Determine the project structure

Run these checks (use shell commands or file reads — whatever is fastest):

1. **Cargo workspace or single crate?**
   - Look for a `Cargo.toml` at the project root.
   - If it contains a `[workspace]` section with `members`, you're in a
     workspace. Note each member crate and its role.
   - Otherwise, it's a single crate. Note whether it's a library (`[lib]`),
     binary (`[[bin]]`), or both.
   - Run `cargo metadata --format-version=1 --no-deps 2>/dev/null` for a quick
     machine-readable inventory if the project is large.

2. **Find rules and convention files.**
   Search for these files (they may not all exist — that's fine):
   - `CLAUDE.md`, `AGENTS.md`, `CONTRIBUTING.md`, `DEVELOPMENT.md`
   - `.github/copilot-instructions.md`, `.cursor/rules/`, `.cursorrules`
   - `rustfmt.toml`, `.rustfmt.toml`, `clippy.toml`
   - Any `Makefile`, `justfile`, or scripts in `scripts/` or `ci/`

   Each one you find contains conventions the project expects you to follow.
   Read them and internalize the rules before writing code. If a CLAUDE.md or
   AGENTS.md exists, its instructions take priority over this skill's defaults.

3. **Check available tooling.**
   ```bash
   cargo fmt --version && cargo clippy --version
   ```
   If these fail, note that verification steps will need adjustment. You can
   still run `cargo check` and `cargo test` as a fallback.

4. **Summarize what you found** in a short bullet list and use
   `AskUserQuestion` to present it for confirmation. Give the user options:
   "Proceed to spec ingestion" / "This needs correction". The user may correct
   your understanding of the crate layout, conventions, or tooling setup.
   Do not move to Phase 1 until you receive the `AskUserQuestion` response.

## Phase 1: Spec Ingestion

You need a spec. A spec describes **what** to build, not how — it covers
behavior, interfaces, edge cases, and acceptance criteria. Without one, you're
guessing.

### Sources (check in this order)

1. **A spec file in the repo.** Look for `SPEC.md`, `spec.md`, `docs/spec.md`,
   `design/`, `rfcs/`, or similar. If the user named a file explicitly, use
   that.

2. **The current conversation.** The user may have pasted or described the spec
   in their message. Treat their description as the spec source.

3. **Ask the user.** If no spec exists anywhere, stop and ask. Don't guess what
   they want. Say something like:

   > "I don't see a spec file in the project, and I don't have a spec in the
   > conversation. Could you provide one? A spec should describe the feature's
   > behavior, inputs/outputs, edge cases, and acceptance criteria. You can
   > paste it here or point me to a file."

   Wait for the spec before proceeding. Do not proceed with "what I think you
   mean" — that's how gaps happen.

### After obtaining the spec

Read the full spec and restate it in your own words as a concise summary
(2-4 bullets). Use `AskUserQuestion` to present your restatement and confirm
the user agrees with your interpretation. Options: "Interpretation is
correct, proceed to planning" / "Let me clarify". This forces you to actually
understand it and gives the user a chance to catch misunderstandings before
you invest time in a bad plan. Do not proceed to Phase 2 (planning) until
you receive the `AskUserQuestion` response.

## Phase 2: Spec Breakdown → Phased Plan

This is the most critical phase of SDD. A bad plan produces bad code no matter
how carefully you implement. A good plan makes implementation mechanical.
**Spend real time here.** Do not rush to code.

The output is `sdd/plan.md` — a phased, ordered set of implementation steps,
each self-contained and verifiable. The plan must be detailed enough that
another agent could pick it up and implement it without reading the original
spec.

### Step 0: Before You Begin — Gather What You Know

You already have from Phase 0 and Phase 1:
- The project structure (workspace members, crate type, module tree)
- The project conventions (from CLAUDE.md, rustfmt.toml, etc.)
- The full spec (restated and confirmed by the user)

Keep all three visible while you plan. Every phase must respect the project's
existing module layout and conventions.

### Step 1: Extract All Requirements from the Spec

Read the spec line by line and extract every testable requirement. Be
exhaustive — if the spec mentions it, capture it. Use this classification:

| Category | Signal words | Example |
|---|---|---|
| **Data models** | "struct", "type", "schema", "fields", "contains", "represents" | "A User has an id, name, and email" |
| **Functions / API** | "endpoint", "returns", "accepts", "function", "method", "exposes" | "GET /users returns a paginated list" |
| **Behavior / Logic** | "when", "if", "validates", "computes", "calculates", "transforms" | "When the balance is zero, return InsufficientFunds" |
| **Error handling** | "error", "fail", "invalid", "must not", "should reject" | "Invalid email format returns 400" |
| **Edge cases** | "empty", "null", "missing", "timeout", "concurrent", "large" | "Handle empty input list gracefully" |
| **Constraints** | "must", "should", "cannot", "at most", "at least", "no more than" | "Names must be <= 100 characters" |
| **Integration / IO** | "reads from", "writes to", "publishes", "subscribes", "calls" | "Publishes an OrderPlaced event to Kafka" |
| **Non-functional** | "performance", "latency", "throughput", "memory", "security" | "Response under 200ms p95" |

Write these down in a scratch list. Number them for traceability (R1, R2,
R3...). You'll use these numbers in the gap report later.

**Check your work:** Count the requirements. If you have fewer than 5 for a
non-trivial spec, you're probably missing edge cases or error handling.

### Step 2: Identify the Dependency Skeleton

Before grouping into phases, map what depends on what. This determines phase
order. Draw a mental (or literal) dependency graph:

1. **What data types must exist first?** These are leaf nodes — nothing depends
   on them, they depend on nothing. List them.

2. **What traits or abstractions sit between data and logic?** If the spec
   mentions "a storage backend" or "a notifier", you need a trait before the
   implementation.

3. **What functions consume the data types?** These are mid-level nodes. They
   depend on (1) and sometimes (2).

4. **What ties everything together?** This is the integration layer — binary
   entry points, API handlers, main orchestrator. Depends on everything above.

5. **What is cross-cutting?** Error types, logging, config — they're used
   everywhere. Build them early, before anything that uses them.

**Rust-specific dependency patterns to recognize:**

| Pattern | Dependency rule |
|---|---|
| `struct Foo { bar: Bar }` | `Bar` must exist first |
| `impl Trait for Foo` | Both `Trait` and `Foo` must exist |
| `fn do(foo: &Foo)` | `Foo` must exist first |
| `fn handle(req: Request) -> Response` | Both types must exist first |
| `mod foo;` declaration | The module file can be stubbed early |
| Workspace crate dependency (`foo = { path = "../foo" }`) | The dependency crate must be ready first |
| `#[cfg(test)] mod tests` | Can be written alongside or after the code they test |
| Derive macros (`#[derive(Debug, Clone, Serialize)]`) | Add derives in the same phase as the struct |

### Step 3: Group Requirements into Candidate Phases

Take your requirement list and cluster related items. Use these grouping
heuristics:

**Primary grouping — by layer (most common):**
- **Foundation phase(s):** Core data types, error enum, config struct. The
  types that everything else will import.
- **Logic phase(s):** Functions, trait implementations, validation, business
  rules. The code that does the work.
- **Integration phase(s):** Binary entry points, API handlers, wiring,
  `main()` or `lib.rs` re-exports. The code that ties it all together.

**Secondary grouping — by concern:**
- If the spec covers multiple features or domains, group by domain. A spec
  about "user profiles + order history" should not mix user structs with order
  structs in the same phase.
- If the spec has clear CRUD operations, each entity's full stack (type →
  logic → API) can be its own phase.

**Tertiary grouping — by risk:**
- Put technically risky or unclear requirements in early phases so you
  discover problems while they're cheap to fix. Don't save the hard part for
  last.
- Put "nice to have" or purely additive features in later phases.

### Step 4: Order the Phases

Now sort your candidate phases into a linear order. Apply these rules, in
priority order:

1. **Dependencies first.** If phase B imports types from phase A, A comes
   first. No exceptions.

2. **Compilability gate.** After every phase, `cargo check` must pass. If a
   phase would leave the project in a broken state, restructure it or merge it
   with the phase that provides the missing pieces. A good test: "Can I
   `cargo check` after this phase without stubbing anything?"

3. **Testability gate.** After every phase, you must be able to write and run
   tests for what you just built. If the code can't be tested yet (e.g., it's
   a handler with no request types), the phase is incomplete.

4. **Value-first ordering.** Among independent phases, implement the most
   valuable / most representative slice first. If phase 2 and phase 3 have no
   dependency relationship, implement the one the user cares about more first.

5. **Risk-first tiebreaker.** When value is equal, implement the riskier
   phase first (new library, complex algorithm, unfamiliar pattern).

### Step 5: Detail Each Phase — Write the Plan File

This is the most important step. A vague phase produces vague code. For each
phase, specify exactly what the agent must produce. If you can't be specific,
the phase isn't ready.

For each phase, write:

#### Phase header
```markdown
### Phase N: <descriptive title>
```
The title should describe the outcome, not the activity. Good: "User data model
and error types". Bad: "Add some structs".

#### Goal (one sentence)
What does the world look like after this phase? Write it as a state, not an
action. Good: "The project compiles with `User` and `UserError` types defined
and documented." Bad: "Create user types."

#### Dependencies
```markdown
**Depends on:** Phase 1 (UserError is used here)
```
Make it explicit. If this is the first phase: `**Depends on:** (none — foundation)`.

#### Deliverables (the critical part)

Each deliverable must be concrete enough that an agent can implement it without
re-reading the spec. For each deliverable, specify:

```markdown
**Deliverables:**

1. **New file: `src/models.rs`** (or `crates/core/src/models.rs` in a workspace)
   - `struct User { id: Uuid, name: String, email: String, created_at: DateTime<Utc> }`
   - Derive: `Debug, Clone, Serialize, Deserialize`
   - `impl User { pub fn new(name: String, email: String) -> Result<Self, UserError> }`
     - Validates: name non-empty and ≤ 100 chars, email contains `@`
     - Returns `UserError::InvalidName` or `UserError::InvalidEmail` on failure

2. **New file: `src/error.rs`**
   - `enum UserError { InvalidName(String), InvalidEmail(String), NotFound(Uuid) }`
   - Derive: `Debug, Clone, thiserror::Error` (or `Display + Error` manually if no deps)
   - `impl From<UserError> for std::io::Error` — stub only

3. **Modify: `src/lib.rs`**
   - Add `pub mod models;` and `pub mod error;`
   - No other changes

4. **New file: `src/models.rs` — `#[cfg(test)] mod tests`**
   - Test: `User::new` with valid inputs → succeeds
   - Test: `User::new` with empty name → `Err(UserError::InvalidName)`
   - Test: `User::new` with name > 100 chars → `Err(UserError::InvalidName)`
   - Test: `User::new` with no `@` → `Err(UserError::InvalidEmail)`
```

**Deliverable format rules:**
- Prefix each with the action: **New file:**, **Modify:**, **Delete:**, or **Add to:**
- For new files, give the exact path relative to the project or crate root.
- For modifications, list the specific additions (not "update lib.rs" — say
  exactly what module declarations, re-exports, or impl blocks to add).
- List all derives, trait implementations, and public API items.
- Include test requirements inline — don't leave testing as an afterthought.
- If the module path depends on project structure (workspace member), use
  the path discovered in Phase 0.

#### Verification
```markdown
**Verification:**
- `cargo check` passes with no errors
- `cargo test` passes — all new tests green
- `cargo clippy -- -D warnings` passes
- `cargo fmt --check` passes
```

If a phase can't be verified (e.g., it only adds a module stub), say so and
explain why.

#### Estimated scope
```markdown
**Estimated scope:** Small (2 new files, 1 modification, ~80 lines of code)
```

Use: **Small** (1-3 files, <150 lines), **Medium** (3-6 files, 150-400 lines),
**Large** (6+ files, >400 lines — consider splitting). This helps the user
judge whether to proceed.

### Step 6: Add Resume Points for Large Plans

If the full plan has more than ~5 phases or any Large phases, insert explicit
resume markers:

```markdown
> **[RESUME POINT]** — The user may stop here and say "continue" to resume
> at Phase 3. Phases 1-2 are complete and compilable.
```

Put these after every 2-3 phases, or after any phase that represents a natural
milestone (e.g., "Core types done, now the API layer").

### Step 7: Sanity-Check the Plan Before Presenting

Run these checks internally before showing the plan to the user:

- [ ] **Every spec requirement maps to at least one deliverable.** Scan your
  R1, R2, R3... list. Each one has a home.
- [ ] **Every deliverable traces to a spec requirement.** If a deliverable
  isn't motivated by the spec, cut it. No gold-plating.
- [ ] **The phase order respects all dependency rules from Step 4.**
- [ ] **After each phase, the project compiles and tests pass.** Walk through
  phase by phase in your head.
- [ ] **No phase requires reading the spec to implement.** The deliverables
  are self-contained instructions.
- [ ] **Error handling is planned, not deferred.** Every fallible operation in
  the spec has a corresponding error variant and test.
- [ ] **Workspace awareness.** If the project is a workspace, deliverables
  specify which member crate each file belongs to. Cross-crate dependencies
  are reflected in phase order.
- [ ] **Convention compliance.** Module names, file paths, and code patterns
  match what you found in Phase 0.

### Common Mistakes — Check for These

| Mistake | Why it's bad | Fix |
|---|---|---|
| "Add error handling" as a separate phase | Error types are needed by everything; a late error phase forces rewrites | Include error types in Phase 1 |
| "Write tests" as a separate phase | Tests written weeks after code are never as good | Include test requirements in every phase's deliverables |
| Vague deliverables like "implement user service" | Different agents will interpret this differently → inconsistent code | Specify structs, methods, signatures, derives, file paths |
| A phase with 15 deliverables | Too large to implement in one turn; if it fails, you lose all progress | Split at natural seams (data vs logic, core vs extensions) |
| No verification criteria specified | The agent won't know when it's "done" → over- or under-implementation | Every phase has an explicit verification checklist |
| Ignoring workspace crate boundaries | Files placed in wrong crate, cross-crate deps not declared in Cargo.toml | Use paths from Phase 0 discovery |
| Plan doesn't mention `Cargo.toml` changes | New dependencies (serde, thiserror, uuid) silently assumed | Add "Modify: Cargo.toml — add dependency X vY" as a deliverable |
| Phases ordered by "easiest first" instead of "dependencies first" | Later phases break earlier ones when real dependencies are discovered | Always compute the dependency graph before ordering |

### Worked Example: Plan Fragment

Here is what a good plan fragment looks like for a spec that says:

> "Add a `Config` struct loaded from a TOML file. Config has `port: u16`
> (default 8080) and `database_url: String` (required). Missing file returns a
> descriptive error. Invalid port (<1024 or >65535) is rejected at parse time."

```markdown
# SDD Plan: Config Loading
> Derived from: conversation (user spec)

## Phases

### Phase 1: Config data model and error types
**Goal:** The project compiles with `Config` struct and `ConfigError` enum
defined, serializable, and documented.

**Depends on:** (none — foundation)

**Deliverables:**

1. **New file: `src/config.rs`**
   - `struct Config { port: u16, database_url: String }`
   - Derive: `Debug, Clone, Serialize, Deserialize`
   - `impl Default for Config` — port = 8080, database_url = "" (sentinel)
   - `impl Config`:
     - `pub fn validate(&self) -> Result<(), ConfigError>` — rejects port < 1024
       or > 65535, rejects empty database_url

2. **New file: `src/config.rs` (same file, bottom) — `#[cfg(test)] mod tests`**
   - Test: default config has port 8080
   - Test: validate accepts port 8080 with non-empty url
   - Test: validate rejects port 80 (below 1024)
   - Test: validate rejects port 70000 (above 65535)
   - Test: validate rejects empty database_url

3. **New file: `src/error.rs`** (or add to existing error module)
   - `enum ConfigError { FileNotFound(PathBuf), ParseError(String), ValidationError(String) }`
   - Derive: `Debug, Clone, thiserror::Error`
   - `impl Display` with user-friendly messages

4. **Modify: `Cargo.toml`**
   - Add `serde = { version = "1", features = ["derive"] }`
   - Add `toml = "0.8"`
   - Add `thiserror = "1"`

5. **Modify: `src/lib.rs`** (or `src/main.rs`)
   - Add `pub mod config;`
   - Add `pub mod error;` (if new)

**Verification:**
- `cargo check` compiles
- `cargo test` — all 5 config tests pass
- `cargo clippy -- -D warnings` clean
- `cargo fmt --check` clean

**Estimated scope:** Small (2 new files, 1 modification, ~90 lines)

### Phase 2: TOML file loading
**Goal:** `Config::load(path)` reads a TOML file, deserializes, validates,
and returns a `Config` or a specific `ConfigError`.

**Depends on:** Phase 1 (uses `Config` and `ConfigError`)

**Deliverables:**

1. **Add to: `src/config.rs` — `impl Config`**
   - `pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError>`
     - Reads file at path → `ConfigError::FileNotFound` if missing
     - Parses TOML → `ConfigError::ParseError` with underlying message
     - Calls `self.validate()` → `ConfigError::ValidationError` on failure
     - Returns validated Config on success

2. **Add to: `src/config.rs` — `#[cfg(test)] mod tests`**
   - Test: load valid TOML file → succeeds with correct values
   - Test: load nonexistent file → `Err(ConfigError::FileNotFound)`
   - Test: load file with invalid TOML syntax → `Err(ConfigError::ParseError)`
   - Test: load file with port=0 → `Err(ConfigError::ValidationError)`
   - Test: load file missing database_url → `Err(ConfigError::ValidationError)`
   (Use `tempfile` or inline `#[cfg(test)]` helper to create temp TOML files)

3. **Modify: `Cargo.toml`** — add `tempfile = "3"` as dev-dependency

**Verification:**
- `cargo test` — all 5 new + all 5 old tests pass (10 total)
- `cargo clippy` and `cargo fmt` clean

**Estimated scope:** Small (0 new files, 2 additions, ~70 lines)
```

Notice what makes this good:
- Every deliverable has exact file paths and symbol names.
- Test cases are specified by name and expected behavior.
- Dependencies (Cargo.toml changes) are explicit.
- Phase 1 is self-contained and fully testable before Phase 2 exists.
- The agent reading Phase 2 knows exactly what methods to add to existing types.
- Verification is concrete, not "make sure it works."

### Present the Plan and Get Sign-Off

Show the complete plan to the user. Format it clearly (the markdown in
`sdd/plan.md` is for the file; in conversation you can summarize). Tell the
user:

- How many phases, and the estimated total scope.
- Which phases are Small/Medium/Large.
- Where the resume points are.
- Whether the project will compile after each phase.

Then use `AskUserQuestion` to present the plan summary and ask for sign-off.
Options: "Sign off — implement the plan now" / "Revise the plan" / "I need
to adjust the spec". Do not implement anything until you receive the
`AskUserQuestion` response with explicit sign-off. If they want changes,
revise and re-present. This is the last cheap moment to catch mistakes.

## Phase 3: Phased Implementation

Implement one phase at a time. Each phase is a mini development cycle:
implement → verify → mark complete.

### For each phase

1. **Mark the phase `in_progress`** in `sdd/plan.md`. Add a `Started:` line
   with the current timestamp. This is the single source of truth for
   resumption — if the session is interrupted, the next session reads this
   and knows exactly where to continue.

   ```markdown
   ### Phase 2: ...
   **Status:** in_progress
   **Started:** 2025-01-15T14:30:00Z
   ```

2. **Implement the changes.** Write the code, following the conventions you
   discovered in Phase 0. Keep edits focused on what the phase demands — no
   opportunistic refactors unless the plan says so.

3. **Verify immediately:**
   ```bash
   cargo fmt --check    # or cargo fmt
   cargo clippy -- -D warnings
   cargo test
   cargo check          # as a lightweight fallback if clippy is unavailable
   ```
   Fix all warnings and errors before moving on. A phase is not "done" with
   failing checks. If a check fails, fix the code, re-run, and repeat until
   everything is green.

4. **Mark the phase `completed`** in `sdd/plan.md` and add a `Completed:`
   timestamp. This makes progress visible across sessions.

   ```markdown
   ### Phase 2: ...
   **Status:** completed
   **Started:** 2025-01-15T14:30:00Z
   **Completed:** 2025-01-15T14:45:00Z
   ```

5. **Briefly report** to the user: what you implemented, what passed, what the
   next phase will cover.

### Mid-phase interruption

If the session ends while a phase is `in_progress` (you hit a token limit,
the user interrupts, or the conversation restarts), the next session will
detect `sdd/plan.md` and find the `in_progress` phase. When resuming:

1. Re-read `sdd/plan.md` and the spec.
2. Run `git diff` to see what code changes were already made for this phase.
3. Run `cargo check` to see if the code compiles.
4. Continue implementing from where you left off — do NOT restart the phase.
5. Run the full verification suite when you're done.

### If a phase fails verification

- If it's a small issue (a clippy lint, a formatting nit), fix it and re-run.
  No need to ask the user for trivial fixes.
- If it's a design issue (the approach doesn't work, a trait bound is
  impossible, a library is incompatible), **pause and explain the problem to the
  user. Use `AskUserQuestion` to present your proposed adjustment and let the
  user choose** from options like "Accept this change to the plan" / "Try a
  different approach (describe it)" / "Pause — I need to think about this".
  The user may have context you don't or may prefer a different tradeoff. Do not
  unilaterally redesign the plan without the user's input.
- If verification reveals a gap between what the spec asked for and what the
  plan covers, **use `AskUserQuestion` to flag it** with options: "Update the
  plan to cover the gap" / "Adjust the spec to match what was built".
- Never silently paper over a failure.

### Between phases

- Do a quick `git diff` or review your changes. Do they actually implement what
  the phase said they would? No extras, no omissions.
- Use `AskUserQuestion` to present a summary of what was completed and what
  the next phase covers. Options: "Continue to next phase" / "Pause here" /
  "Adjust the plan". Do not proceed to the next phase until you receive the
  `AskUserQuestion` response.
- If you spot something in the next phase that now looks wrong based on what you
  just built, update the plan before implementing it — and flag the change to
  the user.

## Phase 4: Zero-Gap Review

After all phases are `completed`, verify that nothing fell through the cracks.

### What to check

1. **Spec-to-implementation traceability.** For each requirement in the spec,
   point to the code that satisfies it. If the spec says "the API must return
   404 for missing resources", find the line where that 404 is emitted. Go
   requirement by requirement — no hand-waving.

2. **Implementation-to-spec traceability.** For each significant piece of code
   you wrote, point to the spec requirement that motivated it. If you find code
   that doesn't trace to any requirement, flag it — it's either unnecessary or
   the spec is incomplete.

3. **No leftover TODOs or placeholder code.** grep for `TODO`, `FIXME`,
   `HACK`, `unimplemented!`, `todo!`. Every such marker is a gap unless
   explicitly allowed by the plan.

4. **Test coverage for spec requirements.** Each acceptance criterion in the
   spec should have at least one test. If something isn't tested, note it.

### Gap report

Write a brief gap report to `sdd/review.md`:

```markdown
# SDD Review: <feature name>
> Completed: <timestamp>

## Spec Coverage
| Requirement | Implementation | Test |
|---|---|---|
| <req> | <file:line> | <file:line or "missing"> |

## Gaps Found
- <gap description> — <recommended action>

## Summary
- Requirements: X implemented, Y missing, Z untested
- Overall: <pass/fail with caveats>
```

Be honest. If there are gaps, say so — the user would rather know now than find
out from a production bug.

### After the gap report

- **Use `AskUserQuestion` to present the gap report and ask for a decision.**
  If gaps were found (missing requirements, untested criteria, leftover TODOs),
  offer these options:
  - "Accept gaps as intentional" (e.g., a requirement was deferred).
  - "Add a new phase to address specific gaps".
  - "File the gaps as follow-up issues".
- **If all requirements are covered and no gaps exist**, use `AskUserQuestion`
  to ask for final approval: "All clear, mark feature complete" / "I want to
  inspect something first".
- Do not proceed to final cleanup until you receive the `AskUserQuestion`
  response.

### Final cleanup

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

One last pass to make sure everything is green.

## Quick Reference

```
Session Entry: Check sdd/plan.md → resume if found, else start fresh
Phase 0: Discover project → AskUserQuestion to confirm understanding
Phase 1: Find/ask for spec → restate it → AskUserQuestion to confirm
Phase 2: Break spec into phased plan → write sdd/plan.md → AskUserQuestion for sign-off
Phase 3: Per phase: implement → verify → report → AskUserQuestion before next phase
Phase 4: Zero-gap review → write sdd/review.md → AskUserQuestion for decision → final cleanup
```
