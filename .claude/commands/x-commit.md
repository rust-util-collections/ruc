---
description: Self-reviewing commit — review uncommitted changes, fix all findings, format, lint, test, commit
---

# Self-Reviewing Commit for RUC

You are performing a self-reviewing commit: review all uncommitted changes, fix every issue found, format, and commit.

## Setup

1. **MANDATORY**: Read `.claude/docs/technical-patterns.md` — bug pattern reference.
2. Read `.claude/docs/review-core.md` — review methodology.
3. Read `.claude/docs/false-positive-guide.md` — consult before reporting any finding.

## Execution Protocol

### Task 1: Deep Self-Review

1. Run `git diff HEAD` to collect all uncommitted changes (`git status --short` for untracked files).
2. If there is nothing to commit, report "nothing to commit" and stop.
3. Identify ALL affected subsystems using the **subsystem mapping table in
   `review-core.md` Phase 1** (single source of truth).
4. For EACH affected subsystem, read the corresponding pattern guide from `.claude/docs/patterns/`.
5. Perform the full regression analysis from review-core.md:
   - **Classify** each change (crypto, command construction, cfg/feature, encoding, timeout/bounds, etc.)
   - **Invariant check** — verify all INV-* from the loaded pattern guides (Phase 3.1)
   - **Boundary conditions** — Phase 3.2 (empty input, short prefixes, timeout 0/max, length off-by-one)
   - **Failure paths** — Phase 3.3 (children killed, fds released, secrets not echoed)
   - **Global state** — Phase 3.4 (LazyLock read-once, serial-test safety)
6. Check cross-cutting concerns (Phase 4):
   - **Security** — nonce freshness, injection surface, secret leakage
   - **Feature graph** — feature isolation, cfg polarity, leaf ∈ group ∈ full
   - **API compatibility** — observable behavior, deprecation policy, semver implication
7. Enforce code style rules (Phase 4.4):
   - No new `#[allow(...)]` — fix warnings at the source
   - Grouped imports; `$crate::` in exported macros
   - Doc-code alignment for public API changes (README, doc/*.md, CLAUDE.md, mapping table, pattern guides)
8. Cross-reference every finding with `false-positive-guide.md` — only retain findings with **concrete evidence**.

### Task 2: Fix All Findings

For EVERY finding from Task 1 (CRITICAL, HIGH, MEDIUM, or LOW):

1. Fix the issue completely — no TODOs, no "fix later", no partial fixes.
2. After all fixes are applied, re-run `git diff HEAD` and repeat Task 1 analysis on the new diff.
3. If new findings emerge from the fixes, fix those too. Iterate until the review is clean.
4. Report the final list of fixes applied.

### Task 3: Format, Lint & Test

1. Run `make fmt` to apply formatting (`rustfmt.toml`: max_width 79).
2. If any `.rs` file changed, run `make lint` — clippy on `full`, with and
   without `--tests`; must pass with zero warnings (all warnings are errors).
   Fix failures at the source (never `#[allow]`) and re-run until clean.
3. If the changes touch code (not just docs/comments), run the tests exactly
   as CI does — serial, because tests mutate process-global env vars:
   - Targeted (single subsystem touched): `cargo test <module> --release -- --test-threads=1`
   - Otherwise full: `make test` (3 passes: default, `--no-default-features`, `full,compact`)
4. If the diff touches `#[cfg(feature)]` logic or `Cargo.toml` features, also
   check isolation for each touched feature:
   `cargo check --no-default-features --features <X>`

### Task 4: Bump Patch Version — MANDATORY

**You MUST complete every step below before proceeding to Task 5. Do NOT skip this task.**

1. Run `git diff HEAD --name-only` — if it lists any `.rs` file, a version bump is required. Skip this task ONLY if every changed file is a non-code file (`.md`, CI config, etc.).
2. Read `Cargo.toml` to get the current `version = "X.Y.Z"`.
3. Compute `NEW = X.Y.(Z+1)` (e.g., `10.0.0` → `10.0.1`). If the changes are breaking (public API removal/change), bump major instead and flag it in the commit message (`feat!:`/`fix!:`).
4. Update `Cargo.toml` — `version = "NEW"`.
5. **Verify**: grep `Cargo.toml` for the NEW version string. If `Cargo.lock` is tracked, run `cargo check` once to refresh it and stage it too.

### Task 5: Commit

1. Run `git diff HEAD --stat` and `git log -5 --oneline` to understand scope and commit style.
2. Draft a commit message:
   - Follow the repo's conventional style (`fix:`, `feat:`, `docs:`, `chore:`, `feat!:` for breaking)
   - Summarize the "why", keep the subject concise; add a body for multi-subsystem changes
3. Stage the relevant files with `git add` (specific files, not `-A`).
4. Commit using a HEREDOC — **do NOT include any co-author line**:

```
git commit -m "$(cat <<'EOF'
<commit message here>
EOF
)"
```

5. Run `git status` to verify the commit succeeded.

## Output Format

```
## Self-Review Commit Summary

**Reviewed**: <number of files changed>
**Subsystems**: <list>
**Findings**: <N found, N fixed> (or "0 — clean")
**Validation**: fmt ✅ | lint ✅ | tests ✅ (or targeted scope)
**Commit**: <short hash> <subject line>
```
