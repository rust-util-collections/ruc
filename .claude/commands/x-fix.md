---
description: Fix audit backlog — resolve every open finding in doc/audit.md, self-review, and commit
---

# Fix Audit Backlog

You are resolving every open finding in `doc/audit.md`, then self-reviewing and committing the result.

**How this differs from `/x-commit`:**
- `/x-commit` = "I've made changes — review them and commit." (starts from uncommitted diff)
- `/x-fix` = "Work through the audit backlog — fix, verify, commit." (starts from `doc/audit.md`)

## Setup

1. **MANDATORY**: Read `.claude/docs/technical-patterns.md` — bug pattern reference.
2. Read `.claude/docs/review-core.md` — review methodology.
3. Read `.claude/docs/false-positive-guide.md` — consult before reporting any finding.
4. Read `doc/audit.md` — this is your **primary work list**.

## Phase 1: Fix

### Task 1: Triage

1. Read `doc/audit.md`.
2. **Re-evaluate Won't Fix**: for each entry under `## Won't Fix`, re-read the
   code at the reported location against the **current** codebase. The decision
   was a snapshot — surrounding code may have changed, making a previously
   disproportionate fix straightforward, or the finding may no longer apply.
   Promote now-fixable entries to `## Open`; remove obsolete ones. Only carry
   forward entries whose original reasoning still holds.
3. If no `## Open` entries exist after re-evaluation, report "nothing to fix" and stop.
4. Sort open findings by severity: CRITICAL → HIGH → MEDIUM → LOW.
5. For each finding, read the code at the reported location with full context (100+ lines).
6. For each affected subsystem, read the corresponding pattern guide from `.claude/docs/patterns/`.

### Task 2: Fix

For each open finding, in severity order:

1. **Understand** the root cause — read the code, trace call sites, understand the invariant being violated.
2. **Implement** a complete fix. The fix must:
   - Fully resolve the finding — not a band-aid, not a workaround
   - Not introduce new issues (check boundary conditions, error paths, global state)
   - Respect the subsystem invariants (INV-*) — especially round-trip symmetry, nonce freshness, clamp direction, feature isolation
   - Follow project conventions (`.c(d!())` chains, `ensure!`, no `#[allow]`)
3. **Verify** the fix by reading the modified code and tracing its effects.
4. If the finding **cannot be fixed** (technical limitation, disproportionate risk, or breaking-change constraint better handled at the next major release), move it to `## Won't Fix` with a clear `**Reason**`.

### Task 3: Validate

1. After all fixes are applied, re-read every modified file to check for regressions.
2. Run `make fmt`, then `make lint` — must pass with zero warnings.
3. Run the tests as CI does: `make test` (or targeted
   `cargo test <module> --release -- --test-threads=1` when only one subsystem
   changed). If feature/cfg logic changed, also run
   `cargo check --no-default-features --features <X>` for touched features.

### Task 4: Update Audit Registry

1. Remove all fixed entries from `## Open`.
2. For entries moved to `## Won't Fix`, add the `**Reason**` field.
3. Write the updated `doc/audit.md`.

## Phase 2: Self-Review

1. Run `git diff HEAD` to see all changes from audit fixes.
2. If the diff is empty, report "nothing to commit" and stop.
3. Execute the `/x-review` Execution Protocol on the diff — invariant checks, boundary conditions, failure paths, global state.
4. Cross-reference every finding with `false-positive-guide.md`.
5. If the review produces **new findings**:
   - Fix them immediately.
   - Update `doc/audit.md`.
   - Repeat until `## Open` has zero entries (or only Won't Fix).

## Phase 3: Commit

Execute Tasks 3–5 of `.claude/commands/x-commit.md` (Format & Lint & Test →
Bump Patch Version → Commit). Key points:

1. `make fmt`, then `make lint` if any `.rs` file changed — must pass clean.
2. Bump patch version in `Cargo.toml` — mandatory if any `.rs` file changed.
3. Conventional commit style (`fix:` prefix typical for audit fixes), subject = the "why".
4. Stage specific files with `git add` (not `-A`), commit via HEREDOC —
   **no co-author line** — then `git status` to verify success.

## Output Format

```
## Audit Fix Summary

**Open before**: N findings
**Fixed**: X
**Won't Fix**: Y (moved with reasons)

### Self-Review
**New findings**: N (all resolved)

### Commit
**Commit**: <short hash> <subject line>
```
