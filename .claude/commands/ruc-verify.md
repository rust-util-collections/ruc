# /ruc-verify — RUC Comprehensive Verification

Run the full verification suite for the ruc crate and report results.

## Execution

Run all steps below. Collect pass/fail for each. Do NOT stop on first failure — run everything and report at the end.

### Step 1: Build (all features)

```bash
cargo build --all-features 2>&1
```

Record: pass/fail + any errors.

### Step 2: Build (default features)

```bash
cargo build 2>&1
```

Record: pass/fail. This catches code that accidentally depends on optional features.

### Step 3: Test

```bash
cargo test --all-features 2>&1
```

Record: pass/fail + count of passed/failed/ignored tests.

### Step 4: Clippy

```bash
cargo clippy --all-features 2>&1
```

Record: pass/fail + any warnings (project uses `#![deny(warnings)]` so warnings are errors).

### Step 5: Feature Isolation Check

Check that each individual feature compiles independently. Run these in parallel where possible:

```bash
cargo check --no-default-features --features cmd
cargo check --no-default-features --features ssh
cargo check --no-default-features --features http
cargo check --no-default-features --features algo_rand
cargo check --no-default-features --features algo_keccak
cargo check --no-default-features --features algo_sha256
cargo check --no-default-features --features algo_aes
cargo check --no-default-features --features algo_ed25519
cargo check --no-default-features --features ende_hex
cargo check --no-default-features --features ende_base64
cargo check --no-default-features --features ende_compress
cargo check --no-default-features --features ende_zstd
cargo check --no-default-features --features ende_json
cargo check --no-default-features --features ende_msgpack
cargo check --no-default-features --features ende_transcode
```

Skip `uau` on non-Linux platforms. Record pass/fail for each feature.

### Step 6: Doc Check

```bash
cargo doc --all-features --no-deps 2>&1
```

Record: pass/fail + any missing-doc warnings.

## Output Format

```
## RUC Verification Report

| Step | Status | Details |
|------|--------|---------|
| Build (all features) | ✅/❌ | ... |
| Build (default) | ✅/❌ | ... |
| Test | ✅/❌ | N passed, N failed, N ignored |
| Clippy | ✅/❌ | N warnings |
| Feature: cmd | ✅/❌ | |
| Feature: ssh | ✅/❌ | |
| Feature: http | ✅/❌ | |
| Feature: algo_rand | ✅/❌ | |
| Feature: algo_keccak | ✅/❌ | |
| Feature: algo_sha256 | ✅/❌ | |
| Feature: algo_aes | ✅/❌ | |
| Feature: algo_ed25519 | ✅/❌ | |
| Feature: ende_hex | ✅/❌ | |
| Feature: ende_base64 | ✅/❌ | |
| Feature: ende_compress | ✅/❌ | |
| Feature: ende_zstd | ✅/❌ | |
| Feature: ende_json | ✅/❌ | |
| Feature: ende_msgpack | ✅/❌ | |
| Feature: ende_transcode | ✅/❌ | |
| Doc generation | ✅/❌ | |

### Overall: PASS / FAIL (N issues)

<if any failures, list details and suggested fixes>
```
