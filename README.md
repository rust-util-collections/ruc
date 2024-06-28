![GitHub top language](https://img.shields.io/github/languages/top/rust-util-collections/RUC)
[![Rust](https://github.com/rust-util-collections/ruc/actions/workflows/rust.yml/badge.svg)](https://github.com/rust-util-collections/ruc/actions/workflows/rust.yml)
[![Latest Version](https://img.shields.io/crates/v/RUC.svg)](https://crates.io/crates/RUC)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/RUC)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.78+-lightgray.svg)

# ruc

Rust Util Collection, components included:

- Chained error management
- Local command execution based on rust standard library
  - required features: `cmd`
- Remote command execution based on the SSH protocol
  - required features: `ssh`
- Interprocess Communication Based on Unix Abstract Sockets
  - required features: `uau`
  - only available on various Linux platforms
  - the built-in functions only support the UDP protocol
- Crypto operations
  - required features: `crypto`
- ...

In addition, there is a feature named "full", using it will enable all functional features.

### Documentations

```shell
# cargo doc --all-features --open
make doc
```

Links to static documentations:
- [Chained error management](doc/errmgmt.md)
- [Local command execution](doc/cmd.md)
- [Remote command execution](doc/ssh.md)
- [Interprocess Communication](doc/uau.md)

### Gratitude

Thanks to all the people who already contributed!

<a href="https://github.com/rust-util-collections/ruc/graphs/contributors">
  <img src="https://contributors-img.web.app/image?repo=rust-util-collections/ruc"/>
</a>
