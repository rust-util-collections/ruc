# Pattern Guide: err + common (always-on core)

Files: `src/lib.rs`, `src/err/mod.rs`, `src/err/macros.rs`, `src/common.rs`
Features: always compiled; `ansi` / `compact` switch error-output formatting only.

## Invariants

### INV-ERR: Error Chain Integrity
- Every `ruc::Result` error is a `Box<dyn RucError>` with a linked cause chain
- `d!()` captures `file!()`, `line!()`, `column!()` at the macro call site — moving code between files must not hardcode locations
- `.c(d!())` on `Result<T, E>` wraps foreign errors; on `Option<T>` converts `None` into an error
- `SimpleMsg::file` is `&'static str` (from `file!()`) — zero-alloc; never change to `String`
- `RUC_LOG_LEVEL` (`"INFO"` default / `"ERROR"`) is read via env — check once-read semantics when touching log paths

### INV-FMT: ansi/compact Formatting
- `ansi` ⇒ ANSI color codes + Unicode tree glyphs (`├──`/`└──`); `!ansi` ⇒ plain ASCII (`|--`/`` `-- ``)
- `compact` ⇒ single-line output (`" 》"` delimiter, no indent) and overrides pretty glyphs
- **cfg polarity is the #1 historical bug here** (swapped branches, fixed in v10.0.0) — for every cfg pair touched, re-verify each body matches its predicate (technical-patterns 4.1)
- Neither feature gates modules nor adds deps; `full` deliberately excludes both

### INV-MACRO: Exported Macro Hygiene
- All public macros are `#[macro_export]` — they live at crate root regardless of source file
- Bodies reference crate items via `$crate::` only (technical-patterns 5.2)
- Variadic macros accept trailing commas: `$(,)*`
- Macros exist only for compile-time info (`file!()`, `line!()`) or literal syntax (`map!{}`); everything else is a function

### INV-COMMON: Helpers
- Collection literals: `map!`, `bmap!`, `set!`, `bset!` — plain and typed forms
- Time: `ts!()` (secs), `ts_ms!()` (millis), `datetime!()`, `gen_datetime()` — datetime format is cached; OS local timezone is used
- `retry(times, delay_ms, f)` — verify retry count semantics (total attempts vs retries) if touched
- `read_file`/`write_file`: `impl AsRef<Path>`, errors chained with `.c(d!())`
- `env_or(key, default)`: returns default on unset — tests for it mutate real env vars (serial-test constraint, technical-patterns 7.1)

## Review Checklist

- [ ] cfg pair bodies match predicates (`ansi`, `compact`, and their combinations)?
- [ ] New macro uses `$crate::`, supports trailing comma, has a doc comment + usage example?
- [ ] No allocation added to `d!()`/`eg!()` happy paths?
- [ ] Error `Display`/`Debug` changes keep chain output parseable (delimiter/indent honored in both compact and normal modes)?
- [ ] New env-var reads: documented in `CLAUDE.md` §Environment Variables, read-once if hot?
- [ ] Tests touching env vars are serial-safe?
- [ ] Feature graph edits in `Cargo.toml`: leaf ∈ group ∈ `full` (technical-patterns 4.3)?
