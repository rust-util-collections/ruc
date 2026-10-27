# RUC maintenance workflow

[AGENTS.md](../../AGENTS.md) is the shared entrypoint. Explicitly invoke
`$x-review`, `$x-fix`, `$x-commit` or `$x-overhaul` from
[`.agents/skills/`](../../.agents/skills/). Arguments are ordinary prompt text;
skill files do not substitute variables or execute embedded commands. Each
`agents/openai.yaml` sets `allow_implicit_invocation: false`.

Ordinary requests do not invoke commit/version workflows. Workflow instructions
do not grant tool permissions or replace runtime approval controls. These skills
create local commits only: no automatic push, PR, tag or history rewrite.
An explicit user request to push is separate authorization. Publishing remains
a human action; never run `cargo publish` from these workflows.

## Rust toolchain

Use the official latest stable Rust for development, dependency updates, CI
and releases. Run `rustup update stable` before Rust validation and select it
with `RUSTUP_TOOLCHAIN=stable` or `cargo +stable`. A locally installed channel
named `stable` may need updating.

The repository defines no compatible compiler range or minimum Rust version.
Do not add `rust-version`, numeric compiler pins, minimum-version CI jobs or
compatibility shims for older compilers. Edition 2024 and dependency versions
remain project choices; this policy does not authorize API or encoding breaks.
Compiler versions in historical release notes describe those releases only.

## Scope resolution

Freeze starting HEAD and resolve revisions as commits with
`git rev-parse --verify --quiet '<revision>^{commit}'`. Treat arguments as data:
quote revisions, reject option-like input, and place pathspecs after `--`.

| Input | Reviewed content |
|---|---|
| Empty | Latest commit versus its first parent; root commit versus the empty tree |
| Positive integer `N` | `git log HEAD~N..HEAD` and `git diff HEAD~N HEAD` |
| `staged` | Index diff only |
| `worktree` | `git diff HEAD` plus intended untracked files from `git ls-files --others --exclude-standard` |
| `all` | Tracked-file inventory from `git ls-files` |
| Commit | That commit versus its first parent; root commit versus the empty tree |
| `A..B` | `git log A..B` and endpoint diff `git diff A B`; never substitute `A...B` |

`x-review` and `x-overhaul` share these scopes. Only `x-review` accepts `--fix`.
Reject multiple scopes, unknown flags and invalid revisions. If a count exceeds
available parents or an all-digit token also resolves as a commit, resolve the
ambiguity with the user. Never silently broaden scope.

Historical scopes fix only still-present defects rooted in that diff. Related
callers and tests supply evidence, not unrelated fix scope. An `all` review
records coverage of every source/test, feature, build/CI, tool, documentation
and agent-instruction group, including gaps; a search or passing suite is not
evidence that all code was reviewed.

## Worktree ownership

Before edits, record branch, starting HEAD, staged/unstaged diffs and untracked
paths. Freeze owned paths/hunks and planned commit units. Register a new fix
path only after inspecting its current diff; recheck for concurrent changes
before staging, including changes within owned files.

Preserve unrelated work and staged boundaries. Do not stash, clean, restore,
reset or overwrite baseline to obtain a clean tree. Never run `make cleanall`.
If a fix overlaps unowned changes inseparably, leave that unit alone and report
the conflict; continue independent safe units. Merge audit edits.

Resolve the intended operation with the user before mutating during a merge,
rebase or cherry-pick, or committing on detached HEAD. Do not switch branches
or rewrite history to bypass an unresolved operation.

## Validation

Use the smallest meaningful regression first. Tests always use
`--test-threads=1`: they mutate process-global environment variables. Test
temporary data and subprocesses belong to the invocation; remote SSH targets
and credentials require their own authorized scope. Never use real hosts just
because historical examples mention them.

- Docs/instruction-only changes: `git diff --check`, local links/anchors and
  skill metadata/reference checks. No Rust suite or version bump.
- Rust changes: `cargo fmt --all -- --check`; format only owned files if needed.
  Run `make lint` and the smallest relevant release-mode test with its feature
  enabled, for example `cargo test --release --features cmd cmd:: -- --test-threads=1`.
- Feature/cfg changes: also run
  `cargo check --no-default-features --features <feature>` for each affected leaf.
- Final code gate, or a full `all` audit even without fixes: `make lint` and
  `make test`. The Makefile is the command source of truth; `make test` runs
  release tests for default, no-default and `full,compact` configurations.
  Use the latest official stable toolchain throughout.

Run checks sequentially. Reuse passing results while relevant code/configuration
is unchanged; composed workflows own one final gate, not repeated standalone
finalizations. Linux CI/native checks are required to exercise Linux-only `uau`;
a macOS build cannot verify that module. Report unavailable platform checks and
pre-existing failures explicitly. A required failing/blocked check leaves the
affected finding Open and prevents the version step.

## Atomic local commits and version

Explicit `x-fix`, `x-commit` and `x-overhaul` invocations authorize local commits.
Standalone `x-review`, including `--fix`, never stages, commits or versions.

1. One independent root cause per commit, with its regression, docs and audit
   update. Re-review each unit and the aggregate diff; fix new regressions in
   new commits. Registry-only dispositions form a separate documentation unit.
2. Combined dirty-tree tests do not prove intermediate commits. If another unit
   can mask a failure, validate only the intended candidate in an isolated
   temporary worktree without disturbing baseline.
3. Stage exact owned paths/hunks; never broad `git add -A`, `-u`, `.`, or
   `git commit -a`. Inspect the exact candidate diff. Preserve unrelated staged
   work with an isolated index or `git commit --only` when entire selected
   paths belong to the unit.
4. Use the repository's conventional English commit subjects. Write multiline
   messages to a temporary file and use `git commit --file`; include no
   co-author/generated-by trailer. Verify each commit and compare status with
   baseline. No amend, rebase, force-push or destructive cleanup.

Keep the starting version and release decision in the invocation ledger. A
version bump is owed when tracked `.rs` files changed since the latest commit
that changed the crate's `version = ` in `Cargo.toml`, including intended Rust
changes committed by this invocation. Check the aggregate invocation/history,
not only `git diff HEAD` after committing each unit:

```sh
base=$(git log -1 --format=%H -G'^version = ' -- Cargo.toml)
git diff --name-only "$base" HEAD -- '*.rs'
```

After required gates pass, with no in-scope Open findings or coverage gaps,
complete one owed patch increment `X.Y.Z` to `X.Y.(Z+1)` in a separate local
`chore: bump version to X.Y.Z` commit. Do not bump again for each fix. Docs/config
alone do not create an owed bump. A blocked run preserves validated commits;
the owed bump remains detectable for a later invocation, including one with
nothing else to commit. Do not overwrite unowned manifest changes or divergent
versions. `Cargo.lock` is ignored; never force-add it.

Do not ship an unintended breaking API/encoding change in a patch. An intended
break requires the user's acceptance and a documented migration before a major
increment; the agent cannot accept it on its own. No automatic tag, push, PR or
publication follows any version commit. No changes and no owed bump means no
empty commit.

## Evidence and handoff

Use [review-core.md](review-core.md) and
[false-positive-guide.md](false-positive-guide.md) for evidence and dispositions.
Report scope, reviewed paths/invariants, coverage gaps, finding dispositions,
validation commands/results, compatibility, local hashes and version outcome.
State what remains Open and what baseline was preserved.
