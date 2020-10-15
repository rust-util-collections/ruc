# myutil

A simple and friendly `error-chain`, with many useful utils as an addition.

The painful experience of using `error-chain` gave birth to this project. It can adapt to almost all scenes without any special implementation.

## Usage

> **Tips**: you can easily implement your own `Error`.

```rust
use myutil::{err::*, *};

fn will_panic() {
    let l1 = || -> Result<()> { Err(eg!("error!")) };
    let l2 = || -> Result<()> { l1().c(d!()) };
    let l3 = || -> Result<()> { l2().c(d!()) };
    let l4 = || -> Result<()> { l3().c(d!()) };

    pnk!(l4());
}
```

# OutPut Sample

```shell
000000 [pidns: NULL][pid: 46574] 2020-09-06 18:18:32
Error:
├── file: src/lib.rs
└── line: 318
Caused By:
├── file: src/lib.rs
└── line: 316
    Caused By:
    ├── file: src/lib.rs
    └── line: 314
        Caused By:
        ├── file: src/lib.rs
        └── line: 312
            Caused By: Some error occur!
            ├── file: src/lib.rs
            └── line: 310
```
