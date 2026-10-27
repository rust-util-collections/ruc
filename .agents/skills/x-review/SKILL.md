---
name: x-review
description: Review RUC Rust changes or the full repository for error-chain, execution, crypto, encoding and feature regressions. Invoke explicitly as $x-review with an optional scope and --fix.
---

# Review RUC

Read [the workflow policy](../../../doc/ai/workflow-policy.md),
[review core](../../../doc/ai/review-core.md),
[technical patterns](../../../doc/ai/technical-patterns.md) and
[the false-positive guide](../../../doc/ai/false-positive-guide.md). Resolve
repository paths from the Git root; links are relative to this file.

Accept one scope (`N`, `all`, `staged`, `worktree`, commit or `A..B`) and optional
`--fix`. Empty scope means the latest commit. Use the shared scope rules;
reject unknown options.

1. Record the baseline, freeze scope and map changed paths through review core.
   Read full functions, callers, error paths and tests. Load only the relevant
   subsystem guides. For `all`, keep a complete tracked-file coverage ledger.
2. Check subsystem invariants, boundaries, cleanup, global state, feature
   isolation and API/encoding compatibility. Keep RUC's practical toolkit APIs;
   do not turn stylistic or speculative preferences into regressions.
3. Refute every candidate against current code, callers, guards and the false
   positive guide. A concrete trigger and observable consequence establish a
   finding; reviewer votes do not. Cover missing paths/invariants before claiming
   completeness. Historical scopes retain only still-present defects.
4. Reconcile in-scope findings in `doc/audit.md` using
   [the registry rules](../../../doc/ai/review-core.md#audit-registry). Code stays
   read-only without `--fix`; the registry is the only default write. An explicit
   no-writes instruction also leaves the registry alone.
5. With `--fix`, make confirmed in-scope fixes sequentially, add focused
   regressions, validate with serial tests and re-review the aggregate diff.
   Preserve unrelated baseline and leave blocked/unverified findings Open.
6. Report scope, coverage/gaps, evidence, dispositions and validation limits.
   A clean scoped review is not a full-repository correctness claim.

Never stage, commit, bump versions, tag, push or publish in this workflow.
