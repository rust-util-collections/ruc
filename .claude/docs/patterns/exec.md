# Pattern Guide: exec (cmd, ssh, uau, http)

Files: `src/cmd.rs` (feature `cmd`), `src/ssh.rs` (`ssh`), `src/uau.rs` (`uau`, Linux-only), `src/http.rs` (`http`)
Theme: talking to the outside world — processes, remote hosts, sockets, HTTP. All blocking by design.

## Invariants

### INV-CMD: Process Lifecycle
- `exec` runs `bash -c <cmd>`; injection is the **caller's documented responsibility** (docs must say "Do not pass unsanitized user input") — see false-positive-guide FP-2
- `exec_timeout` polls in 100ms steps; on timeout or error it MUST `child.kill()` and `child.wait()` (reap) before returning (technical-patterns 2.2)
- `timeout_milliseconds == 0` degrades to plain `exec`
- stdout returned via `from_utf8_lossy` → `String` (documented lossy; FP-8); non-zero exit returns `Err(eg!(stderr))`

### INV-SSH: Remote Execution Safety
- Public-key auth ONLY — never add password/agent auth silently
- **Never build remote shell strings from paths/args** — the `cat <path>` injection was a real bug (technical-patterns 2.1); use the SFTP methods (`read_file`, `replace_file`, `append_file`, `put_file`, `get_file`) for file ops
- Timeout: `RUC_SSH_TIMEOUT` (u32 secs), default 20, clamped `.min(300)`, bounds connect, handshake/auth (`* 1000` ms), and channel idle draining — clamp direction was a real bug (technical-patterns 3.1)
- `exec_cmd` returns `Vec<u8>` (byte-exact) — do not change to lossy `String`
- Session/TcpStream lifecycle: handshake failures must not leak connections

### INV-UAU: Abstract Socket Semantics (Linux-only)
- Abstract namespace = leading NUL in `sun_path`; no filesystem cleanup needed, and the fd is closed automatically by `OwnedFd` on drop
- Receivers must bind an explicit address; anonymous senders get no replies (documented)
- `recv_buf::<N>`/`recvonly_buf::<N>`: a datagram larger than `N` returns an error (v11.0.0; previously silently truncated) — the low-level `recv(buf)` still truncates silently by design (technical-patterns 6.2)
- Module is `#[cfg(target_os = "linux")]` gated — never let it leak into macOS/BSD builds

### INV-HTTP: Client Reuse & Timeout
- `HTTP_CLI: LazyLock<Client>` singleton — one pool for the process; per-request `ClientBuilder` is a historical bug (technical-patterns 7.2)
- `TIME_OUT: LazyLock<Duration>` — `RUC_HTTP_TIMEOUT` parsed as **u8** (cap 255, default 3); out-of-range values fall back to 3, not 255 (technical-patterns 3.2)
- Both statics are read-once: changing env after first request has no effect (documented behavior)
- `http1_only` is set deliberately; header params use `&str` (non-`'static`) lifetimes

## Review Checklist

- [ ] Any string reaching `bash -c` / remote channel built from variables? (CRITICAL if yes and not the documented `exec` primitives)
- [ ] Every timeout/error early-return kills and reaps spawned children?
- [ ] Clamps read correctly: upper = `.min`, lower = `.max`?
- [ ] New env knobs: parsed type documents its natural cap; fallback documented; added to `CLAUDE.md`?
- [ ] fd/session/socket released on all paths (Drop or explicit)?
- [ ] uau code stays inside the Linux cfg gate; oversized-datagram behavior documented (`recv_buf` errors, `recv` truncates)?
- [ ] No per-call client/session construction where a singleton or reuse exists?
- [ ] Timeout tests use generous margins (CI is slow) and stay serial-safe?
