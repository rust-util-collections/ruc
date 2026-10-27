# Audit Findings

> Managed by `$x-review`, `$x-fix`, `$x-commit` and `$x-overhaul`; see
> [registry rules](ai/review-core.md#audit-registry).

## Open

*(none)*

---

## Won't Fix

### [HIGH] ssh: server host keys are never verified

- Location: `src/ssh.rs`, `RemoteHost::gen_session`.
- Security limitation: the helper does not authenticate the server's identity.
- Trigger/consequence: a server impersonating the configured endpoint can
  complete the handshake and accept public-key authentication, then supply
  forged command/file results or receive uploaded contents.
- Guards fail because `handshake` and `userauth_pubkey_file` authenticate the
  protocol/client, while no `host_key`/`known_hosts` comparison authenticates
  the server. Every remote operation shares this session path.
- Disposition: accepted by explicit maintainer decision. SSH is an auxiliary
  feature and strict host-key verification is outside its intended scope.
  Mandatory known-hosts or pinning configuration would reject existing
  connections and add setup disproportionate to this helper's role.
- Reconsider if the maintainer requests strict server identity verification
  or SSH becomes a security boundary. A future fix should choose a trust
  policy and test matching, unknown and changed keys before authentication.
