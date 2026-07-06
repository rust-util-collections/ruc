---
description: Deep regression review of ruc changes (latest commit, N commits, hash, range, or full audit)
argument-hint: "[N | all | <hash> | <hash1>..<hash2>] [--fix]"
---

# Deep Regression Analysis for RUC

You are performing a deep code review of changes to RUC, a Rust utility library
(error chains, command/SSH execution, crypto, encode/decode).
This review combines RUC-specific pattern analysis with Claude Code's multi-agent review architecture.

**Design philosophy reminder**: ruc is a lightweight, practical toolkit — not a showcase of
maximal rigor. Do not push toward over-formalization. Keep APIs simple and pragmatic.

## Setup

1. **MANDATORY**: Read `.claude/docs/technical-patterns.md` first — bug pattern reference (patterns marked HISTORICAL actually happened here).
2. Read `.claude/docs/review-core.md` — review methodology and subsystem mapping.
3. Read `.claude/docs/false-positive-guide.md` — consult before reporting any finding.

## Input

Arguments: `$ARGUMENTS`

Parse to determine scope; `--fix` flag means apply verified fixes after review.
Use the session's current effort level (no explicit override — review depth scales with it naturally).

| Input | Scope |
|-------|-------|
| *(empty)* | Latest commit |
| `N` (integer) | Last N commits |
| `all` | Full codebase audit |
| `<commit hash>` | Specific commit |
| `<hash1>..<hash2>` | Commit range |

Skip to **Full Audit Protocol** for `all`; otherwise use the Execution Protocol below.

## Execution Protocol (diff-based reviews)

### Phase 1: Context & Classification

1. Read the full diff (`git diff <range>`)
2. Identify ALL affected subsystems using the **subsystem mapping table in `review-core.md` Phase 1**
3. For each affected subsystem, read its pattern guide from `.claude/docs/patterns/`
4. Classify each change per `review-core.md` Phase 2

### Phase 2: Parallel Multi-Agent Review

Launch **4 review agents in parallel**, each focusing on a different dimension.
Each agent receives: the full diff, the summary context, the list of affected subsystems,
and the relevant pattern guide excerpts.

**Agent 1 — Correctness Bugs** (deep context read):
Scan for bugs that require understanding surrounding code. Focus on:
- Error chaining: every error site uses `.c(d!())`, never bare `.unwrap()` on `ruc::Result`
- Macros: `#[macro_export]` uses `$crate::` (not `crate::`), trailing commas supported
- Crypto: AES nonce freshness (random 12-byte), ED25519 key handling, RNG seeding
- Command/SSH: timeout handling, child process cleanup, fd leaks, secret non-echo
- Encoding: round-trip correctness (`decode(encode(x)) == x`), feature-gated availability
- Feature graph: each feature compiles alone; `full` includes all sub-features; leaf ∈ group ∈ full
- Global state: `LazyLock` read-once, `env_or` fallback correctness, serial-test safety
- Only flag issues with concrete failure scenarios (see false-positive-guide.md)

**Agent 2 — Diff-Only Bugs** (diff surface scan):
Scan ONLY the diff lines without reading extra context. Flag:
- Syntax errors, type errors, missing imports (will not compile)
- Clear logic errors visible in the diff alone (inverted conditions, off-by-one)
- Unreachable code, dead branches introduced by the change
- Ignore anything that requires surrounding code to validate

**Agent 3 — Cross-Cutting & Security** (context-aware):
Check every change for:
- Security: nonce freshness, injection surface, secret leakage, RNG seeding
- Feature graph: compiles alone (`cargo check --no-default-features --features X`)? cfg polarity correct? leaf ∈ group ∈ full?
- API compatibility: observable behavior changes? deprecation policy honored? semver implication?
- Performance: does this add allocation on hot paths (error creation, encode/decode)?

**Agent 4 — Code Style & Conventions** (project rules):
Check changed files against:
- `#![deny(warnings)]` and `#![deny(missing_docs)]` — no new suppressions
- Imports grouped (std → external → crate), common prefixes merged
- Exported macros use `$crate::` and support trailing commas
- Doc-code alignment: public API changes must update doc comments, README.md, doc/*.md, CLAUDE.md, review-core.md Phase 1, and affected pattern guides
- Feature-gated modules use `#![allow(missing_docs)]` at module level

**CRITICAL: Only report HIGH SIGNAL issues.** Flag only:
- Code that will definitely fail to compile
- Code that will definitely produce wrong results
- Clear invariant violations from technical-patterns.md
- Concrete crash/leak/corruption/security scenarios

Do NOT flag: style preferences, "consider" suggestions without concrete downside, issues a linter catches, issues matching false-positive-guide.md patterns. Do NOT push toward over-formalization — ruc is pragmatic.

### Phase 3: Verification

For each finding from Phase 2 agents, launch a **verification agent** that:
1. Re-reads the reported code location with full context
2. Attempts to CONFIRM or REFUTE the finding against actual code
3. Cross-references with `false-positive-guide.md`
4. Returns only CONFIRMED findings with concrete evidence

Filter out any finding not confirmed by its verification agent.

### Phase 4: Audit Registry

1. Read `doc/audit.md` (create if absent)
2. **Prune**: Remove `## Open` entries that are 100% fixed in current code
3. **Merge**: Add confirmed findings under `## Open`, deduplicating against existing entries. Sort by severity (CRITICAL → HIGH → MEDIUM → LOW)
4. **Re-evaluate Won't Fix**: For each `## Won't Fix` entry, re-read the code. Promote to `## Open` if now fixable; remove if no longer applicable; keep if reason still holds
5. Write updated `doc/audit.md`. **Never include timestamps, dates, or time-based markers.**

Format:

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

### Phase 5: Report

Use the **ReportFindings** tool with the confirmed findings. Then output a terminal summary:

```
## Review Summary

**Scope**: <commits/diff description>
**Subsystems**: <list>
**Findings**: N (X critical, Y high, Z medium, W low)

## Findings
(one line per finding with severity and location)
```

If zero findings: `**Result**: LGTM — no regressions found. Coverage: <subsystems and invariants checked>.`

### Phase 6: Fix (if --fix)

If `--fix` was passed and findings exist:
1. Apply each fix to the working tree
2. Re-report findings via ReportFindings with `outcome` set (`fixed`, `skipped`, `no_change_needed`)

---

## Full Audit Protocol (for `all` mode)

### Strategy: Parallel Subsystem Audit

Launch **one Agent per pattern guide** in parallel — 4 agents total:

| Agent | Files | Guide |
|-------|-------|-------|
| err-common | `src/lib.rs`, `src/err/`, `src/common.rs`, `Cargo.toml` features | `patterns/err-common.md` |
| exec | `src/cmd.rs`, `src/ssh.rs`, `src/uau.rs`, `src/http.rs` | `patterns/exec.md` |
| algo | `src/algo/` (all) | `patterns/algo.md` |
| ende | `src/ende/` (all) | `patterns/ende.md` |

Each agent's prompt must be self-contained and include:
1. Exact file list for the subsystem
2. Full content of the corresponding pattern guide from `.claude/docs/patterns/`
3. Instructions to read `technical-patterns.md` and `false-positive-guide.md`
4. The code style rules from Agent 4
5. High-signal-only + pragmatic-design reminder: flag only confirmed bugs, don't push toward over-formalization

### Aggregation

After all agents complete:
1. Collect all findings
2. Launch verification agents for each finding (Phase 3)
3. Deduplicate cross-subsystem findings
4. Update audit registry (Phase 4)
5. Report with ReportFindings + terminal summary (Phase 5)
6. Fix if --fix (Phase 6)
