# MyUtil

A simple and friendly `error-chain`, with many useful utils as an addition.

The painful experience of using `error-chain` gave birth to this project. It can adapt to almost all scenes without any special implementation.

## Usage

> **Tips**: you can easily implement your own `Error` type.

```rust
use myutil::{err::*, *};

fn will_panic() {
    let l1 = || -> Result<()> { Err(eg!(-9, "The final error message!")) };
    let l2 = || -> Result<()> { l1().c(d!(@-10)) };
    let l3 = || -> Result<()> { l2().c(d!(-11, "A custom message!")) };
    let l4 = || -> Result<()> { l3().c(d!(@-12)) };

    pnk!(l4());
}
```

# OutPut Sample

```shell
000000 [pidns: 4026531836][pid: 64518] 2020-12-29 10:52:52
Error:
├── eno: -1
├── file: src/lib.rs
├── line: 342
└── column: 9
Caused By:
├── eno: -12
├── file: src/lib.rs
├── line: 340
└── column: 44
    Caused By: A custom message!
    ├── eno: -11
    ├── file: src/lib.rs
    ├── line: 339
    └── column: 44
        Caused By:
        ├── eno: -10
        ├── file: src/lib.rs
        ├── line: 338
        └── column: 44
            Caused By: The final error message!
            ├── eno: -9
            ├── file: src/lib.rs
            ├── line: 337
            └── column: 41
```
