# Working on RUC

## What is this project?

RUC (Rust Util Collections) is a comprehensive utility library for Rust providing:
- **Chained error management** — `RucError` trait, `Result<T>`, `d!()`/`eg!()`/`pnk!()`/`ensure!()` macros
- **Local command execution** — `cmd::exec`, `cmd::exec_timeout`
- **Remote SSH execution** — `ssh::RemoteHost` with SFTP operations
- **IPC** — Unix abstract UDP sockets (`uau`, Linux-only)
- **HTTP** — blocking GET/POST with connection pooling
- **Algorithms** — AES-256-GCM, ED25519, Keccak-256, SHA-256, random generation
- **Encoding/Decoding** — hex, base64, zlib, zstd, JSON, MessagePack, transcode

## Design Philosophy

ruc is a **lightweight, easy-to-use, practical toolkit** — not a showcase of
maximal rigor. Keep APIs simple, docs plain, and solutions pragmatic. Never
make common call sites more verbose; prefer additive convenience over
mandatory ceremony. Reviews must not push toward over-formalization.

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

## AI workflows

[AGENTS.md](AGENTS.md) is the shared agent entrypoint. Explicitly invoke the
skills in [`.agents/skills/`](.agents/skills/); arguments are ordinary prompt
text. Each skill's `agents/openai.yaml` preserves explicit selection with
`allow_implicit_invocation: false`. Ordinary requests do not implicitly invoke
commit or version workflows.

| Skill | Behavior |
|---|---|
| [`$x-review`](.agents/skills/x-review/SKILL.md) | Review the latest commit by default; accepts a scope and `--fix`. Only the audit is writable without `--fix`; never commits. |
| [`$x-fix`](.agents/skills/x-fix/SKILL.md) | Recheck `doc/audit.md`, fix confirmed Open entries and create atomic local commits. |
| [`$x-commit`](.agents/skills/x-commit/SKILL.md) | Review and commit intended worktree changes, optionally limited by pathspecs. |
| [`$x-overhaul`](.agents/skills/x-overhaul/SKILL.md) | Review the latest commit by default (`all` for the full repository), fix confirmed findings and create atomic local commits. |

Read [workflow-policy.md](doc/ai/workflow-policy.md) for scope resolution,
worktree ownership, serial validation and local version/commit rules. Commit
workflows complete one owed version bump for tracked Rust changes since the
last version commit; docs/config-only work does not create one. They never
push, open a PR, tag or publish. Publishing remains a human action; an explicit
user request to push is separate authorization.

Supporting guides in `doc/ai/`:

- [review-core.md](doc/ai/review-core.md) — evidence, subsystem mapping and audit registry rules
- [technical-patterns.md](doc/ai/technical-patterns.md) — bug catalog; HISTORICAL entries are past defects, not current findings
- [false-positive-guide.md](doc/ai/false-positive-guide.md) — refute candidates before reporting
- [err/common](doc/ai/patterns/err-common.md), [execution](doc/ai/patterns/exec.md),
  [algorithms](doc/ai/patterns/algo.md) and [encoding](doc/ai/patterns/ende.md) — subsystem contracts

[doc/audit.md](doc/audit.md) tracks confirmed outstanding findings. Its historical
empty state is not a fresh whole-repository audit. Preserve existing findings
and unrelated work; do not run destructive `make cleanall` or rewrite history.
Workflow instructions describe task scope and do not replace runtime permissions.

## Conventions

- `#![deny(warnings)]` and `#![deny(missing_docs)]` in `lib.rs`
- Error chaining: always use `.c(d!())`, never `.unwrap()` on `ruc::Result`
- `#[macro_export]` macros reference crate items via `$crate::`, never `crate::`
- Feature-gated modules use `#![allow(missing_docs)]` at module level
- Use the official latest stable Rust (edition 2024) for development, CI and
  releases. Update with `rustup update stable` and select it with
  `RUSTUP_TOOLCHAIN=stable` or `cargo +stable`. Define no compatible Rust version
  range or minimum compiler; do not add `rust-version`, numeric toolchain pins
  or older-compiler compatibility shims.
- `SimpleMsg::file` is `&'static str` (from `file!()`) — zero-alloc error locations
- AES encryption always uses random 12-byte nonce prepended to ciphertext
- HTTP client is reused via `LazyLock<Client>` — do not create per-request clients
- Deprecated APIs delegate to replacements; keep for one major version before removal
- Keep repository documentation, instructions and commit messages in English; respond to the user in their language.
- Do not add `Co-Authored-By:` or similar attribution trailers to commits.

## Environment Variables

- `RUC_LOG_LEVEL` — `"INFO"` (default) or `"ERROR"`
- `RUC_SSH_TIMEOUT` — SSH timeout in seconds (default 20, max 300)
- `RUC_HTTP_TIMEOUT` — HTTP timeout in seconds (default 3, max 255)
