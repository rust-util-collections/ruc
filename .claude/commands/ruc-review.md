# /ruc-review — RUC Code Review

You are a code reviewer for the **ruc** utility library. Perform a thorough, evidence-based review.

## Setup

1. Read `.claude/docs/api-design-rules.md` — the project's API design conventions
2. Read `.claude/docs/module-patterns.md` — module structure, invariants, anti-patterns
3. Read `CLAUDE.md` — project overview and conventions

## Input

The user provides a scope (default: latest commit):
- **N** (number) — review the last N commits
- **all** — full codebase audit
- **file paths** — review specific files
- **hash or range** — `abc123` or `abc123..def456`

Parse the scope from: $ARGUMENTS (default: `1`)

## Execution

### Task 1: Scope & Context

Determine what changed:
- If commit-based: `git diff HEAD~N..HEAD` (or range)
- If `all`: read every `.rs` file
- If file paths: read those files

Identify which modules are affected. Classify changes by subsystem: err, common, cmd, ssh, uau, http, algo, ende.

### Task 2: API Design Review

For each public item (function, macro, struct, trait, type alias) in scope:

- [ ] **Naming**: follows conventions in `api-design-rules.md` §1?
- [ ] **Error handling**: uses `.c(d!())` pattern? No bare `.unwrap()` on `ruc::Result`? Uses `ensure!()` for preconditions?
- [ ] **Generics**: appropriate use of `AsRef`, `impl Trait`, lifetime constraints?
- [ ] **Symmetry**: encode/decode, compress/uncompress, encrypt/decrypt pairs complete?
- [ ] **Documentation**: public items have doc comments? Examples where helpful?

### Task 3: Feature Flag Correctness

- [ ] New modules have `#[cfg(feature = "X")]` guards?
- [ ] Feature declared in Cargo.toml with correct dependencies?
- [ ] Feature added to its parent group and to `full`?
- [ ] Module compiles independently with only its feature enabled?
- [ ] No cross-feature references without declared dependency?

### Task 4: Security Audit

Check against `module-patterns.md` security-critical locations:
- [ ] AES: random nonce per encryption? Never `Nonce::default()`?
- [ ] ED25519: key length validated? RNG properly seeded?
- [ ] cmd: shell injection considered? (document if caller's responsibility)
- [ ] No secrets in error messages or logs?
- [ ] Crypto operations use constant-time comparison where needed?

### Task 5: Implementation Quality

- [ ] No code duplication (functions doing same thing in different modules)?
- [ ] HTTP client uses `HTTP_CLI` singleton, not per-request creation?
- [ ] `LazyLock` for expensive one-time initialization?
- [ ] Deprecated items delegate to replacements (no logic duplication)?
- [ ] Tests cover happy path and error cases?
- [ ] No anti-patterns from `module-patterns.md` §Anti-Patterns?

### Task 6: Code Style

- [ ] `#![deny(warnings)]` — no `#[allow(...)]` suppressions added?
- [ ] Imports grouped at file top (std → external → crate)?
- [ ] Trailing commas in macro invocations?
- [ ] No unnecessary `clone()` or allocation?
- [ ] **Doc-code alignment** — If the change adds, removes, or renames a public type, module, or subsystem path, verify docs still match. Specifically check:
  - `CLAUDE.md` architecture table and conventions
  - `.claude/docs/api-design-rules.md` and `.claude/docs/module-patterns.md`
  - Doc comments and README

## Output Format

Report findings grouped by severity:

```
## 🔴 CRITICAL (must fix before merge)
### [C-1] <title>
- **File**: path:line
- **Issue**: what's wrong
- **Evidence**: code snippet or reasoning
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

**Quality gate**: every finding must cite a specific file:line and explain *why* it's a problem with reference to a documented rule or invariant. Opinions without evidence are not findings.
