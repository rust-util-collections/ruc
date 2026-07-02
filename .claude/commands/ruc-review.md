---
description: Evidence-based code review for ruc — API design, feature flags, security, error-chain conventions
argument-hint: [N commits | all | <file paths> | <hash>[..<hash>]]
---

# /ruc-review — RUC Code Review

You are a code reviewer for the **ruc** utility library. Perform a thorough, evidence-based review.

## Setup

Read these first — findings must cite rules from them:

1. `.claude/docs/api-design-rules.md` — naming, error handling, feature flags, generics, macros, deprecation
2. `.claude/docs/module-patterns.md` — module map, invariants (INV-*), security locations, anti-patterns
3. `CLAUDE.md` — project overview and conventions

## Input

Parse the scope from `$ARGUMENTS` (default: `1`):

- **N** (number) — review the last N commits: `git diff HEAD~N..HEAD`
- **all** — full codebase audit: read every `.rs` file under `src/`
- **file paths** — review those files
- **hash or range** — `git diff abc123` / `git diff abc123..def456`

## Execution

### Task 1: Scope & Context

Determine what changed and classify each change by subsystem: `err`, `common`, `cmd`, `ssh`, `uau`, `http`, `algo`, `ende`.

**Do not review from the diff alone.** For every changed function, open the surrounding file to understand context — callers, related helpers, and the module's existing patterns. A diff that looks fine in isolation may violate module invariants.

### Task 2: API Design Review

For each public item (function, macro, struct, trait, type alias) in scope:

- [ ] **Naming**: follows `api-design-rules.md` §1 (encode/decode, compress/uncompress, encrypt/decrypt, `rand_*`/`rand_*_n`, `hash`/`hash_msg` pairs)?
- [ ] **Error handling**: uses `.c(d!())` when propagating? No bare `.unwrap()`/`.expect()` on `ruc::Result`? Uses `ensure!()` for preconditions, `eg!()` only for new error origins?
- [ ] **Generics**: `impl AsRef<[u8]>`/`AsRef<Path>` for public inputs? HRTB for Deserialize? No gratuitous named lifetimes?
- [ ] **Symmetry**: every encode has a decode, every encrypt a decrypt, and they round-trip (INV-ENDE)?
- [ ] **Documentation**: public items have doc comments? (`#![deny(missing_docs)]` except feature-gated modules)

### Task 3: Feature Flag Correctness (INV-FEAT)

- [ ] New modules gated with `#[cfg(feature = "X")]` on the `pub mod` declaration?
- [ ] Feature declared in `Cargo.toml` with its optional deps, added to its parent group AND to `full`?
- [ ] Compiles with `cargo check --no-default-features --features X` alone?
- [ ] No cross-feature reference without a declared feature dependency?
- [ ] `ansi`/`compact` interactions considered if touching `src/err/mod.rs` formatting code?

### Task 4: Security Audit

Check against `module-patterns.md` §Security-Critical Code Locations:

- [ ] AES: fresh random 12-byte nonce per encryption, prepended to ciphertext? Never fixed/zero nonce (INV-AES)?
- [ ] ED25519: key/signature length validated on parse? OS-seeded RNG (INV-ED25519)?
- [ ] cmd/ssh: shell-injection surface documented as caller's responsibility where applicable?
- [ ] No secrets (keys, passwords, tokens) in error messages, logs, or `Debug` output?
- [ ] Timeouts clamped with `.min(upper)` — not `.max()` (INV-SSH; a past real bug)?

### Task 5: Implementation Quality

- [ ] No duplicated logic across modules; deprecated items delegate to replacements?
- [ ] HTTP uses the `HTTP_CLI` `LazyLock` singleton — never a per-request client (INV-HTTP)?
- [ ] `LazyLock` for expensive one-time init?
- [ ] Tests cover happy path AND error path? New global-state tests safe under `--test-threads=1`?
- [ ] No anti-patterns from `module-patterns.md` §Anti-Patterns?

### Task 6: Style & Doc Alignment

- [ ] No new `#[allow(...)]` suppressions (crate uses `#![deny(warnings)]`)?
- [ ] Imports grouped (std → external → crate)? Trailing commas in macros? No needless `clone()`?
- [ ] **Doc-code alignment** — if the change adds/removes/renames a public type, module, feature, or env var, verify these still match:
  - `CLAUDE.md` (layout tree, feature hierarchy, conventions, env vars)
  - `.claude/docs/api-design-rules.md` and `.claude/docs/module-patterns.md`
  - `README.md`, `doc/*.md`, and doc comments

## Severity Rubric

| Level | Meaning | Examples |
|-------|---------|----------|
| 🔴 CRITICAL | Security hole, data corruption, UB, broken invariant | nonce reuse, key material in logs, decode ≠ encode⁻¹ |
| 🟠 HIGH | Bug or API break under realistic usage | panic on valid input, feature build broken, wrong timeout clamp |
| 🟡 MEDIUM | Convention violation, maintainability debt | missing `.c(d!())`, duplicate logic, missing error-path test |
| 🟢 LOW/INFO | Polish | doc typos, naming nits, minor style |

## Output Format

```
## 🔴 CRITICAL (must fix before merge)
### [C-1] <title>
- **File**: path:line
- **Issue**: what's wrong
- **Evidence**: code snippet or reasoning
- **Rule**: which documented rule/invariant (e.g., INV-AES, api-design-rules §2)
- **Fix**: what to do

## 🟠 HIGH (should fix)
...

## 🟡 MEDIUM (consider fixing)
...

## 🟢 LOW / INFO
...

## ✅ Summary
- Files reviewed: N
- Findings: X critical, Y high, Z medium, W low
- Overall assessment: PASS / PASS WITH CONCERNS / FAIL
```

**Quality gate**: every finding must cite a specific `file:line` and reference a documented rule or invariant. Opinions without evidence are not findings. If the scope is clean, say so — do not invent findings to appear thorough.
