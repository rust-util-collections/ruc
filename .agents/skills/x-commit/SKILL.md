---
name: x-commit
description: Review intended RUC worktree changes, fix confirmed regressions, validate, and create atomic local commits with the required version update. Invoke explicitly as $x-commit, optionally with Git pathspecs.
---

# Review and commit RUC changes

Read [the workflow policy](../../../doc/ai/workflow-policy.md),
[review core](../../../doc/ai/review-core.md),
[technical patterns](../../../doc/ai/technical-patterns.md) and
[the false-positive guide](../../../doc/ai/false-positive-guide.md). Resolve
repository paths from the Git root; links are relative to this file.

This explicit invocation authorizes local commits of intended changes and
required fixes. Accept optional Git pathspecs; reject option-like input and
place pathspecs after `--` when running Git commands.

1. Record staged, unstaged and untracked baseline, starting HEAD, version and
   ownership. Pathspecs limit intended candidates. Without them, inspect all
   intended changes. Resolve unclear ownership without absorbing concurrent
   work. Nothing intended and no owed bump means nothing to commit.
2. Freeze owned paths/hunks, preserve staged boundaries and split independent
   root causes into units. Inspect full implementations, callers/tests and
   mapped subsystem guides for each unit.
3. Review invariants, boundary/failure paths, global state, feature isolation
   and API/encoding compatibility. Refute candidates before fixing confirmed
   regressions with focused tests and relevant docs. Record unfixed confirmed
   defects in `doc/audit.md`; do not commit a known-defective unit.
4. Validate and commit each unit under the shared protocol. Format only owned
   files, use serial tests, stage exact paths/hunks and inspect the candidate.
   Keep tests, contract docs and audit changes with the corresponding fix.
5. Review the aggregate diff and complete the final gates and owed version step
   once. Reuse unchanged-state results; a composed overhaul owns finalization.
6. Report scope, fixes, validation, compatibility, hashes/subjects, version and
   untouched baseline. Preserve validated units if another unit is blocked.

No automatic push, PR, tag, publication or history rewrite.
