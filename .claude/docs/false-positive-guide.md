# RUC False Positive Guide

Before reporting any finding, check it against this guide. If a finding matches
a false positive pattern below, either suppress it or explicitly note why it
does NOT apply.

---

## FP-1: Rust Ownership System Already Prevents It

**Pattern**: Reporting use-after-free, double-free, or dangling references in safe Rust.
**Rule**: ruc contains no `unsafe` in production paths (a test uses `unsafe { env::set_var }` as required by Rust 2024). The borrow checker prevents these at compile time. Only report memory-safety issues if an `unsafe` block is actually introduced.

## FP-2: `bash -c` Injection in `cmd::exec` Is Documented Caller Responsibility

**Pattern**: Reporting that `cmd::exec(cmd)` / `exec_timeout` allow shell injection.
**Rule**: These APIs *are* deliberate "run this shell string" primitives; their docs state "Do not pass unsanitized user input." Do not re-report the API's existence.
**When to report**: (1) ruc itself interpolates data into a shell string internally (e.g., building a command from a path — see the historical SSH `cat` bug); (2) a new API forwards user data to a shell without the documented warning; (3) an SFTP/API alternative exists but shelling out is used anyway.

## FP-3: Blocking I/O Is By Design

**Pattern**: Suggesting async/await, non-blocking sockets, or "this call can block".
**Rule**: `http`, `ssh`, `cmd`, `uau` are intentionally synchronous/blocking utilities. Blocking is the contract, not a bug. Only report if a *documented timeout* fails to bound the blocking time.

## FP-4: Unwrap/Expect on Known-Valid State

**Pattern**: Reporting `unwrap()`/`expect()` as potential panics.
**Rule**: Before reporting, check:
1. Is the value guaranteed by prior logic in the same scope (e.g., `ClientBuilder::build()` with only valid static options)?
2. Is it test code? (`pnk!`/`unwrap` are fine in tests)
3. Is it `unwrap_or`/`unwrap_or_else` (not a panic at all)?
**When to report**: Only with a concrete input that makes it fail in production. Note: `.unwrap()` on a `ruc::Result` is still a *convention* finding (use `pnk!()`), severity MEDIUM — but never call it a panic-safety bug unless it can actually fire.

## FP-5: Clippy Would Catch It

**Pattern**: Reporting lints that `cargo clippy` + `#![deny(warnings)]` already enforce.
**Rule**: CI gates on clippy with deny-all. Do not duplicate clippy. Focus on semantic correctness clippy cannot see (clamp direction, cfg polarity, nonce freshness, round-trip symmetry).

## FP-6: "Consider" Without Concrete Downside

**Pattern**: Suggesting refactors, extra validation, or "defensive" code with no specific failure scenario.
**Rule**: Every finding needs a concrete scenario where current code produces a wrong result, a panic, a leak, or a security hole.

## FP-7: Test-Only Code Held to Production Standards

**Pattern**: Reporting error handling, hardcoded values, or perf issues in `#[cfg(test)]` code.
**Rule**: Tests may use `unwrap()`, fixed paths, `sleep_ms!`, and serial-execution assumptions (CI runs `--test-threads=1`). Only report a test that tests the wrong thing, or a test that breaks under serial execution.

## FP-8: Lossy UTF-8 in String-Returning APIs

**Pattern**: Reporting `from_utf8_lossy` as data corruption in `cmd::exec`.
**Rule**: `exec` returns `String` by contract — lossy conversion is the documented tradeoff; `ssh::exec_cmd` returns `Vec<u8>` for byte-exact needs. Only report if a *byte-exact* API path introduces lossy conversion.

## FP-9: Performance Issue Without a Realistic Scenario

**Pattern**: Reporting an allocation or clone as a performance problem.
**Rule**: ruc is a utility library — the hot loop belongs to the caller. Report only: per-call construction of heavy reusable resources (HTTP client — Pattern 7.2), allocation in the error-*free* path of high-frequency helpers (`d!()`, `ts!()`), or O(n²) where O(n) is trivially available.

## FP-10: Deprecated API Still Present

**Pattern**: Reporting that a `#[deprecated]` item should be deleted.
**Rule**: Policy keeps deprecated items for ≥ 1 major version, delegating to replacements. Only report: forked logic (Pattern 8.1), missing `since`/`note`, or items eligible for removal during a *major-bump release* review.

## FP-11: `--test-threads=1` Looks Like a Smell

**Pattern**: Suggesting tests be parallelized or the flag removed.
**Rule**: Serial tests are mandated because tests mutate process-global env vars (`src/common.rs`). The flag is the fix, not the bug.

## FP-12: Feature Gate on Items Instead of Modules

**Pattern**: Reporting that individual functions lack `#[cfg(feature)]` attributes.
**Rule**: The convention is gating at the `pub mod` declaration in `lib.rs`/`mod.rs`, not per item. Only report if a module-level gate is actually missing or a cross-feature reference lacks a Cargo.toml dependency (Pattern 4.2).
