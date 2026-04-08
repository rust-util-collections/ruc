# RUC API Design Rules

## 1. Naming Conventions

### Functions
- **encode/decode pairs**: `encode` / `decode`, or prefixed `json_encode` / `json_decode`
- **compress/uncompress pairs**: `{algo}_compress` / `{algo}_uncompress`
- **crypto pairs**: `encrypt` / `decrypt`, with `_to_base64` / `_from_base64` variants
- **hash functions**: `hash(input)` for single, `hash_msg(slices)` for multi-input
- **random generators**: `rand_{format}()` for default size, `rand_{format}_n(n)` for custom size
- **file helpers**: `read_file` / `write_file` — thin wrappers with error chain integration

### Macros
- **Short names** for high-frequency use: `d!()`, `eg!()`, `pnk!()`, `ts!()`, `ts_ms!()`
- **Descriptive names** for less frequent use: `ensure!()`, `sleep_ms!()`, `datetime!()`
- **Collection literals**: `map!{}`, `bmap!{}`, `set!{}`, `bset!{}` — no prefix variants

### Types
- Hash outputs: `{Algo}Hash = [u8; N]` (e.g., `KeccakHash`, `Sha256Hash`)
- Error types: `SimpleError<E>`, `SimpleMsg<E>` — generic over message type
- Result alias: `Result<T> = core::result::Result<T, Box<dyn RucError>>`

## 2. Error Handling Patterns

### Core Pattern: `.c(d!())`
Every fallible operation in ruc should chain errors using `.c(d!())`:
```rust
some_operation().c(d!())                    // no extra message
some_operation().c(d!("context info"))      // with context
some_operation().c(d!("{}", variable))      // with formatted context
```

### Error Construction: `eg!()`
Create standalone errors:
```rust
Err(eg!("message"))           // simple error
Err(eg!("{}", value))         // formatted error
Err(eg!())                    // empty error (location only)
```

### Assertions: `ensure!()`
Guard conditions that return `Err` on failure:
```rust
ensure!(condition);                        // auto-generates message from expr
ensure!(condition, "must be positive");    // custom message
ensure!(x > 0, "x={} invalid", x);        // formatted message
```

### Panic on Error: `pnk!()`
Unwrap-or-die with full error chain:
```rust
pnk!(fallible_operation())
pnk!(fallible_operation(), "extra context")
```

### Rules
- **Never** use `.unwrap()` / `.expect()` on `ruc::Result` — use `pnk!()` instead
- **Always** chain with `.c(d!())` when propagating errors across function boundaries
- **Use `ensure!`** for precondition checks instead of manual `if !cond { return Err(eg!(...)) }`
- **Use `eg!()`** only for constructing new error origins, not for wrapping existing errors
- Internal helper errors (not crossing public API) may use `.c(d!())` without message

## 3. Feature Flag Organization

### Hierarchy: atom → group → full
```
full
├── cmd
├── uau (nix, rand)
├── ssh (ssh2)
├── http (reqwest)
├── algo
│   ├── algo_crypto
│   │   ├── algo_ed25519 (ed25519-dalek, ende_base64, ende_hex, rand, serde)
│   │   └── algo_aes (ende_base64, aes-gcm, algo_hash, rand)
│   ├── algo_rand (rand, ende_hex)
│   └── algo_hash
│       ├── algo_keccak (sha3)
│       └── algo_sha256 (sha2)
└── ende
    ├── ende_hex (hex)
    ├── ende_base64 (base64)
    ├── ende_compress (flate2)
    ├── ende_zstd (zstd)
    ├── ende_json (serde, serde_json)
    ├── ende_msgpack (serde, rmp)
    └── ende_transcode (serde-transcode, ende_json, ende_msgpack)
```

### Rules
- Each leaf feature maps to exactly one optional dependency (or a small cluster)
- Group features aggregate leaves — **never** add direct dependencies to group features
- `#[cfg(feature = "X")]` guards go on `pub mod` declarations, not on individual items
- Modules behind features use `#![allow(missing_docs)]` at module level
- Cross-feature dependencies (e.g., `algo_aes` needs `algo_hash`) are declared in Cargo.toml feature deps
- New features must be added to both their group and `full`

## 4. Generic Constraint Style

- **Input bytes**: prefer `impl AsRef<[u8]>` or `&[u8]` depending on context
  - `AsRef<[u8]>` when callers commonly pass `String`, `Vec<u8>`, `&str`, etc.
  - `&[u8]` when the function is internal or performance-critical
- **File paths**: `impl AsRef<Path>` for public API, `&Path` for internal
- **Serialization**: `T: Serialize` / `T: for<'a> Deserialize<'a>` (use HRTB for Deserialize)
- **Lifetimes**: avoid named lifetimes unless borrow relationships are non-obvious
  - Prefer owned `RemoteHostOwned` + `From<&RemoteHostOwned> for RemoteHost<'_>`

## 5. Macro Design Principles

- Use macros when **compile-time information** is needed: `file!()`, `line!()`, `column!()`
- Use macros for **collection literal syntax**: `map!{k => v}`
- Use functions for everything else — macros are harder to debug and don't show in IDE
- All public macros use `#[macro_export]` — they live at crate root regardless of source file
- Internal references use `$crate::` paths, never `crate::`
- Support trailing commas in variadic macros: `$(,)*`

## 6. Deprecated API Policy

- Mark with `#[deprecated(since = "X.Y.Z", note = "use `new_name` instead")]`
- Deprecated functions delegate to their replacement (never duplicate logic)
- Macro branches that can't use `#[deprecated]` get doc comments: `/// NOTE: deprecated, use X instead`
- Keep deprecated items for at least one major version before removal
- Document migration path in the deprecation `note`
