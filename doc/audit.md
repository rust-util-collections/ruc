# Audit Findings

> Managed by `$x-review`, `$x-fix`, `$x-commit` and `$x-overhaul`; see
> [registry rules](ai/review-core.md#audit-registry).

## Open

### [HIGH] ssh: server host keys are never verified

- Location: `src/ssh.rs`, `RemoteHost::gen_session`.
- Contract: server identity must be authenticated before remote execution or
  file transfer can be trusted.
- Trigger/consequence: a server impersonating the configured endpoint can
  complete the handshake and accept public-key authentication, then supply
  forged command/file results or receive uploaded contents.
- Guards fail because `handshake` and `userauth_pubkey_file` authenticate the
  protocol/client, while no `host_key`/`known_hosts` comparison authenticates
  the server. Every remote operation shares this session path.
- Fix/validation: choose an explicit trust policy (known-hosts file or pinned
  host key), reject unknown/mismatched keys before authentication, and test
  matching, unknown and changed keys against a local test server. Enforcing
  trust changes existing connection behavior and needs user acceptance.

---

## Won't Fix

*(none)*
