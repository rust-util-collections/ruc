# RUC Review Core Methodology

This document defines the systematic review protocol for ruc code changes.

---

## Phase 1: Context Gathering

Before analyzing any change, gather context:

1. **Read the diff** — understand every changed line
2. **Identify affected subsystems** — map changed files using this table.
   This table is the **single source of truth** for subsystem mapping; commands
   (`/x-review`, `/x-commit`, `/x-fix`, `/x-overhaul`) reference it instead of duplicating it.

   | Changed path | Subsystem | Feature(s) | Pattern guide (`.claude/docs/patterns/`) |
   |--------------|-----------|------------|------------------------------------------|
   | `src/err/` (mod.rs, macros.rs), `src/lib.rs` | error management | *(always on; `ansi`/`compact` switch formatting)* | `err-common.md` |
   | `src/common.rs` | collections/time/fs macros & helpers | *(always on)* | `err-common.md` |
   | `src/cmd.rs` | local command execution | `cmd` | `exec.md` |
   | `src/ssh.rs` | remote SSH execution + SFTP | `ssh` | `exec.md` |
   | `src/uau.rs` | Unix abstract UDP sockets (Linux-only) | `uau` | `exec.md` |
   | `src/http.rs` | blocking HTTP client | `http` | `exec.md` |
   | `src/algo/rand.rs` | random generation | `algo_rand` | `algo.md` |
   | `src/algo/hash/` | Keccak-256, SHA-256 | `algo_keccak`, `algo_sha256` | `algo.md` |
   | `src/algo/crypto/aes.rs` | AES-256-GCM | `algo_aes` | `algo.md` |
   | `src/algo/crypto/ed25519/` | ED25519 sign/verify | `algo_ed25519` | `algo.md` |
   | `src/ende/` | encode/decode (hex, base64, zlib, zstd, json, msgpack, transcode) | `ende_*` | `ende.md` |
   | `Cargo.toml` (features section) | feature graph | *(all)* | `err-common.md` §feature rules |
3. **Load subsystem patterns** — read the pattern guide for each affected subsystem (skip unaffected ones)
4. **Check call sites** — grep for all callers of changed functions (including doc examples and tests)
5. **Check related tests** — identify which `#[cfg(test)]` modules cover the changed code

## Phase 2: Change Classification

Classify each change into one or more categories:

| Category | Description | Risk Level |
|----------|-------------|------------|
| Crypto | AES nonce/key handling, ED25519 keys, RNG usage | CRITICAL |
| Command construction | Strings passed to `bash -c` or remote shells | CRITICAL |
| cfg/feature logic | `#[cfg(feature)]` branches, Cargo.toml feature graph | HIGH |
| Encoding | encode/decode, compress/uncompress pairs | HIGH |
| Timeout/bounds | Clamping, env-var parsing, numeric limits | HIGH |
| Resource lifecycle | Process spawn/kill, socket/fd lifetime, Drop impls | HIGH |
| Control flow | if/else, match, loop, early return changes | HIGH |
| Error handling | `.c(d!())` chains, `ensure!`, `eg!`, macro changes | MEDIUM |
| Public API surface | New/renamed/deprecated items, signatures | MEDIUM |
| Docs/comments | Doc comments, README, guides | LOW |
| Test changes | New or modified test cases | LOW |

## Phase 3: Regression Analysis

For each HIGH or CRITICAL change, perform deep analysis:

### 3.1 Invariant Check

Verify the invariants of every affected subsystem — each pattern guide lists
its own invariants (INV-* sections). The cross-cutting ones:

- **Error chain**: every propagated error goes through `.c(d!())`; `d!()` captures file/line/column at the call site
- **Round-trip**: `decode(encode(x)) == x`, `uncompress(compress(x)) == x`, `decrypt(k, encrypt(k, x)) == x`
- **Fresh nonce**: every AES encryption generates a new random 12-byte nonce
- **Clamp direction**: upper bounds use `.min(N)`, lower bounds use `.max(N)` — never swapped
- **Feature isolation**: each feature compiles alone (`cargo check --no-default-features --features X`)
- **cfg polarity**: `#[cfg(feature)]` / `#[cfg(not(feature))]` bodies are not swapped

### 3.2 Boundary Condition Analysis

Check edge cases specific to the change:

- Empty input (`&[]`, `""`) for every encode/decode/hash/crypto function
- Input shorter than a required prefix (e.g., AES ciphertext < 12-byte nonce)
- Timeout = 0, timeout = max (u8/u32 parse limits), env var unset / unparsable
- Buffer exactly at capacity (uau `recv_buf::<N>` with message length == N)
- Non-UTF8 bytes where `String` output is produced (`from_utf8_lossy` semantics)
- Key/signature length off by one (ED25519: 32-byte keys, 64-byte signatures)

### 3.3 Failure Path Analysis

For every new error path introduced:

- Are spawned processes killed on the timeout/error path? (no zombie/orphan processes)
- Are fds/sockets/sessions released (Drop or explicit close) on early return?
- Is the error propagated with `.c(d!())` and enough context to debug?
- Does partial failure leave global state (env vars, singletons) consistent?
- No secrets (keys, passwords, plaintext) embedded in error messages or logs

### 3.4 Global State & Concurrency

For changes touching shared state:

- `LazyLock` statics (HTTP_CLI, TIME_OUT, log level): initialized once — is the read-once semantic acceptable for the change?
- Tests mutating env vars (`env::set_var`) MUST be safe under `--test-threads=1` and must not assume isolation from other tests
- Blocking APIs by design (http, ssh, cmd) — do not introduce hidden async or thread spawning

## Phase 4: Cross-Cutting Concerns

### 4.1 Security

If the change touches `algo/crypto/`, `cmd.rs`, `ssh.rs`, or `http.rs`:
- Nonce/RNG: OS-seeded, never fixed, never reused?
- Injection: strings interpolated into shell commands documented as caller's responsibility, or eliminated (prefer SFTP/API over shelling out)?
- Constant-time comparison where secret equality is checked?
- No secret material in `Debug` impls, error messages, or logs?

### 4.2 API Contract

- Does the change alter observable behavior (output format, error type, timing)?
- Renames/removals: is the old name kept as `#[deprecated]` delegating to the new one?
- Breaking changes require a major version bump (semver)

New/changed public APIs must follow the house style:
- **Naming pairs**: `encode`/`decode` (optionally prefixed: `json_encode`), `{algo}_compress`/`{algo}_uncompress`, `encrypt`/`decrypt` (+`_to_base64`/`_from_base64`), `hash`/`hash_msg`, `rand_{fmt}`/`rand_{fmt}_n`
- **Generics**: `impl AsRef<[u8]>` for byte inputs callers may pass as `String`/`Vec`/`&str`; `impl AsRef<Path>` for paths; `T: Serialize` / `T: for<'a> Deserialize<'a>` (HRTB) for serde; avoid named lifetimes unless borrows are non-obvious (prefer the `RemoteHostOwned` + `From<&Owned>` pattern)
- **Types**: hash outputs are fixed arrays (`[u8; 32]`), error location file is `&'static str`
- **Macros only** for compile-time info or literal syntax; short names (`d!`, `eg!`) reserved for high-frequency use

### 4.3 Performance

- No per-call construction of reusable resources (HTTP clients, compiled patterns)
- No unnecessary allocation in hot error-free paths (`&'static str` over `String` where possible)
- Only flag performance issues with a realistic usage scenario — this is a util library; the hot path is the caller's loop

### 4.4 Code Style Rules

These are enforced project conventions — violations are findings (severity LOW):
- **No new `#[allow(...)]`** — `#![deny(warnings)]` is the law; fix at the source (documented cfg-parity exceptions aside)
- **Import grouping**: the `use crate::*;` prelude comes first, then std, then external crates; merge common prefixes
- **Macros**: `$crate::` paths (never `crate::` or unqualified sibling macros), trailing-comma support `$(,)*`
- **rustfmt**: `max_width = 79` (run `make fmt`)
- **Doc-code alignment**: public API changes must update doc comments, `README.md`,
  `doc/*.md`, `CLAUDE.md` (layout/feature/env tables), the Phase 1 mapping table
  above, and the affected `.claude/docs/patterns/` guide

## Phase 5: Reporting

### Finding Format

```
[SEVERITY] subsystem: one-line summary

WHERE: file:line_range
WHAT: Description of the issue
WHY: Why this is a problem (cite an invariant or a technical-patterns.md pattern)
FIX: Suggested fix (if clear) or questions to resolve
```

### Severity Levels

- **CRITICAL**: security hole (nonce reuse, injection, key leak), data corruption, broken round-trip
- **HIGH**: wrong results on realistic input, panic on valid input, broken feature build, resource leak
- **MEDIUM**: convention violation (`.c(d!())` missing), edge-case gap, missing error-path test
- **LOW**: style, clarity, doc drift
- **INFO**: observation or question, not necessarily a bug

### Quality Gate

Only report findings with **concrete evidence** from the code. Never report:
- Hypothetical issues without a specific triggering condition
- Style preferences not related to correctness
- "Consider" suggestions without a clear downside to the current code

Consult `.claude/docs/false-positive-guide.md` before finalizing any finding.
