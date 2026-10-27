---
name: x-fix
description: Recheck confirmed RUC defects in doc/audit.md, fix them with focused regressions, and create atomic local commits. Invoke explicitly as $x-fix without arguments.
---

# Fix recorded RUC defects

Read [the workflow policy](../../../doc/ai/workflow-policy.md),
[review core](../../../doc/ai/review-core.md),
[technical patterns](../../../doc/ai/technical-patterns.md) and
[the false-positive guide](../../../doc/ai/false-positive-guide.md). Resolve
repository paths from the Git root; links are relative to this file.

Accept no arguments. This explicit invocation authorizes local fix commits.

1. Record starting HEAD, baseline and ownership. Read `doc/audit.md`; recheck
   retained Won't Fix entries against current code, promoting now-actionable
   defects and removing obsolete claims with evidence.
2. Work Open entries in severity order. Read full implementations, callers,
   tests and mapped subsystem guides. Deduplicate root causes; remove already
   fixed or disproven entries with refutation evidence.
3. Freeze each unit's paths, fix its root cause, add a meaningful regression
   and update relevant docs. Check error chains, cleanup, feature/encoding
   contracts and serial-test behavior. Validate before removing its Open entry;
   include that removal with the fix in one atomic local commit.
4. Keep blocked or insufficiently verified defects Open. Won't Fix requires a
   real defect and a concrete reason its safe fix is disproportionate; lack of
   access or a failed check is a blocker. Continue independent safe units.
5. Re-review the aggregate diff; run the final applicable gates once and finish
   any owed version step under the shared policy. With no Open work and no
   owed bump, report nothing to fix. Registry-only corrections are documentation
   units; do not invent tasks from historical bug catalogs.
6. Report dispositions, validation, compatibility, local hashes, version and
   preserved baseline. When composed by `x-overhaul`, inherit its scope/ledger;
   the parent owns the final gate and version step.

No automatic push, PR, tag, publication or history rewrite.
