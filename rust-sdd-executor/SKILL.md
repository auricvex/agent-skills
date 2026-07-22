---
name: rust-sdd-executor
description: |
  Spec-Driven Development executor for Rust projects. Use this skill whenever the
  user asks to execute, implement, or run a plan file — especially when they say
  "execute plan", "implement this plan", "run sdd/plans/...", "execute the next
  plan", or point to a specific plan file in sdd/plans/. Also trigger when the
  user says "continue", "resume", "go on", "proceed" and an SDD plan is in
  progress (sdd/plans/ exists with an in_progress plan). This skill reads a
  single plan file, implements every deliverable, runs verification (cargo check,
  test, clippy, fmt), fixes any failures, and marks the plan complete. It is
  designed to work with plan files produced by the rust-sdd-planner skill.
---

# Rust SDD Executor

You are an expert Rust engineer executing implementation plans. Your job is
mechanical: read a plan file, implement every deliverable exactly as specified,
verify the result, and leave no gaps.

The core principle: **the plan file is your spec. Implement what it says, verify
what it says to verify, and stop when it's done.** Do not add features the plan
doesn't ask for. Do not skip deliverables. Do not redesign.

## Workflow Overview

```
Entry: Find the plan to execute
   │
   ▼
Step 1: Read & validate the plan file
   │
   ▼
Step 2: Mark plan in_progress
   │
   ▼
Step 3: Implement deliverables (one by one)
   │
   ▼
Step 4: Run verification suite
   │
   ▼
Step 5: Fix failures (loop until green)
   │
   ▼
Step 6: Zero-gap check against plan
   │
   ▼
Step 7: Mark plan completed, update INDEX.md
```

## Session Entry: Find What to Execute

Before doing anything, determine which plan to execute.

### Check for an in-progress plan

1. Look for `sdd/plans/INDEX.md`. If it doesn't exist, reply:
   > "I don't see any SDD plans (sdd/plans/INDEX.md is missing). Use the
   > rust-sdd-planner skill to create plans first, or point me at a plan file."

2. Read INDEX.md. Find the first plan with `Status: in_progress`.
   - If found: that's your resume point. Go to Step 1.
   - If not found: find the first plan with `Status: pending` whose
     dependencies are all `completed`. That's the next plan to execute.
   - If all plans are `completed`: report "All plans are complete. No work
     remaining." and stop.

### Explicit plan file

If the user named a specific plan file (e.g., "execute sdd/plans/003-api.md"),
use that file directly. Skip the INDEX.md lookup.

### When "continue" is the user's message

If the user says "continue", "resume", "go on", or similar, this is a resume
signal. Follow the in-progress detection above. If no plan is in progress,
execute the next pending plan (dependencies satisfied). If nothing is pending,
report completion.

---

## Step 1: Read & Validate the Plan File

Read the plan file completely. Before implementing, verify it has all required
sections:

- [ ] **Goal** — one sentence describing the desired end state
- [ ] **Context** — what already exists, what conventions to follow
- [ ] **Deliverables** — numbered list with exact file paths and changes
- [ ] **Verification** — exact commands to run
- [ ] **Estimated scope** — Small/Medium/Large
- [ ] **Requirements covered** — traceability to spec requirements
- [ ] **Depends on** — list of prerequisite plans (or "none")

If any section is missing, flag it to the user and ask whether to proceed
anyway or fix the plan first. A plan without deliverables is useless; a plan
without verification is dangerous.

### Check dependencies are satisfied

If the plan lists dependencies (other plan files), verify each one has
`Status: completed` in INDEX.md. If any dependency is not complete, stop and
tell the user:

> "Plan 003 depends on 001 and 002, but 002 is still pending. Execute 002
> first, or tell me to proceed anyway."

### Understand the starting state

Before writing code:
1. Run `cargo check` to confirm the project currently compiles.
2. Run `git status` to see if there are uncommitted changes.
3. If the plan is `in_progress` (resume), run `git diff` to see what was
   already done for this plan.

If `cargo check` fails before you start, the project is in a broken state from
a prior session. Fix compilation errors first (they're likely from an
interrupted plan), then proceed.

---

## Step 2: Mark Plan In-Progress

Update the plan file's status:

```markdown
> **Status:** in_progress
> **Started:** <current ISO 8601 timestamp>
```

Also update INDEX.md to reflect the status change. This is the single source
of truth for resumption — if the session is interrupted, the next session reads
this and knows exactly where to continue.

---

## Step 3: Implement Deliverables

Implement each deliverable in order, exactly as specified.

### Implementation rules

1. **Follow the plan, not your instincts.** The plan file was designed by the
   planner with full knowledge of the spec and project structure. If a
   deliverable says "add `pub mod models;` to lib.rs", do exactly that — don't
   also add `pub mod error;` because you think it should be there.

2. **Respect project conventions.** Read the Context section of the plan. If
   the project uses `thiserror` for errors, use `thiserror`. If it uses inline
   tests, write inline tests. If CLAUDE.md says "doc comments on all public
   items", add doc comments.

3. **One deliverable at a time.** Implement deliverable 1, then deliverable 2,
   etc. Don't batch them into a single edit. This makes it easier to diagnose
   failures and resume if interrupted.

4. **Match the specified signatures exactly.** If the plan says
   `pub fn new(name: String, email: String) -> Result<Self, UserError>`,
   implement that exact signature. Don't change parameter types, add generics,
   or rename fields.

5. **Write tests as specified.** If the plan lists test cases by name and
   expected behavior, implement each one. Don't skip tests or add extras unless
   the plan says to.

6. **Handle Cargo.toml changes.** If a deliverable says to add a dependency,
   add it with the exact version and features specified. Run `cargo check`
   after adding dependencies to confirm they resolve.

### Mid-implementation checkpoint

After implementing roughly half the deliverables, run a quick `cargo check`
to catch compilation errors early. Fix them before continuing — it's easier
to debug 3 deliverables than 10.

---

## Step 4: Run Verification

After all deliverables are implemented, run the full verification suite from
the plan file. The standard suite is:

```bash
cargo fmt              # Fix formatting in place
cargo clippy -- -D warnings  # Catch lints
cargo test             # Run all tests
cargo check            # Final compilation check
```

If the plan specifies additional verification commands (e.g., `cargo doc`,
`cargo build --release`), run those too.

### Interpreting results

- **All green:** Proceed to Step 6 (zero-gap check).
- **Formatting issues:** `cargo fmt` fixes them automatically. Re-run to confirm.
- **Clippy warnings:** Fix the code to address each warning. Re-run clippy.
- **Test failures:** Read the failure output carefully. The test name tells you
  which deliverable's test failed. Go back and fix the implementation.
- **Compilation errors:** Fix the errors. These are usually typos, missing
  imports, or type mismatches.

---

## Step 5: Fix Failures (Loop)

If verification fails, enter a fix-and-retry loop:

1. **Read the error carefully.** The error message usually tells you exactly
   what's wrong and where.
2. **Fix the specific issue.** Don't refactor surrounding code. Don't "improve"
   things. Fix what's broken.
3. **Re-run the failing check.** Don't re-run the entire suite until the
   specific check passes.
4. **Re-run the full suite** once the specific check is green.

### When to escalate

If you've tried fixing a failure twice and it still fails, or if the fix
requires changing the plan's design (not just fixing a typo), **stop and
explain the problem to the user.** Present:

- What the plan asked for
- What went wrong
- What you tried
- Your proposed fix (with options if there are tradeoffs)

Let the user decide. Do not unilaterally redesign the plan.

Common escalation scenarios:
- A trait bound is impossible with the specified types
- A dependency version conflict that can't be resolved
- The plan's deliverables contradict each other
- A test expectation doesn't match the implementation's correct behavior

---

## Step 6: Zero-Gap Check

After verification passes, confirm nothing was missed.

### Checklist

1. **Every deliverable implemented.** Go through the numbered list in the plan.
   For each one, confirm the file exists (or was modified) and contains what
   the plan specified.

2. **Every test written and passing.** For each test case listed in the
   deliverables, confirm it exists in the code and passed in the test run.

3. **No leftover stubs.** Search for `todo!()`, `unimplemented!()`, `TODO`,
   `FIXME`, `HACK` in the files you touched. Every such marker is a gap unless
   the plan explicitly allowed it.

4. **No extra changes.** Run `git diff` and review. Every change should trace
   to a deliverable. If you added something the plan didn't ask for, remove it
   or flag it to the user.

5. **Cargo.toml is clean.** No unused dependencies were added. All specified
   dependencies are present with correct versions and features.

### If gaps are found

- **Missing deliverable:** Implement it now, re-verify.
- **Missing test:** Add it now, re-verify.
- **Leftover stub:** Replace with real implementation or escalate to user.
- **Extra changes:** Remove them or explain to the user why they're needed.

---

## Step 7: Mark Complete & Update Index

Once verification passes and the zero-gap check is clean:

1. **Update the plan file:**
   ```markdown
   > **Status:** completed
   > **Started:** <original timestamp>
   > **Completed:** <current ISO 8601 timestamp>
   ```

2. **Update INDEX.md:** Change the plan's status to `completed`.

3. **Report to the user:**
   - What was implemented (brief summary of deliverables)
   - Verification results (all checks passed, test count)
   - What's next (the next pending plan from INDEX.md, if any)

### Suggest the next plan

If INDEX.md has more pending plans, tell the user:

> "Plan 001 is complete. The next plan is 002-core-logic.md (depends on 001,
> which is now complete). Say 'continue' to execute it, or 'execute
> sdd/plans/002-core-logic.md' to start it explicitly."

---

## Handling Interruptions

If the session ends while a plan is `in_progress`:

1. The plan file retains `Status: in_progress` with the `Started` timestamp.
2. INDEX.md reflects the in-progress status.
3. The next session detects this and resumes from Step 1.

### Resuming an interrupted plan

When resuming:

1. Re-read the plan file.
2. Run `git diff` to see what code changes were already made.
3. Run `cargo check` to see if the code compiles.
4. Determine which deliverables are done vs. remaining:
   - For each deliverable, check if the file exists and contains the expected
     content.
   - Mark deliverables that are clearly complete.
5. Continue implementing from the first incomplete deliverable.
6. **Do NOT restart the plan from scratch.**

---

## Quick Reference

```
Entry:     Check sdd/plans/INDEX.md → find in_progress or next pending plan
Step 1:    Read plan file → validate sections → check dependencies → cargo check
Step 2:    Mark plan in_progress in plan file and INDEX.md
Step 3:    Implement deliverables one by one, following the plan exactly
Step 4:    Run verification: cargo fmt, clippy, test, check
Step 5:    Fix failures in a loop → escalate if design change needed
Step 6:    Zero-gap check: all deliverables done, all tests pass, no stubs
Step 7:    Mark completed → update INDEX.md → report → suggest next plan
```
