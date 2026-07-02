---
description: Prepare a ruc release — verify, draft changelog, audit deprecations, produce a release checklist
argument-hint: [target version, e.g. 10.1.0]
---

# /ruc-release — RUC Release Preparation

Prepare a release for the ruc crate: validate everything, generate a changelog, produce a checklist.

If `$ARGUMENTS` contains a target version, validate `Cargo.toml` against it; otherwise use the version already in `Cargo.toml`.

## Setup

1. Read `CLAUDE.md` for project context
2. Read `Cargo.toml` for the current version
3. Read `CHANGELOG.md` for the existing entry format

## Execution

### Step 1: Version Check

```bash
git tag --sort=-v:refname | head -5
```

Verify the `Cargo.toml` version is **strictly newer** than the latest tag (semver ordering, not string ordering). Determine the bump type (major/minor/patch) and check it matches the nature of the changes found in Step 3 — e.g., breaking changes require a major bump. Flag any mismatch.

### Step 2: Full Verification

Execute the `/ruc-verify` workflow (all steps, not `quick`). **If anything fails, report and STOP** — do not proceed with release prep until clean.

### Step 3: Changelog Draft

```bash
git log <last_tag>..HEAD --oneline
```

For any commit whose one-line summary is ambiguous, inspect it with `git show <hash> --stat` before categorizing. Categorize into:

- **Breaking Changes** — API removals, signature changes, behavior changes
- **New Features** — new functions, macros, modules, features
- **Bug Fixes** — correctness fixes, security fixes
- **Improvements** — performance, code quality, dependency updates
- **Deprecations** — newly deprecated items

Write a draft entry matching the existing `CHANGELOG.md` format (keep-a-changelog style).

### Step 4: Deprecated API Audit

```bash
grep -rn '#\[deprecated' src/
```

For each deprecated item, check:

- [ ] `since` version refers to a real past release (compare against `git tag`)?
- [ ] `note` contains concrete migration guidance?
- [ ] The replacement exists and the deprecated item delegates to it (no duplicated logic)?
- [ ] Deprecated for 2+ major versions → flag for removal in this release if it is a major bump

### Step 5: Docs Consistency

Check all user-facing docs against the current public API:

- [ ] `README.md` — feature list current, minimum rustc version matches `Cargo.toml` edition, no references to removed/renamed APIs
- [ ] `doc/*.md` (cmd.md, errmgmt.md, ssh.md, uau.md) — examples still compile conceptually, no stale API names
- [ ] `CLAUDE.md` + `.claude/docs/*.md` — layout tree, feature hierarchy, invariants still accurate

### Step 6: Publish Dry Run

```bash
cargo publish --dry-run
```

Record: pass/fail. (Real `cargo publish` is deliberately denied in settings — it is a human action.)

## Output Format

```
## RUC Release Preparation: vX.Y.Z

### Version
- Current: X.Y.Z
- Previous tag: vA.B.C
- Bump type: major/minor/patch — justified? YES/NO

### Verification: PASS/FAIL
<one-line summary per /ruc-verify step>

### Changelog Draft
## [X.Y.Z] - YYYY-MM-DD
### Breaking Changes
- ...
### New Features
- ...
### Bug Fixes
- ...
### Improvements
- ...
### Deprecations
- ...

### Deprecated API Status
| Item | Since | Replacement | Action |
|------|-------|-------------|--------|
| ...  | ...   | ...         | keep / remove |

### Docs: OK / NEEDS UPDATE
<details if needs update>

### Release Checklist (human actions)
- [ ] All verification steps pass
- [ ] Changelog reviewed and committed to CHANGELOG.md
- [ ] Docs up to date
- [ ] `cargo publish --dry-run` succeeds
- [ ] Git tag created: `git tag vX.Y.Z`
- [ ] Published: `cargo publish`
- [ ] Tag pushed: `git push --tags`
```
