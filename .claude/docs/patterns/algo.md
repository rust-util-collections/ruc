# Pattern Guide: algo (rand, hash, crypto)

Files: `src/algo/rand.rs` (`algo_rand`), `src/algo/hash/keccak/` (`algo_keccak`), `src/algo/hash/sha256/` (`algo_sha256`), `src/algo/crypto/aes.rs` (`algo_aes`), `src/algo/crypto/ed25519/` (`algo_ed25519`)
Theme: security-critical. Findings here default one severity level UP.

## Invariants

### INV-AES: AES-256-GCM Nonce Safety
- Every `encrypt()` generates a **fresh random 12-byte nonce**; output layout = `nonce(12) || ciphertext`
- `decrypt()` extracts the first 12 bytes as nonce; input `< 12` bytes MUST error, not panic
- Key derivation: `Keccak256(password) → 32-byte AES key`
- Fixed/zero nonce was a real v10.0.0 bug (technical-patterns 1.1) — highest-priority check in this file
- `*_to_base64` / `*_from_base64` variants must round-trip with the raw variants

### INV-ED25519: Key Handling
- Two layers: `origin/` (raw `ed25519_dalek::SigningKey`/`VerifyingKey`) and `readable/` (base64-wrapped `SignKey`/`VerifyKey`/`Sig` with serde)
- `TryFrom<String>` validates decoded length: 32 bytes (keys), 64 bytes (signatures) — error, never panic (technical-patterns 1.3)
- `create_keypair()` uses OS-seeded RNG (technical-patterns 1.2)
- Shared decode logic lives in the `decode_bytes::<N>` helper — do not re-inline per-type copies (that duplication was refactored away in d8d4858)
- Serde representations must stay stable — they are a wire format

### INV-HASH: Digest Discipline
- `hash(input)` single-input; `hash_msg(&[&[u8]])` multi-part — multi-part must equal hashing the concatenation ONLY if documented; otherwise domain separation is deliberate — check which before "fixing"
- Output types are fixed arrays: `KeccakHash = [u8; 32]`, `Sha256Hash = [u8; 32]` — never `Vec<u8>`
- Empty input is valid and must produce the algorithm's canonical empty digest

### INV-RAND: Random Generation
- `rand_hex()` default size / `rand_hex_n(n)` custom; `rand_data(len)` raw bytes
- `rand_jwt`/`rand_jwt_n` are deprecated delegators to `rand_hex*` — keep as one-line delegation (technical-patterns 8.1)
- Randomness from OS entropy; output uniformity: hex output length semantics (chars vs bytes) must stay documented

## Security Review Rules (mandatory for any diff here)

1. Trace every nonce/key/salt from creation to use — is it fresh, random, correctly sized?
2. Any comparison of secrets (MACs, signatures) — constant-time? (GCM tag check is inside `aes-gcm`; don't hand-roll)
3. Any new error message — does it echo key/password/plaintext material? (technical-patterns 1.4)
4. Any format change (nonce position, base64 alphabet, serde layout) — breaks decryption of existing data ⇒ BREAKING, requires major bump
5. Round-trip tests exist including empty input and wrong-key/corrupted-input failure cases?

## Review Checklist

- [ ] Nonce fresh-per-call, prepended, length-validated on decrypt?
- [ ] Key/sig lengths validated with `Result` errors?
- [ ] RNG is OS-seeded — no test seeds in production paths?
- [ ] Deprecated items delegate, not fork?
- [ ] Feature deps intact in Cargo.toml (`algo_aes` → `algo_hash`+`ende_base64`+`rand`; `algo_ed25519` → `ende_base64`+`ende_hex`+`rand`+`serde`)?
- [ ] No secret material in errors/logs/Debug?
