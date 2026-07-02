---
description: Run the full ruc verification suite (build, test, clippy, fmt, feature isolation, docs) and report a pass/fail matrix
argument-hint: [quick]
allowed-tools: Bash(cargo *), Bash(make *)
---

# /ruc-verify — RUC Comprehensive Verification

Run the verification suite for the ruc crate and report results.

**Rules:**
- Run ALL steps. Do NOT stop on first failure — collect every result and report at the end.
- Run independent checks in parallel where possible (they only read the source tree; cargo serializes builds via its own lock).
- If `$ARGUMENTS` is `quick`, run only Steps 1–4 (skip feature isolation and docs).

## Steps

### Step 1: Build

```bash
cargo build --all-features    # everything
cargo build                   # default features only (catches accidental reliance on optional features)
```

### Step 2: Test (must mirror CI — see `Makefile` and `.github/workflows/`)

```bash
cargo test --release -- --test-threads=1
cargo test --release --no-default-features -- --test-threads=1
cargo test --release --features="full,compact" -- --test-threads=1
```

**Why `--test-threads=1`**: tests mutate process-global state (`env::set_var` in `src/common.rs`); parallel test threads race on it. Never "fix" a flaky test by removing this flag.

**Why `full,compact`**: the `compact` feature switches error-formatting code paths in `src/err/mod.rs`; it must be exercised explicitly since `full` does not include it.

Record: pass/fail + count of passed/failed/ignored per combination.

### Step 3: Clippy

```bash
cargo clippy --features="full"
cargo clippy --features="full" --tests
```

The crate uses `#![deny(warnings)]`, so any warning is a failure.

### Step 4: Format Check

```bash
cargo fmt --check
```

Note: `rustfmt.toml` sets `max_width = 79`.

### Step 5: Feature Isolation

Each feature must compile with only its own declared dependencies:

```bash
for f in cmd ssh http algo_rand algo_keccak algo_sha256 algo_aes algo_ed25519 \
         ende_hex ende_base64 ende_compress ende_zstd ende_json ende_msgpack ende_transcode \
         ansi compact; do
  cargo check --no-default-features --features "$f" 2>&1 | tail -1
done
```

Skip `uau` on non-Linux platforms (it is Linux-only). Record pass/fail per feature.

### Step 6: Doc Check

```bash
cargo doc --all-features --no-deps 2>&1
```

Record: pass/fail + any missing-doc warnings (`#![deny(missing_docs)]` is set, but feature-gated modules use `#![allow(missing_docs)]`).

## Output Format

```
## RUC Verification Report

| Step | Status | Details |
|------|--------|---------|
| Build (all features)        | ✅/❌ | |
| Build (default)             | ✅/❌ | |
| Test (release)              | ✅/❌ | N passed / N failed / N ignored |
| Test (no-default-features)  | ✅/❌ | ... |
| Test (full,compact)         | ✅/❌ | ... |
| Clippy (full)               | ✅/❌ | N warnings |
| Clippy (full, tests)        | ✅/❌ | N warnings |
| Format                      | ✅/❌ | |
| Feature isolation (17)      | ✅/❌ | list any failing feature |
| Doc generation              | ✅/❌ | |

### Overall: PASS / FAIL (N issues)
```

If anything failed: quote the exact error output, identify the root cause file:line, and suggest a concrete fix. Do not attempt fixes unless the user asks.
