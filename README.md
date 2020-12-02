# MyUtil

A simple and friendly `error-chain`, with many useful utils as an addition.

The painful experience of using `error-chain` gave birth to this project. It can adapt to almost all scenes without any special implementation.

## Usage

> **Tips**: you can easily implement your own `Error` type.

```rust
use myutil::{err::*, *};

fn will_panic() {
    let l1 = || -> Result<()> { Err(eg!("Some error occur!")) };
    let l2 = || -> Result<()> { l1().c(d!()) };
    let l3 = || -> Result<()> { l2().c(d!()) };
    let l4 = || -> Result<()> { l3().c(d!()) };

    pnk!(l4());
}
```

# OutPut Sample

```shell
000000 [pidns: 4026531836][pid: 29729] 2020-12-02 14:47:21
Error:
├── eno: -1
├── file: src/lib.rs
└── line: 334
Caused By:
├── eno: -1
├── file: src/lib.rs
└── line: 332
    Caused By:
    ├── eno: -1
    ├── file: src/lib.rs
    └── line: 330
        Caused By:
        ├── eno: -1
        ├── file: src/lib.rs
        └── line: 328
            Caused By: Some error occur!
            ├── eno: -1
            ├── file: src/lib.rs
            └── line: 326
```
