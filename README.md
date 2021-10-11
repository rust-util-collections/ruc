![](https://tokei.rs/b1/github/FindoraNetwork/ruc)
![GitHub top language](https://img.shields.io/github/languages/top/FindoraNetwork/ruc)
![GitHub issues](https://img.shields.io/github/issues-raw/FindoraNetwork/ruc)
![GitHub pull requests](https://img.shields.io/github/issues-pr-raw/FindoraNetwork/ruc)

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
use ruc::{err::*, *};

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
# 2021-09-09 9:23:56 [pid: 20837] [pidns: 4026531836]
ERROR: ...
|-- file: src/lib.rs
|-- line: 270
`-- column: 9
Caused By: ...
|-- file: src/lib.rs
|-- line: 264
`-- column: 16
    Caused By: CustomErr(-1)
    |-- file: src/lib.rs
    |-- line: 262
    `-- column: 44
        Caused By: ERR_UNKNOWN
        |-- file: src/lib.rs
        |-- line: 261
        `-- column: 44
            Caused By: A custom message!
            |-- file: src/lib.rs
            |-- line: 260
            `-- column: 44
                Caused By: ...
                |-- file: src/lib.rs
                |-- line: 259
                `-- column: 69
                    Caused By: The final error message!
                    |-- file: src/lib.rs
                    |-- line: 258
                    `-- column: 41
```

#### colorful

```
# 2021-09-09 9:23:57 [pid: 20909] [pidns: 4026531836]
ERROR: ...
├── file: src/lib.rs
├── line: 270
└── column: 9
Caused By: ...
├── file: src/lib.rs
├── line: 264
└── column: 16
    Caused By: CustomErr(-1)
    ├── file: src/lib.rs
    ├── line: 262
    └── column: 44
        Caused By: ERR_UNKNOWN
        ├── file: src/lib.rs
        ├── line: 261
        └── column: 44
            Caused By: A custom message!
            ├── file: src/lib.rs
            ├── line: 260
            └── column: 44
                Caused By: ...
                ├── file: src/lib.rs
                ├── line: 259
                └── column: 69
                    Caused By: The final error message!
                    ├── file: src/lib.rs
                    ├── line: 258
                    └── column: 41
```
