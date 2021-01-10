# RUC

Rust Util Collection, a simple and friendly `error-chain`, with many useful utils as an addition.

The painful experience of using `error-chain` gave birth to this project. It can adapt to almost all scenes without any special implementation.

## Documents

```shell
make doc
```

## Usage

> **Tips**: you can easily implement your own `Error` type.

```rust
use myutil::{err::*, *};

const ERR_UNKNOWN: i32 = -100;

fn will_panic() {
    let l1 = || -> Result<()> { Err(eg!(-9, "The final error message!")) };
    let l2 = || -> Result<()> { l1().c(d!()) };
    let l3 = || -> Result<()> { l2().c(d!(-11, "A custom message!")) };
    let l4 = || -> Result<()> { l3().c(e!(ERR_UNKNOWN)) };
    let l5 = || -> Result<()> { l4().c(d!(@-12)) };

    pnk!(l5());
}
```

## OutPut Sample

#### nocolor (features = "ansi")

```
# 2021-01-10 11:23:07 [idx: 0] [pid: 52827] [pidns: NULL]
Error:
|-- eno: -1
|-- file: src/lib.rs
|-- line: 370
`-- column: 9
Caused By:
|-- eno: -12
|-- file: src/lib.rs
|-- line: 368
`-- column: 44
    Caused By: ERR_UNKNOWN
    |-- eno: -100
    |-- file: src/lib.rs
    |-- line: 367
    `-- column: 44
        Caused By: A custom message!
        |-- eno: -11
        |-- file: src/lib.rs
        |-- line: 366
        `-- column: 44
            Caused By:
            |-- eno: -1
            |-- file: src/lib.rs
            |-- line: 365
            `-- column: 44
                Caused By: The final error message!
                |-- eno: -9
                |-- file: src/lib.rs
                |-- line: 364
                `-- column: 41
```

#### colorful

```
# 2021-01-10 11:25:14 [idx: 0] [pid: 52892] [pidns: NULL]
Error:
├── eno: -1
├── file: src/lib.rs
├── line: 370
└── column: 9
Caused By:
├── eno: -12
├── file: src/lib.rs
├── line: 368
└── column: 44
    Caused By: ERR_UNKNOWN
    ├── eno: -100
    ├── file: src/lib.rs
    ├── line: 367
    └── column: 44
        Caused By: A custom message!
        ├── eno: -11
        ├── file: src/lib.rs
        ├── line: 366
        └── column: 44
            Caused By:
            ├── eno: -1
            ├── file: src/lib.rs
            ├── line: 365
            └── column: 44
                Caused By: The final error message!
                ├── eno: -9
                ├── file: src/lib.rs
                ├── line: 364
                └── column: 41
```
