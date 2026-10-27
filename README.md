![GitHub top language](https://img.shields.io/github/languages/top/rust-util-collections/RUC)
[![Rust](https://github.com/rust-util-collections/ruc/actions/workflows/rust.yml/badge.svg)](https://github.com/rust-util-collections/ruc/actions/workflows/rust.yml)
[![Latest Version](https://img.shields.io/crates/v/RUC.svg)](https://crates.io/crates/RUC)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/RUC)

# ruc

Rust Util Collections, components included:

- Chained error management
- Local command execution based on the Rust standard library
  - required features: `cmd`
- Remote command execution and SFTP file transfer over SSH
  - required features: `ssh`
- Interprocess communication based on Unix abstract sockets
  - required features: `uau`
  - Linux only
  - UDP only
- Algorithm operations
  - required features: `algo`
    - random generation, Keccak-256/SHA-256 hashing
    - AES-256-GCM encryption, ED25519 signatures
- HTTP client (blocking GET/POST)
  - required features: `http`
  - plain HTTP only — this crate enables no TLS backend
- Encode/Decode operations
  - required features: `ende`
    - hex, base64
    - zlib/zstd compression
    - JSON, MessagePack, and JSON↔MessagePack transcoding
- ...

In addition, the `full` feature enables all functional features above
(the output-format features `ansi` and `compact` are separate).

Use the official latest stable Rust. This repository defines no minimum
compiler or compatible Rust version range; see [the toolchain policy](doc/ai/workflow-policy.md#rust-toolchain).

### Documentation

```shell
# cargo doc --all-features --open
make doc
```

Static documentation:
- [Chained error management](https://github.com/rust-util-collections/ruc/blob/master/doc/errmgmt.md)
- [Local command execution](https://github.com/rust-util-collections/ruc/blob/master/doc/cmd.md)
- [Remote command execution](https://github.com/rust-util-collections/ruc/blob/master/doc/ssh.md)
- [Interprocess communication](https://github.com/rust-util-collections/ruc/blob/master/doc/uau.md)

#### ENV VARs

- `$RUC_LOG_LEVEL`: log verbosity, `INFO`(default) or `ERROR`
  - any other value falls back to `INFO`
  - `INFO` lets `info!`/`info_omit!` print; `ERROR` suppresses them
  - read once at first use, later changes have no effect
- `$RUC_SSH_TIMEOUT`: SSH timeout, in seconds
  - bounds connect, handshake/authentication, and idle output draining
  - Default to 20s
  - The max value is 300s(5 minutes); larger values are clamped to 300
  - Unparsable values fall back to the default(20s)
- `$RUC_HTTP_TIMEOUT`: HTTP request timeout, in seconds
  - Default to 3s
  - The max value is 255s(`u8::MAX`)
  - Unparsable or out-of-range values fall back to the default(3s)
  - Read once at the first request, later changes have no effect

### AI maintenance

[AGENTS.md](AGENTS.md) is the shared agent guide. Explicit repository workflows
live in [`.agents/skills/`](.agents/skills/): `$x-review`, `$x-fix`, `$x-commit`
and `$x-overhaul`. See [the workflow policy](doc/ai/workflow-policy.md) for
scopes, serial validation and local version/commit behavior, and
[the review guide](doc/ai/review-core.md) for subsystem contracts and evidence.

### Gratitude

Thanks to all the people who already contributed!

<a href="https://github.com/rust-util-collections/ruc/graphs/contributors">
  <img src="https://contributors-img.web.app/image?repo=rust-util-collections/ruc"/>
</a>
