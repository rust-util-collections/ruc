# RUC Module Patterns

## Module Responsibility Map

| Module | Feature | Purpose | Dependencies |
|--------|---------|---------|-------------|
| `err` | *(always)* | Error trait, chain, macros | none |
| `common` | *(always)* | Collection macros, time, retry, file I/O | `time` |
| `cmd` | `cmd` | Local shell command execution | none (uses `std::process`) |
| `ssh` | `ssh` | Remote SSH command + SFTP | `ssh2` |
| `uau` | `uau` | Unix abstract UDP sockets (Linux) | `nix`, `rand` |
| `http` | `http` | Blocking HTTP GET/POST | `reqwest` |
| `algo::rand` | `algo_rand` | Random hex strings, random bytes | `rand`, `ende::hex` |
| `algo::hash::keccak` | `algo_keccak` | Keccak-256 hash | `sha3` |
| `algo::hash::sha256` | `algo_sha256` | SHA-256 hash | `sha2` |
| `algo::crypto::aes` | `algo_aes` | AES-256-GCM encrypt/decrypt | `aes-gcm`, `rand`, `algo::hash::keccak`, `ende::base64` |
| `algo::crypto::ed25519` | `algo_ed25519` | ED25519 sign/verify | `ed25519-dalek`, `rand`, `serde`, `ende::base64`, `ende::hex` |
| `ende::hex` | `ende_hex` | Hex encode/decode | `hex` |
| `ende::base64` | `ende_base64` | Base64 encode/decode | `base64` |
| `ende::compress` | `ende_compress` | Zlib compress/uncompress | `flate2` |
| `ende::zstd` | `ende_zstd` | Zstd compress/uncompress | `zstd` |
| `ende::json` | `ende_json` | JSON serialize/deserialize | `serde`, `serde_json` |
| `ende::msgpack` | `ende_msgpack` | MessagePack serialize/deserialize | `serde`, `rmp-serde` |
| `ende::transcode` | `ende_transcode` | JSON↔MessagePack conversion | `serde-transcode`, `ende_json`, `ende_msgpack` |

## Critical Invariants

### INV-ERR: Error Chain Integrity
- Every `ruc::Result` error is a `Box<dyn RucError>` with linked cause chain
- `d!()` always captures `file!()`, `line!()`, `column!()` at the call site
- `.c(d!())` on `Result<T, ERR>` wraps foreign errors; on `Option<T>` converts None
- `SimpleMsg::file` is `&'static str` — never allocate for file paths

### INV-ENDE: Encode/Decode Symmetry
- For every `encode` there is a matching `decode` that round-trips: `decode(encode(x)) == x`
- For every `compress` there is a matching `uncompress`: `uncompress(compress(x)) == x`
- For every `encrypt` there is a matching `decrypt`: `decrypt(key, encrypt(key, x)) == x`
- JSON↔MessagePack transcode is bidirectional and round-trips through structured types

### INV-AES: AES-GCM Nonce Safety
- Every `encrypt()` call generates a **unique random 12-byte nonce**
- Nonce is prepended to ciphertext: `output = nonce(12 bytes) || ciphertext`
- `decrypt()` extracts nonce from first 12 bytes, rejects input shorter than 12 bytes
- Key derivation: `Keccak256(password) → 32-byte AES key`
- **Never** use a fixed or zero nonce

### INV-ED25519: Key Handling
- `readable::SignKey` / `VerifyKey` / `Sig` are base64-encoded wrappers
- Raw keys are `ed25519_dalek::SigningKey` / `VerifyingKey`
- `TryFrom<String>` validates key length (32 bytes for keys, 64 bytes for signatures)
- `create_keypair()` uses OS-seeded RNG

### INV-FEAT: Feature Isolation
- Each `#[cfg(feature = "X")]` module compiles **independently** with only its declared deps
- No module may reference another feature-gated module without declaring the dependency
- `default = ["ansi"]` — only ANSI coloring is on by default
- `full` aggregates all features — must include every leaf feature (directly or transitively)

### INV-HTTP: Client Reuse
- `HTTP_CLI` is a `LazyLock<Client>` singleton — one connection pool for all requests
- Timeout is read once from `RUC_HTTP_TIMEOUT` at first access
- Headers use `&str` lifetimes (not `'static`) to support runtime-built values

### INV-SSH: Timeout Bounds
- Default: 20 seconds
- Range: 1–300 seconds (`timeout.min(300)`)
- Configurable via `RUC_SSH_TIMEOUT` environment variable

## Security-Critical Code Locations

| Location | Risk | Invariant |
|----------|------|-----------|
| `algo/crypto/aes.rs` | AES nonce reuse → plaintext recovery | INV-AES |
| `algo/crypto/ed25519/origin/mod.rs` | Key generation randomness | INV-ED25519 |
| `algo/crypto/ed25519/readable/mod.rs` | Base64 key parsing, length validation | INV-ED25519 |
| `cmd.rs` | Shell injection via `bash -c` | Caller's responsibility |
| `ssh.rs` | Remote code execution | Auth via public key only |

## Anti-Patterns to Watch For

1. **`.unwrap()` on `ruc::Result`** — use `pnk!()` instead for proper error chain output
2. **Manual `if !cond { return Err(eg!(...)) }`** — use `ensure!()` instead
3. **`Nonce::default()` or fixed nonce in AES** — always random
4. **Per-request `ClientBuilder::new().build()`** — use the `HTTP_CLI` singleton
5. **`timeout.max(N)` for upper bounds** — use `.min(N)` (max sets lower bound)
6. **`file: String` in error structs** — use `&'static str` from `file!()`
7. **Feature-gated code without `#[cfg]` on the module** — breaks minimal builds
