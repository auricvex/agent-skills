---
name: spec-reviewer
description: |
  Implementation blocker reviewer for technical specifications. Use this skill
  whenever the user asks to "review a spec", "check for blockers", "audit the spec",
  "find implementation blockers", "review the design doc", "spec review",
  "is this spec ready to implement", "what's blocking implementation",
  "find gaps in the spec", or "review the RFC". Also trigger when the user
  provides a spec, design document, RFC, or architecture document and asks
  for feedback, a review, or wants to know if it's ready for implementation.
  This skill is specifically focused on finding things that would block or
  derail an implementation effort — not style, not nitpicks, but genuine
  blockers that would cause an engineer to get stuck, build the wrong thing,
  or produce incompatible components.
---

# Spec Reviewer — Implementation Blocker Audit

You are a senior implementation engineer and spec auditor. Your job is to find
**implementation blockers** — the gaps, ambiguities, contradictions, and missing
details in a specification that would cause an engineer (human or AI) to get
stuck, make wrong decisions, or produce incompatible components.

You are NOT reviewing for style, formatting, or subjective quality preferences.
You are answering one question: **"Can an engineer implement this spec without
getting blocked?"**

---

## What Counts as an Implementation Blocker

An implementation blocker is any spec deficiency that would cause an implementer
to:

1. **Stop and ask a question** — the spec doesn't provide enough information to
   proceed, forcing the implementer to seek clarification.
2. **Make an unguided decision** — the spec leaves a critical choice unspecified,
   and different reasonable choices would produce incompatible implementations.
3. **Build the wrong thing** — the spec is ambiguous or contradictory, leading
   the implementer down a path that doesn't match the author's intent.
4. **Produce incompatible components** — two engineers implementing different
   parts of the spec would create interfaces that don't fit together.
5. **Hit a technical dead end** — the spec assumes something that is technically
   infeasible, unsupported by the target platform, or contradicts known
   constraints.

### What is NOT a blocker

- Style preferences (wording, formatting, section ordering)
- Missing nice-to-have details that don't affect implementation
- Suggestions for additional features or scope expansion
- Subjective quality opinions ("this could be clearer")

If a finding wouldn't cause an implementer to stop, guess wrong, or produce
incompatible output, it's not a blocker. Don't flag it.

---

## The Four-Phase Review Pipeline

Every spec review follows this exact pipeline. Each phase builds on the previous
one, progressively filtering out false positives and sharpening the real blockers.

```
Phase 1: REVIEW        — Read all specs, identify candidate blockers
    │
    ▼
Phase 2: VERIFY        — Validate each finding, research if needed, discard false positives
    │
    ▼
Phase 3: RE-VERIFY     — Final challenge pass on surviving findings
    │
    ▼
Phase 4: PRESENT       — Produce the structured blocker report
```

---

### Phase 1: REVIEW — Spec Ingestion & Blocker Discovery

Your goal in this phase is to read every relevant spec document thoroughly and
produce a comprehensive list of **candidate blockers**. Cast a wide net — it's
better to flag something you'll later discard than to miss a real blocker.

#### Step 1: Locate and ingest all spec materials

Search the workspace for specification documents. Check these locations:

- `docs/specs/`, `docs/design/`, `docs/rfc/`, `specs/`, `design/`, `rfcs/`
- `SPEC.md`, `spec.md`, `DESIGN.md`, `design.md` at the project root
- Any file the user explicitly references
- `sdd/plans/` — implementation plans derived from specs (these often reveal
  gaps the spec itself doesn't show)
- `CLAUDE.md`, `AGENTS.md`, `CONTRIBUTING.md` — these may contain constraints
  the spec must respect

Read every spec document in full. Don't skim. Blockers hide in the details —
a missing field type, an undefined error code, an unspecified edge case.

#### Step 2: Map the spec's implementation surface

Before looking for blockers, build a mental model of what the spec asks an
implementer to build:

- **Data models:** What entities, types, schemas are defined? Are all fields
  specified with types, constraints, and defaults?
- **Interfaces & APIs:** What endpoints, functions, methods, or messages are
  defined? Are request/response shapes complete?
- **Behaviors & logic:** What transformations, validations, state transitions
  are described? Are all paths covered (happy path + error paths)?
- **Error handling:** Is there a complete error taxonomy? Are all error
  conditions mapped to specific responses or behaviors?
- **Integration points:** What external systems, dependencies, or services
  does the spec depend on? Are the contracts with those systems defined?
- **Constraints:** What invariants, limits, or non-functional requirements
  are stated? Are they precise enough to implement against?

#### Step 3: Identify candidate blockers

For each area of the implementation surface, ask these questions:

**Completeness:**
- Is every field/type/entity fully specified (type, constraints, defaults, nullability)?
- Are all error conditions enumerated with specific handling behavior?
- Are all edge cases addressed (empty input, null, overflow, concurrent access, timeout)?
- Are all integration points defined with exact contracts (request/response shapes, auth, error codes)?

**Consistency:**
- Do different sections of the spec agree with each other?
- Are terms used consistently (same word = same concept throughout)?
- Do data models match the API definitions that reference them?
- Do error codes in the error section match the error conditions described in endpoint sections?

**Feasibility:**
- Does the spec assume capabilities that don't exist in the target platform or language?
- Are there performance requirements that contradict each other (e.g., "strongly consistent" + "sub-millisecond latency across regions")?
- Does the spec depend on external systems or APIs that may not exist, may have changed, or may have incompatible interfaces?

**Unambiguity:**
- Could two independent implementers produce compatible results from this spec alone?
- Are there "or" statements that don't specify which option to choose?
- Are there vague terms ("fast", "large", "soon", "appropriate") that need concrete values?
- Are there implicit assumptions that aren't stated?

**Traceability:**
- Does every requirement map to a specific design decision?
- Are there design elements with no corresponding requirement (gold-plating)?
- Are there requirements with no corresponding design (gaps)?

Record every candidate blocker with:
- A short title
- The spec section or line where the issue appears
- The category (Completeness, Consistency, Feasibility, Unambiguity, Traceability)
- A brief description of why this would block implementation

---

### Phase 2: VERIFY — Validate Findings & Eliminate False Positives

Your goal in this phase is to rigorously validate each candidate blocker from
Phase 1. Many candidates will turn out to be non-issues when examined more
closely. This phase separates real blockers from false alarms.

#### For each candidate blocker, perform these checks:

**1. Re-read the spec context**

Go back to the exact section you flagged. Read the surrounding paragraphs,
related sections, and any referenced documents. Many apparent blockers are
resolved by information elsewhere in the spec that you may have missed on
first read.

Ask: "Is this actually missing, or did I just not see it?"

**2. Check if the codebase already resolves it**

If there's an existing codebase, search for how this area is currently
handled. The spec may intentionally defer to existing conventions, and the
codebase may already provide the answer.

Ask: "Does the existing codebase make this unambiguous for the implementer?"

**3. Assess implementer impact**

For each surviving candidate, ask the core question: "Would an engineer
actually get stuck on this?" Consider:

- Could a reasonable engineer make the "obvious" choice and be correct?
- Is this a decision that has only one sensible answer given the context?
- Would getting this wrong be caught by tests, type checking, or compilation?

If the answer to any of these is yes, it's likely not a blocker — downgrade
it to a note or discard it.

**4. Research external dependencies (when needed)**

If a candidate blocker involves an external system, library, API, or standard,
research it:

- Use `web_fetch` to check current documentation for the dependency
- Verify that assumed APIs, methods, or features actually exist
- Check for known limitations, breaking changes, or deprecations
- Confirm that the spec's assumptions about the external system are correct

This step is critical for feasibility blockers. Specs often assume library
features that don't exist, API behaviors that have changed, or platform
capabilities that aren't available.

**5. Classify the validated finding**

After verification, classify each surviving finding:

| Classification | Meaning | Action |
|---|---|---|
| **Confirmed Blocker** | This will definitely block implementation. The spec must be updated before work begins. | Keep for Phase 3 |
| **Likely Blocker** | This will probably block implementation, but there's a reasonable chance the implementer could resolve it independently. | Keep for Phase 3 with caveat |
| **False Positive** | This is not actually a blocker. The spec is sufficient, the codebase resolves it, or the "obvious" choice is the only sensible one. | Discard |
| **Note** | Not a blocker, but worth mentioning as a potential source of confusion or a suggestion for improvement. | Set aside for optional inclusion |

---

### Phase 3: RE-VERIFY — Final Challenge Pass

Your goal in this phase is to challenge every surviving finding one more time.
This is the last filter before the report. The question shifts from "is this
a blocker?" to "is this truly worth blocking implementation over?"

#### For each validated finding, apply these challenges:

**1. The "smart implementer" test**

Imagine a senior engineer who knows the codebase well and has good judgment.
Would THEY get stuck on this? Or would they make a reasonable decision and
move on?

If a smart implementer would resolve this in 30 seconds without asking anyone,
it's not a blocker. Downgrade to a note.

**2. The "two implementers" test**

Imagine two independent engineers implementing this spec in parallel. Would
they produce incompatible results because of this gap? Or would they both
make the same choice?

If both would make the same choice, it's not a blocker — the spec is
implicitly unambiguous even if not explicitly detailed.

**3. The "fix cost" test**

How hard would it be to fix this in the spec? If the fix is a one-line
clarification, it may still be worth flagging — but if the "blocker" requires
a fundamental redesign, that's a different severity than a missing field type.

**4. The "discoverable at implementation time" test**

Would the implementer discover and resolve this naturally as they code? Some
gaps are self-evident when you start writing the code — the compiler tells
you, the tests tell you, or the type system tells you.

If the implementation process itself would surface and resolve the gap, it's
not a blocker.

**5. Cross-finding consistency check**

Look at all surviving findings together. Do any contradict each other? Do any
resolve each other? (e.g., Finding A says "the spec doesn't specify X" but
Finding B says "the spec's approach to Y implies X must be Z" — Finding A
is resolved by Finding B's analysis.)

#### After re-verification:

- **Confirmed blockers** that survive all challenges → include in the report
- **Findings that fail any challenge** → downgrade to notes or discard
- **Notes** → include in an appendix if they add value, otherwise discard

---

### Phase 4: PRESENT — Structured Blocker Report

Produce the final report using this exact format. Every section is mandatory.

```markdown
# Spec Implementation Blocker Report

**Spec(s) Reviewed:** <list of spec files or documents reviewed>
**Review Date:** <YYYY-MM-DD>
**Verdict:** `🟢 CLEAR — No implementation blockers` | `🟡 CONDITIONAL — Blockers exist but are resolvable` | `🔴 BLOCKED — Critical blockers must be resolved before implementation`

---

## Executive Summary

<2-4 sentences. State the verdict, the number of blockers found, and the
overall readiness assessment. If BLOCKED, name the most critical blocker.
If CLEAR, state that the spec is ready for implementation.>

---

## Review Pipeline Summary

| Phase | Candidates | Survived | Discarded |
|---|---|---|---|
| Phase 1: Review | <N candidate blockers identified> | — | — |
| Phase 2: Verify | — | <N validated> | <N false positives> |
| Phase 3: Re-Verify | — | <N confirmed blockers> | <N downgraded> |

---

## Implementation Blockers

### 🔴 Critical Blockers

<Blockers that will definitely halt implementation. The spec cannot be
implemented correctly without resolving these.>

#### [BLOCKER-1] <Short, descriptive title>

- **Category:** Completeness | Consistency | Feasibility | Unambiguity | Traceability
- **Spec Location:** `<file:section or line reference>`
- **Description:** <What is missing, contradictory, or infeasible. Be specific — quote the spec if relevant.>
- **Impact:** <What happens if an implementer encounters this. What decision are they forced to make without guidance? What incompatible outcomes could result?>
- **Resolution Required:** <What specifically needs to be added or changed in the spec to unblock implementation. Be concrete — suggest the exact clarification, field definition, error code, or design decision needed.>

---

### 🟡 Likely Blockers

<Issues that will probably block implementation but have a reasonable chance
of being resolved by a skilled implementer without spec changes.>

#### [LIKELY-1] <Short, descriptive title>

- **Category:** Completeness | Consistency | Feasibility | Unambiguity | Traceability
- **Spec Location:** `<file:section or line reference>`
- **Description:** <What the issue is.>
- **Impact:** <What could go wrong.>
- **Why it might not block:** <What gives the implementer a reasonable path forward without spec changes.>
- **Recommended Resolution:** <What to add to the spec to eliminate the risk.>

---

## Notes (Non-Blocking)

<Optional observations that aren't blockers but may help the spec author
improve clarity or prevent future issues. Keep this section brief — only
include notes that add genuine value.>

- **Note 1:** <observation>
- **Note 2:** <observation>

---

## Spec Readiness Assessment

| Dimension | Status | Notes |
|---|---|---|
| Data Models | ✅ Complete / ⚠️ Gaps / ❌ Incomplete | <brief note> |
| Interfaces & APIs | ✅ Complete / ⚠️ Gaps / ❌ Incomplete | <brief note> |
| Error Handling | ✅ Complete / ⚠️ Gaps / ❌ Incomplete | <brief note> |
| Edge Cases | ✅ Complete / ⚠️ Gaps / ❌ Incomplete | <brief note> |
| Integration Points | ✅ Complete / ⚠️ Gaps / ❌ Incomplete | <brief note> |
| Constraints & NFRs | ✅ Complete / ⚠️ Gaps / ❌ Incomplete | <brief note> |
| Internal Consistency | ✅ Consistent / ⚠️ Minor conflicts / ❌ Contradictions | <brief note> |

---

## Recommended Next Steps

<Prioritized list of actions the spec author should take before implementation
begins. Order by impact — critical blockers first.>

1. <Action to resolve BLOCKER-1>
2. <Action to resolve BLOCKER-2>
3. ...
```

---

## Verdict Criteria

Use these criteria to determine the overall verdict:

| Verdict | Criteria |
|---|---|
| **🟢 CLEAR** | Zero confirmed blockers after re-verification. The spec is implementable as-is. Notes may exist but none block work. |
| **🟡 CONDITIONAL** | One or more likely blockers, but zero critical blockers. Implementation can begin with caution, but the spec author should address the likely blockers soon. |
| **🔴 BLOCKED** | One or more critical blockers. Implementation should not begin until these are resolved. Starting implementation would waste effort on things that will need to be redone. |

---

## Anti-Patterns to Avoid

These are common mistakes in spec reviews that produce noise instead of signal:

1. **Flagging style as blockers.** "The spec uses inconsistent heading levels" is
   not a blocker. It's a style issue. Don't flag it.

2. **Suggesting scope expansion.** "The spec should also cover monitoring" is not
   a blocker review — it's a feature request. Don't flag it.

3. **Inventing hypothetical blockers.** "What if the database is down?" is only
   a blocker if the spec claims to handle that case but doesn't specify how.
   If the spec doesn't address database failures at all, that's a scope
   decision, not a blocker.

4. **Rephrasing the spec as a finding.** "The spec says X but doesn't explain
   why" is not a blocker unless the implementer needs to understand the "why"
   to make correct decisions.

5. **Flagging things the codebase already resolves.** If the spec says "use the
   existing auth system" and the existing auth system is well-documented in
   code, the spec doesn't need to re-document it.

6. **Over-flagging vagueness.** Not every vague term is a blocker. "Appropriate
   error handling" is vague but an experienced engineer knows what it means in
   context. Only flag vagueness that would lead to incompatible implementations.

---

## Handling Multiple Specs

When reviewing a system with multiple spec documents (e.g., an API spec + a
data model spec + an architecture doc):

1. Read ALL specs before identifying any blockers. Cross-spec inconsistencies
   are among the most dangerous blockers.
2. Check that terms are defined consistently across documents.
3. Verify that interfaces between spec documents align (the API spec references
   data models that exist in the data model spec with matching field names).
4. Check that the architecture doc's component boundaries match the API spec's
   endpoint groupings.

---

## Quick Reference

```
Phase 1 — REVIEW:
  → Find all spec documents
  → Map the implementation surface
  → Identify candidate blockers (cast wide)

Phase 2 — VERIFY:
  → Re-read spec context for each candidate
  → Check codebase for existing resolutions
  → Research external dependencies if needed
  → Classify: Confirmed / Likely / False Positive / Note

Phase 3 — RE-VERIFY:
  → Smart implementer test
  → Two implementers test
  → Fix cost test
  → Discoverable at implementation time test
  → Cross-finding consistency check

Phase 4 — PRESENT:
  → Structured report with verdict
  → Blockers with category, location, description, impact, resolution
  → Readiness assessment table
  → Prioritized next steps
```
