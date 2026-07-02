# RUC — Claude Code Project Guide

## What is this project?

RUC (Rust Util Collections) is a comprehensive utility library for Rust providing:
- **Chained error management** — `RucError` trait, `Result<T>`, `d!()`/`eg!()`/`pnk!()`/`ensure!()` macros
- **Local command execution** — `cmd::exec`, `cmd::exec_timeout`
- **Remote SSH execution** — `ssh::RemoteHost` with SFTP operations
- **IPC** — Unix abstract UDP sockets (`uau`, Linux-only)
- **HTTP** — blocking GET/POST with connection pooling
- **Algorithms** — AES-256-GCM, ED25519, Keccak-256, SHA-256, random generation
- **Encoding/Decoding** — hex, base64, zlib, zstd, JSON, MessagePack, transcode

## Build & Test

```bash
cargo build                    # default features (ansi only)
cargo build --all-features     # everything
make test                      # CI-equivalent: 3 passes, --release -- --test-threads=1
make lint                      # cargo clippy --features="full" (+ --tests)
cargo fmt --check              # rustfmt.toml: max_width = 79
cargo doc --all-features --open  # docs
```

CI runs `make build` + `make test` — the `Makefile` is the source of truth.
Tests MUST be single-threaded (`--test-threads=1`): they mutate global env vars (`src/common.rs`).

## Project Layout

```
src/
├── lib.rs              # crate root, re-exports err::*
├── err/
│   ├── mod.rs          # RucError trait, SimpleError, SimpleMsg, Result<T>
│   └── macros.rs       # d!(), eg!(), pnk!(), info!(), ensure!(), etc.
├── common.rs           # map!/bmap!/set!/bset!, ts!/ts_ms!, retry(), file utils, env_or()
├── cmd.rs              # local command execution (feature: cmd)
├── ssh.rs              # SSH remote execution (feature: ssh)
├── uau.rs              # Unix abstract UDP sockets (feature: uau, Linux-only)
├── http.rs             # HTTP GET/POST (feature: http)
├── algo/
│   ├── mod.rs
│   ├── rand.rs         # random hex/bytes generation (feature: algo_rand)
│   ├── hash/
│   │   ├── keccak/     # Keccak-256 (feature: algo_keccak)
│   │   └── sha256/     # SHA-256 (feature: algo_sha256)
│   └── crypto/
│       ├── aes.rs      # AES-256-GCM with random nonce (feature: algo_aes)
│       └── ed25519/    # ED25519 sign/verify (feature: algo_ed25519)
│           ├── origin/ # raw dalek keys
│           └── readable/ # base64-wrapped keys with serde
└── ende/
    ├── hex.rs          # hex encode/decode (feature: ende_hex)
    ├── base64.rs       # base64 encode/decode (feature: ende_base64)
    ├── compress.rs     # zlib (feature: ende_compress)
    ├── zstd.rs         # zstd (feature: ende_zstd)
    ├── json.rs         # JSON serde (feature: ende_json)
    ├── msgpack.rs      # MessagePack serde (feature: ende_msgpack)
    └── transcode.rs    # JSON↔MessagePack (feature: ende_transcode)
```

## Feature Flag Hierarchy

```
full → cmd, uau, ssh, http, algo, ende
algo → algo_crypto (algo_ed25519, algo_aes), algo_rand, algo_hash (algo_keccak, algo_sha256)
ende → ende_hex, ende_base64, ende_compress, ende_zstd, ende_json, ende_msgpack, ende_transcode
```

Output-format features (outside `full`, gate no modules): `ansi` (default, colored errors), `compact` (single-line errors). `compact` is only tested via `--features="full,compact"`.

## Commands

- `/x-review` — deep regression analysis (supports: N commits, `all`, hash, range)
- `/x-fix` — fix audit backlog: resolve `doc/audit.md` → self-review → commit
- `/x-commit` — self-reviewing commit: review uncommitted changes → fix → validate → bump patch version → commit
- `/x-overhaul` — full codebase overhaul: review all → fix → commit

Supporting documentation in `.claude/docs/`:
- `technical-patterns.md` — cataloged bug patterns (HISTORICAL entries actually happened here)
- `review-core.md` — systematic review methodology + subsystem mapping table (single source of truth)
- `false-positive-guide.md` — rules for filtering spurious findings
- `patterns/` — per-subsystem review guides (err-common, exec, algo, ende)

Additional documentation in `doc/`:
- `audit.md` — audit findings registry (auto-managed by /x-review and /x-fix)

Shared tool permissions live in `.claude/settings.json` (`cargo publish`, `git push --force`, destructive cleans are denied — publishing is a human action).

## Conventions

- `#![deny(warnings)]` and `#![deny(missing_docs)]` in `lib.rs`
- Error chaining: always use `.c(d!())`, never `.unwrap()` on `ruc::Result`
- `#[macro_export]` macros reference crate items via `$crate::`, never `crate::`
- Feature-gated modules use `#![allow(missing_docs)]` at module level
- No nightly features — stable Rust only (edition 2024, rustc 1.86+)
- `SimpleMsg::file` is `&'static str` (from `file!()`) — zero-alloc error locations
- AES encryption always uses random 12-byte nonce prepended to ciphertext
- HTTP client is reused via `LazyLock<Client>` — do not create per-request clients
- Deprecated APIs delegate to replacements; keep for one major version before removal

## Environment Variables

- `RUC_LOG_LEVEL` — `"INFO"` (default) or `"ERROR"`
- `RUC_SSH_TIMEOUT` — SSH timeout in seconds (default 20, max 300)
- `RUC_HTTP_TIMEOUT` — HTTP timeout in seconds (default 3, max 255)
