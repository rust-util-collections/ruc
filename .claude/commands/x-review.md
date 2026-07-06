---
description: Deep regression review of ruc changes (latest commit, N commits, hash, range, or full audit)
argument-hint: "[N | all | <hash> | <hash1>..<hash2>] [--fix]"
---

# Deep Regression Analysis for RUC

You are performing a deep code review of changes to RUC, a Rust utility library
(error chains, command/SSH execution, crypto, encode/decode).
This review combines RUC-specific pattern analysis with Claude Code's native
multi-agent review architecture: **dimensional review agents → adversarial
verification → completeness critic → structured report**.

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

---

## Unified Protocol

All modes follow the same 7-phase structure. Mode-specific adaptations are noted inline.

### Phase 1: Scope & Context

**All modes**:
1. Identify affected subsystems using the **subsystem mapping table in `review-core.md` Phase 1**
2. Load the pattern guide (`.claude/docs/patterns/`) for each affected subsystem

**Diff modes** (empty, N, hash, range):
3. Read the full diff (`git diff <range>`)
4. Classify each change per `review-core.md` Phase 2 (category + risk level)

**`all` mode**: all 4 subsystems affected; load all 4 pattern guides.

### Phase 2: Parallel Multi-Dimensional Review

This is the core of the review. Launch agents that cover distinct review *dimensions* —
different ways of seeing the same code, not just different files.

---

#### A. Diff modes (empty, N, hash, range)

Launch **4 agents in parallel**. Each receives: the full diff, affected subsystem list,
and relevant pattern guide excerpts.

**Agent 1 — Correctness Bugs** (reads changed files with full context):
- Error chaining: every error site uses `.c(d!())`, never bare `.unwrap()` on `ruc::Result`
- Macros: `#[macro_export]` uses `$crate::` (not `crate::`), trailing commas via `$(,)*`
- Crypto: AES nonce freshness (random 12-byte), ED25519 key length validation, RNG seeding
- Command/SSH: timeout kills child + reaps, fd leaks, shell injection (FP-2 aware)
- Encoding: round-trip (`decode(encode(x)) == x`), feature-gated availability
- Feature graph: each feature compiles alone; `full` includes all; leaf ∈ group ∈ full
- Global state: `LazyLock` read-once, `env_or` fallback, serial-test safety
- Reference: `technical-patterns.md` categories 1-8; pattern guide invariants

**Agent 2 — Diff-Only Bugs** (scans diff surface, no extra context):
- Syntax errors, type errors, missing imports (will not compile)
- Clear logic errors in the diff alone (inverted conditions, off-by-one)
- Unreachable code, dead branches introduced by the change
- Ignore anything that requires surrounding code to validate

**Agent 3 — Cross-Cutting & Security** (all changed files, context-aware):
- Security: nonce freshness, injection surface, secret leakage, RNG seeding
- Feature graph: compiles alone? cfg polarity correct? leaf ∈ group ∈ full?
- API compatibility: observable behavior changes? deprecation policy (delegate, not fork)? semver?
- Performance: allocation added on hot paths (error creation, encode/decode)?

**Agent 4 — Code Style & Conventions** (all changed files):
- `#![deny(warnings)]` and `#![deny(missing_docs)]` — no new `#[allow(…)]` suppressions
- Imports grouped: `use crate::*;` prelude → std → external crates; common prefixes merged
- Exported macros use `$crate::` and support trailing commas
- Doc-code alignment: public API changes must update doc comments, README.md, doc/*.md,
  CLAUDE.md, review-core.md Phase 1 mapping table, and affected pattern guides
- Feature-gated modules use `#![allow(missing_docs)]` at module level

---

#### B. `all` mode (full audit)

Full audit uses **two layers** — subsystem depth first, then cross-cutting breadth.

**Layer 1 — Subsystem Audit (4 agents, parallel)**:

| Agent | Files | Guide |
|-------|-------|-------|
| err-common | `src/lib.rs`, `src/err/`, `src/common.rs`, `Cargo.toml` features | `patterns/err-common.md` |
| exec | `src/cmd.rs`, `src/ssh.rs`, `src/uau.rs`, `src/http.rs` | `patterns/exec.md` |
| algo | `src/algo/` (all) | `patterns/algo.md` |
| ende | `src/ende/` (all) | `patterns/ende.md` |

Each agent's prompt must be self-contained:
1. Exact file list for the subsystem
2. Full content of the corresponding pattern guide from `.claude/docs/patterns/`
3. Instructions to read `technical-patterns.md` and `false-positive-guide.md`
4. High-signal-only + pragmatic-design reminder

**Layer 2 — Cross-Cutting Review (2 agents, parallel, launched after Layer 1 completes)**:

Once all subsystem agents report, launch 2 agents that read **ALL source files** with a
global lens. These catch what subsystem-isolated agents miss.

**Agent A — Cross-Cutting & Security** (all files):
- Feature graph: every leaf ∈ group ∈ `full`; each leaf compiles alone
- cfg polarity: every `#[cfg(feature)]` / `#[cfg(not(feature))]` pair across the codebase
- Secret leakage: scan every `eg!()` call, `Debug` impl, and error message for key/password/plaintext
- API consistency: naming pairs symmetric across subsystems? deprecation delegates, not forks?
- Env var consistency: all env vars documented in CLAUDE.md? read-once semantics honored?
- Global state: `LazyLock` usage correct; test env mutations serial-safe

**Agent B — Code Style & Conventions** (all files):
- All public items documented; `#![deny(missing_docs)]` clean
- Feature-gated modules use `#![allow(missing_docs)]` at module level
- Import grouping consistent (std → external → crate) across ALL files
- All `#[macro_export]` macros use `$crate::`; trailing commas supported
- Naming conventions: `encode`/`decode`, `compress`/`uncompress` pairs symmetric everywhere
- Doc-code alignment: CLAUDE.md, README.md, review-core.md Phase 1 table match current code

---

**CRITICAL — High-signal gate (applies to ALL agents in ALL modes)**:

Only report findings with **concrete failure scenarios**:
- Code that will definitely fail to compile
- Code that will definitely produce wrong results on realistic input
- Clear invariant violations from `technical-patterns.md`
- Concrete crash / leak / corruption / security scenarios

Do NOT flag: style preferences, "consider" suggestions without concrete downside,
linter-caught issues, or anything matching `false-positive-guide.md` patterns.
Do NOT push toward over-formalization — ruc is pragmatic.

### Phase 3: Adversarial Verification

For each finding from Phase 2, launch **3 verification agents in parallel**. Each agent:
1. Re-reads the reported code location with **full context**
2. Is instructed to **try to REFUTE** the finding — find concrete reasons it is NOT a real bug
3. Cross-references with `false-positive-guide.md`
4. Returns: `{confirmed: bool, evidence: string}`

**Survival rule**: a finding is CONFIRMED only if **≥2 of 3** verification agents confirm
it as real. Findings with 0–1 confirmations are discarded.

This adversarial pattern prevents plausible-but-wrong findings from surviving —
it is the same mechanism used by the native code-review skill.

If zero findings emerged from Phase 2, skip this phase.

### Phase 4: Completeness Critic

Launch **one final review agent** that audits the review itself:
- What subsystems, files, or functions were NOT examined?
- What invariants from the pattern guides were NOT verified?
- What edge cases (empty input, boundary values, error paths) were NOT checked?
- What cross-subsystem interactions were missed?

If gaps are found, loop back to Phase 2 with the specific gap as new scope (launch
targeted agents for the missing coverage only). If no gaps remain, proceed.

If zero findings emerged from Phase 2 and the completeness critic finds no gaps,
skip directly to Phase 6 (no audit.md changes needed).

### Phase 5: Audit Registry

1. Read `doc/audit.md` (create if absent)
2. **Prune**: Remove `## Open` entries that are 100% fixed in current code
3. **Merge**: Add confirmed findings under `## Open`, deduplicating against existing entries.
   Sort by severity: CRITICAL → HIGH → MEDIUM → LOW
4. **Re-evaluate Won't Fix**: For each `## Won't Fix` entry, re-read the code.
   Promote to `## Open` if now fixable; remove if no longer applicable; keep if reason still holds
5. Write updated `doc/audit.md`. **Never include timestamps, dates, or time-based markers.**

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

### Phase 6: Report

Use the **ReportFindings** tool with the confirmed findings (empty array if none).
Then output a terminal summary:

```
## Review Summary

**Scope**: <commits/diff description or "full audit">
**Subsystems**: <list>
**Findings**: N (X critical, Y high, Z medium, W low)

## Findings
(one line per finding: severity, location, one-line summary)
```

If zero findings:
`**Result**: LGTM — no regressions found. Coverage: <subsystems and invariants checked>.`

### Phase 7: Fix (if --fix)

If `--fix` was passed and findings exist:
1. Apply each fix to the working tree
2. Re-report findings via ReportFindings with `outcome` set (`fixed`, `skipped`, `no_change_needed`)
