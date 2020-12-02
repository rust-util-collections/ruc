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
000000 [pidns: NULL][pid: 82456] 2020-12-02 16:52:33
Error:
├── eno: -1
├── file: src/lib.rs
└── line: 338
Caused By:
├── eno: -12
├── file: src/lib.rs
└── line: 336
    Caused By: A custom message!
    ├── eno: -11
    ├── file: src/lib.rs
    └── line: 335
        Caused By:
        ├── eno: -10
        ├── file: src/lib.rs
        └── line: 334
            Caused By: The final error message!
            ├── eno: -9
            ├── file: src/lib.rs
            └── line: 333
```
