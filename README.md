![GitHub top language](https://img.shields.io/github/languages/top/ccmlm/RUC)
[![Latest Version](https://img.shields.io/crates/v/RUC.svg)](https://crates.io/crates/RUC)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/RUC)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ccmlm/RUC/Rust)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.63+-lightgray.svg)

# RUC

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
- ...

In addition, there is a feature named "full", using it will enable all functional features.

## Documentations

```shell
# cargo doc --all-features --open
make doc
```

Links to static documentations:
- [Chained error management](doc/errmgmt.md)
- [Local command execution](doc/cmd.md)
- [Remote command execution](doc/ssh.md)
- [Interprocess Communication](doc/uau.md)
