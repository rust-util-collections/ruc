# /ruc-release — RUC Release Preparation

Prepare a release for the ruc crate. Validate everything, generate changelog, produce a checklist.

## Setup

1. Read `CLAUDE.md` for project context
2. Read `Cargo.toml` for current version

## Execution

### Step 1: Version Check

Read `Cargo.toml` version. Find the latest git tag:
```bash
git tag --sort=-v:refname | head -5
```

Verify the Cargo.toml version is **newer** than the latest tag. Flag if not.

### Step 2: Run Full Verification

Execute the `/ruc-verify` workflow. All steps must pass. If any fail, report them and stop — do not proceed with release prep until clean.

### Step 3: Generate Changelog

Get all commits since the last tag:
```bash
git log <last_tag>..HEAD --oneline
```

Categorize commits into:
- **Breaking Changes** — API removals, signature changes, behavior changes
- **New Features** — new functions, macros, modules, features
- **Bug Fixes** — correctness fixes, security fixes
- **Improvements** — performance, code quality, dependency updates
- **Deprecations** — newly deprecated items

Write a draft CHANGELOG entry in keep-a-changelog format.

### Step 4: Deprecated API Audit

Search for all `#[deprecated` attributes:
```bash
grep -rn '#\[deprecated' src/
```

For each deprecated item, check:
- [ ] Has a `since` version that matches a past release?
- [ ] Has a `note` with migration guidance?
- [ ] The replacement function exists and works?
- [ ] If deprecated for 2+ major versions, flag for removal consideration

### Step 5: README Check

Read `README.md`. Check if it references:
- [ ] Correct minimum rustc version (matches `Cargo.toml` edition requirements)
- [ ] All major features listed
- [ ] Any recently added features that should be mentioned
- [ ] No references to removed/renamed APIs

### Step 6: Documentation Spot Check

Check that new public items have doc comments:
```bash
cargo doc --all-features --no-deps 2>&1
```

Flag any missing-docs warnings.

## Output Format

```
## RUC Release Preparation: vX.Y.Z

### Version
- Current: X.Y.Z
- Previous tag: vA.B.C
- Version bump type: major/minor/patch

### Verification: PASS/FAIL
<summary from /ruc-verify>

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
|------|-------|------------|--------|
| ... | ... | ... | keep/remove |

### README: OK / NEEDS UPDATE
<details if needs update>

### Release Checklist
- [ ] All verification steps pass
- [ ] Changelog reviewed and finalized
- [ ] README up to date
- [ ] Deprecated items audited
- [ ] Version in Cargo.toml is correct
- [ ] `cargo publish --dry-run` succeeds
- [ ] Git tag created: `git tag vX.Y.Z`
- [ ] Published: `cargo publish`
- [ ] Tag pushed: `git push --tags`
```
