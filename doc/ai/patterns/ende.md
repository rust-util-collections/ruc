# Pattern Guide: ende (encode/decode)

Files: `src/ende/{hex,base64,compress,zstd,json,msgpack,transcode}.rs` (features `ende_*`)
Theme: symmetric data transforms. The one law: **everything round-trips**.

## Invariants

### INV-ENDE: Round-Trip Symmetry
- `decode(encode(x)) == x` for hex/base64; `uncompress(compress(x)) == x` for zlib/zstd; msgpack/json serde round-trips through typed values
- Every new codec function ships WITH its inverse and a round-trip test (technical-patterns 6.1), including the empty input (`b""`)
- Naming: `encode`/`decode` (or `json_encode`/`json_decode` prefixed), `{algo}_compress`/`{algo}_uncompress` — never mix `de`/`un` prefixes within a pair

### INV-JSON/MSGPACK: Serde Discipline
- Bounds: `T: Serialize` for encode, `T: for<'a> Deserialize<'a>` (HRTB) for decode
- `json_encode_str` returns `String`; byte APIs return `Vec<u8>` — keep the split
- The msgpack dependency is `rmp-serde` aliased as `rmp` in Cargo.toml — use `rmp::` paths

### INV-TRANSCODE: Bidirectional Streaming
- `transcode` converts JSON↔MessagePack via `serde-transcode` streaming — both directions exist (`convert_json_to_msgpack` and `convert_msgpack_to_json`)
- Serializer must be flushed before taking the inner buffer (`.into_inner().flush()`)
- JSON input is one complete document: trailing whitespace is accepted, but
  a second value or trailing garbage must error (`Deserializer::end()`).
- Transcoding is structure-preserving: numbers/strings/maps survive both directions; if a lossy case exists (e.g., non-string map keys JSON can't express), it must be documented

### INV-COMPRESS: Level & Format Stability
- zlib via `flate2`, zstd via `zstd` — compression *level* changes are observable (output bytes differ) but legal; *format* changes (magic, framing) are BREAKING
- Decompression of hostile input must not OOM-panic uncontrollably where the underlying crate offers limits — errors flow through `.c(d!())`

## Review Checklist

- [ ] New/changed codec has matching inverse + round-trip test incl. empty input?
- [ ] Corrupted-input decode returns `Err` (never panic)? Test exists?
- [ ] Generic bounds follow the serde discipline (HRTB on decode)?
- [ ] Naming symmetric per convention?
- [ ] Feature isolation: each `ende_*` compiles alone; `ende_transcode` declares `ende_json`+`ende_msgpack`?
- [ ] Output-format changes flagged as breaking (persisted data compatibility)?
