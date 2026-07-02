# RUC Technical Bug Patterns

This document catalogs known bug categories for ruc. Load this document FIRST
before performing any review or debug analysis.

Every pattern marked **[HISTORICAL]** actually occurred in this codebase and
was fixed in a past release — treat these as the highest-value checks.

**Scope note**: This file covers cross-cutting categories. Subsystem-specific
invariants and checklists live in `.claude/docs/patterns/`:
`err-common.md`, `exec.md`, `algo.md`, `ende.md` — always load the relevant
guide for affected subsystems.

---

## Category 1: Cryptography Bugs

### 1.1 Fixed/Reused Nonce **[HISTORICAL: fixed in v10.0.0]**
**Pattern**: AES-GCM encryption with a constant nonce (`Nonce::default()`, zero
nonce, or any nonce not freshly random per call).
**Where**: `src/algo/crypto/aes.rs`.
**Impact**: Catastrophic — GCM nonce reuse under the same key allows plaintext
recovery and authentication forgery.
**Check**: Every `encrypt()` call generates a fresh random 12-byte nonce and
prepends it to the ciphertext. `decrypt()` reads the nonce from the first 12
bytes and rejects shorter input.

### 1.2 Weak or Unseeded RNG
**Pattern**: Key/nonce generation using a deterministic or poorly-seeded RNG.
**Where**: `src/algo/crypto/`, `src/algo/rand.rs`.
**Impact**: Predictable keys/nonces.
**Check**: Key material comes from the OS RNG (`rand` with OS entropy). Test
seeds never leak into production paths.

### 1.3 Key/Signature Length Not Validated
**Pattern**: Parsing external base64/hex into fixed-size key types without
checking decoded length.
**Where**: `src/algo/crypto/ed25519/readable/` (`TryFrom<String>` impls).
**Impact**: Panic on slice-to-array conversion, or silent truncation.
**Check**: Decoded length is validated (32 bytes keys, 64 bytes signatures)
and errors are returned via `Result`, never `panic!`.

### 1.4 Secret Material in Errors/Logs
**Pattern**: Including passwords, keys, or plaintext in `eg!()` messages,
`Debug` output, or log lines.
**Where**: any crypto or auth code path.
**Impact**: Secrets end up in logs and error chains that callers print.
**Check**: Error messages describe *what failed*, never echo secret inputs.

## Category 2: Command Execution Bugs

### 2.1 Shell Injection via Constructed Commands **[HISTORICAL: fixed in d8d4858]**
**Pattern**: Building a shell command by interpolating a path/argument into a
string executed via `bash -c` or a remote shell — e.g. the old SSH `read_file`
ran `cat <path>` remotely; a path containing `$(...)` executed code.
**Where**: `src/cmd.rs` (documented as caller's responsibility), `src/ssh.rs`
(must NOT shell out when an SFTP/API equivalent exists).
**Impact**: Arbitrary command execution.
**Check**: New APIs never interpolate data into shell strings. File transfer
uses SFTP. If a shell-string API is unavoidable, its docs must state "Do not
pass unsanitized user input."

### 2.2 Zombie/Orphan Process on Timeout
**Pattern**: Timeout path returns an error without killing the spawned child,
or kills it without reaping (`wait`).
**Where**: `src/cmd.rs` `exec_timeout` polling loop.
**Impact**: Process leak; on repeat, fd/PID exhaustion.
**Check**: Every early-exit path either got `Output` (already reaped) or calls
`child.kill()` before returning.

### 2.3 Lossy UTF-8 Masking Real Output
**Pattern**: `from_utf8_lossy` silently mangles binary output where bytes were
expected.
**Where**: `src/cmd.rs`, `src/ssh.rs` (note: `ssh::exec_cmd` returns `Vec<u8>`
for this reason).
**Impact**: Data corruption for binary-producing commands.
**Check**: Byte-returning APIs stay `Vec<u8>`; `String` APIs document lossy
conversion.

## Category 3: Bounds & Timeout Bugs

### 3.1 Inverted Clamp Direction **[HISTORICAL: fixed in v10.0.0]**
**Pattern**: `timeout.max(300)` used to enforce an upper bound — but `.max(300)`
sets a *lower* bound (result is always ≥ 300).
**Where**: `src/ssh.rs`, any env-var-driven limit.
**Impact**: Configuration silently ignored; SSH timeout was always ≥ 300s.
**Check**: Upper bound → `.min(N)`. Lower bound → `.max(N)`. Read it aloud:
"the result is at most/at least N".

### 3.2 Env Var Parse Fallback Surprises
**Pattern**: `parse::<u8>().unwrap_or(default)` — an out-of-range value like
`"300"` fails u8 parse and silently falls back to the default (3s), not to 255.
**Where**: `src/http.rs` (`RUC_HTTP_TIMEOUT`, u8 → cap 255), `src/ssh.rs`
(`RUC_SSH_TIMEOUT`, u32 then `.min(300)`).
**Impact**: User believes they set a big timeout; they got the small default.
**Check**: Fallback behavior on unparsable/out-of-range values is documented.
New env vars follow the same read-once (`LazyLock`) + clamp + document pattern.

## Category 4: cfg / Feature Graph Bugs

### 4.1 Swapped cfg Branches **[HISTORICAL: fixed in v10.0.0]**
**Pattern**: The `#[cfg(feature = "X")]` and `#[cfg(not(feature = "X"))]`
bodies are interchanged — e.g. `ansi` builds once emitted plain ASCII and
non-ansi builds emitted ANSI codes.
**Where**: `src/err/mod.rs` (`ansi`/`compact` formatting fns), any cfg pair.
**Impact**: Wrong behavior in exactly the configuration that isn't tested.
**Check**: For every cfg pair, verify each body matches its own predicate.
Both configurations must be compiled by CI (`full` and `full,compact` passes).

### 4.2 Feature Leakage
**Pattern**: Code under feature A references an item only compiled under
feature B, without Cargo.toml declaring `A = [..., "B"]`.
**Where**: any feature-gated module; historic risk area: `algo_aes` →
`algo_hash`, `algo_ed25519` → `ende_base64`/`ende_hex`.
**Impact**: `--features A` alone fails to compile — broken for downstream
users who enable minimal features.
**Check**: `cargo check --no-default-features --features X` for the touched
feature. New cross-module use ⇒ new feature dependency in Cargo.toml.

### 4.3 Feature Not Registered in Group/full
**Pattern**: New leaf feature exists but is missing from its parent group
(`algo`, `ende`) or from `full`.
**Where**: `Cargo.toml` `[features]`.
**Impact**: `--all-features`/`full` users silently miss the module; docs.rs
(built with `full`) omits it.
**Check**: leaf ∈ group ∧ (leaf ∈ full directly or via group).

## Category 5: Error-Chain Bugs

### 5.1 Broken Chain (bare `?` / unwrap)
**Pattern**: Propagating with bare `?` on a foreign error type, or
`.unwrap()`/`.expect()` on `ruc::Result`.
**Where**: everywhere; the point of the crate is chained context.
**Impact**: Lost file/line breadcrumbs; panic without chain output.
**Check**: Cross-boundary propagation uses `.c(d!())`; die-on-error uses
`pnk!()`; preconditions use `ensure!()`.

### 5.2 Macro Path Hygiene
**Pattern**: `#[macro_export]` macro body referencing `crate::foo` instead of
`$crate::foo`.
**Where**: `src/err/macros.rs`, `src/common.rs`.
**Impact**: Macro breaks when invoked from downstream crates.
**Check**: All crate-item references inside exported macros use `$crate::`.

### 5.3 Allocating Error Locations
**Pattern**: Storing `file: String` (allocated) instead of `&'static str`
from `file!()`.
**Where**: `src/err/mod.rs` (`SimpleMsg`).
**Impact**: Pointless allocation on every error site; API regression.
**Check**: `SimpleMsg::file` stays `&'static str`.

## Category 6: Encode/Decode Bugs

### 6.1 Asymmetric Pair
**Pattern**: `encode` accepts a type/format that the matching `decode` cannot
round-trip (or one of the pair is missing entirely).
**Where**: `src/ende/*`, `src/algo/crypto/*` (`*_to_base64`/`*_from_base64`).
**Impact**: Data written today cannot be read tomorrow.
**Check**: Every new codec ships with a round-trip test
(`decode(encode(x)) == x`), including the empty input.

### 6.2 Silent Truncation on Fixed Buffers
**Pattern**: Receiving into a fixed-size buffer and discarding the "message
longer than buffer" case.
**Where**: `src/uau.rs` `recv_buf::<N>`.
**Impact**: Truncated datagrams treated as complete messages.
**Check**: Buffer-size APIs document truncation semantics; callers can choose
a bigger `N`.

## Category 7: Global State Bugs

### 7.1 Test Races on Process Globals
**Pattern**: A test calls `env::set_var` (or mutates other process globals)
and assumes isolation.
**Where**: `src/common.rs` tests; any new test using env vars.
**Impact**: Flaky/parallel-dependent failures — the reason CI mandates
`--test-threads=1`.
**Check**: New tests tolerate serial execution order; never remove the
`--test-threads=1` flag to "fix" flakiness.

### 7.2 Per-Call Construction of Singletons **[HISTORICAL: fixed in v10.0.0]**
**Pattern**: Building a new `reqwest::Client` (or similar heavy resource) per
request instead of reusing the `LazyLock` singleton.
**Where**: `src/http.rs` (`HTTP_CLI`).
**Impact**: Connection-pool thrashing, fd churn, latency.
**Check**: All request paths go through the singleton; timeout changes go
through `TIME_OUT`, read once.

## Category 8: Deprecation Bugs

### 8.1 Deprecated Item with Forked Logic
**Pattern**: A deprecated function keeps its own implementation instead of
delegating to its replacement; the two drift apart.
**Where**: any `#[deprecated]` item.
**Impact**: Bug fixed in the new path persists in the old path.
**Check**: Deprecated bodies are one-line delegations. `since` matches a real
release; `note` names the replacement.
