---
description: Deep regression review of ruc changes (latest commit, N commits, hash, range, or full audit)
argument-hint: "[N | all | <hash> | <hash1>..<hash2>]"
---

# Deep Regression Analysis for RUC

You are performing a deep code review of changes to ruc, a Rust utility
library (error chains, command/SSH execution, crypto, encode/decode).

## Setup

1. **MANDATORY**: Read `.claude/docs/technical-patterns.md` first — the bug pattern reference (patterns marked HISTORICAL actually happened here).
2. Read `.claude/docs/review-core.md` — the review methodology.
3. Read `.claude/docs/false-positive-guide.md` — consult before reporting any finding.

## Input

Arguments: `$ARGUMENTS`

Parse the arguments to determine review scope:

| Input | Scope | How |
|-------|-------|-----|
| *(empty)* | Latest commit | `git diff HEAD~1`, `git log -1` |
| `N` (integer) | Last N commits | `git diff HEAD~N`, `git log -N --oneline` |
| `all` | Full codebase audit | Read all source files by subsystem (see Full Audit Protocol below) |
| `<commit hash>` | Specific commit | `git diff <hash>~1 <hash>` |
| `<hash1>..<hash2>` | Commit range | `git diff <hash1> <hash2>` |

For diff-based reviews (everything except `all`), proceed to the Execution Protocol.
For `all`, skip to the **Full Audit Protocol** at the end of this document.

## Execution Protocol

### Task 1: Context & Classification

1. Read the full diff carefully
2. Identify ALL affected subsystems using the **subsystem mapping table in
   `review-core.md` Phase 1** (single source of truth)
3. For EACH affected subsystem, read its pattern guide from `.claude/docs/patterns/`
   (the table lists which guide covers which subsystem — skip guides for unaffected subsystems)
4. Classify each change per the review-core methodology (Phase 2 table)

### Task 2: Deep Regression Analysis

For each HIGH or CRITICAL classified change:

1. **Read the surrounding code** — at least 50 lines of context around each change; never review from the diff alone
2. **Trace call sites** — grep for all callers of changed functions, including doc examples and `#[cfg(test)]` blocks
3. **Check invariants** — verify each INV-* from the loaded pattern guides and review-core Phase 3.1
4. **Boundary conditions** — check edge cases from review-core Phase 3.2 (empty input, short prefixes, timeout 0/max, length off-by-one)
5. **Failure paths** — analyze error handling per review-core Phase 3.3 (child processes killed? fds released? secrets not echoed?)
6. **Global state** — verify per review-core Phase 3.4 (LazyLock read-once, serial-test safety)

For each finding:
- Cross-reference `technical-patterns.md` — which pattern does it match?
- Cross-reference `false-positive-guide.md` — is this a known false positive?
- Only report with **concrete evidence**

### Task 3: Cross-Cutting Analysis

Check every change for (review-core Phase 4):
1. **Security** — nonce freshness, injection surface, secret leakage, RNG seeding
2. **Feature graph** — compiles alone (`cargo check --no-default-features --features X`)? cfg polarity correct? leaf ∈ group ∈ full?
3. **API compatibility** — observable behavior changes? deprecation policy honored? semver implication?

### Task 4: Code Style Enforcement

Check changed files against review-core Phase 4.4:
1. No new `#[allow(...)]` — `#![deny(warnings)]` is the law
2. Imports grouped (std → external → crate), common prefixes merged
3. Exported macros use `$crate::` and support trailing commas
4. **Doc-code alignment** — public API changes must update: doc comments,
   `README.md`, `doc/*.md`, `CLAUDE.md` (layout/features/env tables),
   `review-core.md` Phase 1 mapping table, and the affected pattern guide

### Task 5: Audit Registry (doc/audit.md)

After completing the analysis:

1. Read `doc/audit.md` from the project root (create if absent).
2. **Prune**: for each entry under `## Open`, verify against the current codebase; remove entries that are 100% fixed.
3. **Merge**: add new findings under `## Open`, deduplicating; sort by severity (CRITICAL → HIGH → MEDIUM → LOW).
4. **Re-evaluate Won't Fix**: for each entry under `## Won't Fix`, re-read the
   code at the reported location against the **current** codebase. The label is
   a snapshot judgment — code may have changed. For each entry:
   - Original reason still holds → leave in place.
   - Now fixable with reasonable effort → promote to `## Open` with updated assessment.
   - No longer applicable → remove entirely.
   Never silently carry forward a Won't Fix entry without fresh evaluation.
5. Write the updated `doc/audit.md`.

The file format:

```markdown
# Audit Findings

> Auto-managed by /x-review and /x-fix.

## Open

### [SEVERITY] subsystem: one-line summary
- **Where**: file:line_range
- **What**: description
- **Why**: invariant/pattern violated
- **Suggested fix**: how to fix

---

## Won't Fix

### [SEVERITY] subsystem: one-line summary
- **Where**: file:line_range
- **What**: description
- **Reason**: why this cannot or should not be fixed
```

## Output Format

```
## Review Summary

**Commit**: <hash> <subject>
**Subsystems**: <list of affected subsystems>
**Risk Level**: CRITICAL / HIGH / MEDIUM / LOW

## Findings

### [SEVERITY] subsystem: one-line summary

**Where**: file:line_range
**What**: Description
**Why**: Invariant/pattern violated (cite technical-patterns.md or a pattern guide INV-*)
**Fix**: Suggested fix or questions

---

(repeat for each finding)

## No Issues Found

(list areas checked where no issues were found, to demonstrate coverage)
```

If zero findings after full analysis, report:
```
## Review Summary
**Result**: LGTM — no regressions found
**Coverage**: <list of subsystems and invariants checked>
```

---

## Full Audit Protocol (for `all` mode)

When `$ARGUMENTS` is `all`, perform a full codebase audit.

### Strategy: Parallel Subsystem Audit

Launch **one Agent per pattern guide** in parallel — 4 agents total:

| Agent | Files | Guide |
|-------|-------|-------|
| err-common | `src/lib.rs`, `src/err/`, `src/common.rs`, `Cargo.toml` features | `patterns/err-common.md` |
| exec | `src/cmd.rs`, `src/ssh.rs`, `src/uau.rs`, `src/http.rs` | `patterns/exec.md` |
| algo | `src/algo/` (all) | `patterns/algo.md` |
| ende | `src/ende/` (all) | `patterns/ende.md` |

Agents are **stateless** — each prompt must be self-contained and include:
1. The exact file list for the subsystem
2. The corresponding pattern guide path (instruct the agent to read it)
3. Instructions to read `technical-patterns.md` and `false-positive-guide.md`
4. The code style rules from Task 4, and the finding output format above

### Aggregation

After all agents complete:
1. Collect all findings
2. Deduplicate cross-subsystem findings
3. Sort by severity: CRITICAL → HIGH → MEDIUM → LOW
4. Run Task 5 (audit registry) on the merged results
5. Output a unified audit report:

```
## Full Audit Report

**Scope**: All source files
**Subsystems Audited**: <list>
**Total Findings**: N (X critical, Y high, Z medium, W low)

## Findings

(sorted by severity, grouped by subsystem)

## Clean Areas

(subsystems with no findings — list what was checked)
```
