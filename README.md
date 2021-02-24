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

#[derive(Debug, Eq, PartialEq)]
struct CustomErr(i32);

fn will_panic() {
    let l1 = || -> Result<()> { Err(eg!("The final error message!")) };
    let l2 = || -> Result<()> { l1().c(d!()) };
    let l3 = || -> Result<()> { l2().c(d!("A custom message!")) };
    let l4 = || -> Result<()> { l3().c(d!("ERR_UNKNOWN")) };
    let l5 = || -> Result<()> { l4().c(d!(@CustomErr(-1))) };

    pnk!(l5());
}
```

## OutPut Sample

#### nocolor (features = "ansi")

```
# 2021-02-24 9:31:12 [idx: 0] [pid: 11843] [pidns: 4026531836]
Error:
|-- file: src/lib.rs
|-- line: 355
`-- column: 9
Caused By: CustomErr(-1)
|-- file: src/lib.rs
|-- line: 353
`-- column: 44
    Caused By: ERR_UNKNOWN
    |-- file: src/lib.rs
    |-- line: 352
    `-- column: 44
        Caused By: A custom message!
        |-- file: src/lib.rs
        |-- line: 351
        `-- column: 44
            Caused By:
            |-- file: src/lib.rs
            |-- line: 350
            `-- column: 44
                Caused By: The final error message!
                |-- file: src/lib.rs
                |-- line: 349
                `-- column: 41
```

#### colorful

```
# 2021-02-24 9:31:13 [idx: 0] [pid: 12058] [pidns: 4026531836]
Error:
├── file: src/lib.rs
├── line: 355
└── column: 9
Caused By: CustomErr(-1)
├── file: src/lib.rs
├── line: 353
└── column: 44
    Caused By: ERR_UNKNOWN
    ├── file: src/lib.rs
    ├── line: 352
    └── column: 44
        Caused By: A custom message!
        ├── file: src/lib.rs
        ├── line: 351
        └── column: 44
            Caused By:
            ├── file: src/lib.rs
            ├── line: 350
            └── column: 44
                Caused By: The final error message!
                ├── file: src/lib.rs
                ├── line: 349
                └── column: 41
```
