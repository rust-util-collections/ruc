# MyUtil

A simple and friendly `error-chain`, with many useful utils as an addition.

The painful experience of using `error-chain` gave birth to this project. It can adapt to almost all scenes without any special implementation.

## Documents

```shell
make doc
```

## Usage

> **Tips**: you can easily implement your own `Error` type.

```rust
use myutil::{err::*, *};

fn will_panic() {
    let l1 = || -> Result<()> { Err(eg!(-9, "The final error message!")) };
    let l2 = || -> Result<()> { l1().c(d!(@-10)) };
    let l3 = || -> Result<()> { l2().c(d!(-11, "A custom message!")) };
    let l4 = || -> Result<()> { l3().c(d!()) };
    let l5 = || -> Result<()> { l4().c(d!()) };
    let l6 = || -> Result<()> { l5().c(d!()) };
    let l7 = || -> Result<()> { l6().c(d!(@-12)) };

    pnk!(l7());
}
```

## OutPut Sample

#### nocolor (features = "ansi")

```
# 2021-01-05 11:51:16 [idx: 0] [pid: 14939] [pidns: 4026531836]
Error:
|-- eno: -1
|-- file: src/lib.rs
|-- line: 362
`-- column: 9
Caused By:
|-- eno: -12
|-- file: src/lib.rs
|-- line: 360
`-- column: 44
    Caused By:
    |-- eno: -1
    |-- file: src/lib.rs
    |-- line: 359
    `-- column: 44
        Caused By:
        |-- eno: -1
        |-- file: src/lib.rs
        |-- line: 358
        `-- column: 44
            Caused By:
            |-- eno: -1
            |-- file: src/lib.rs
            |-- line: 357
            `-- column: 44
                Caused By: A custom message!
                |-- eno: -11
                |-- file: src/lib.rs
                |-- line: 356
                `-- column: 44
                    Caused By:
                    |-- eno: -10
                    |-- file: src/lib.rs
                    |-- line: 355
                    `-- column: 44
                        Caused By: The final error message!
                        |-- eno: -9
                        |-- file: src/lib.rs
                        |-- line: 354
                        `-- column: 41
```

#### colorful

```
# 2021-01-05 11:51:16 [idx: 0] [pid: 15182] [pidns: 4026531836]
Error:
├── eno: -1
├── file: src/lib.rs
├── line: 362
└── column: 9
Caused By:
├── eno: -12
├── file: src/lib.rs
├── line: 360
└── column: 44
    Caused By:
    ├── eno: -1
    ├── file: src/lib.rs
    ├── line: 359
    └── column: 44
        Caused By:
        ├── eno: -1
        ├── file: src/lib.rs
        ├── line: 358
        └── column: 44
            Caused By:
            ├── eno: -1
            ├── file: src/lib.rs
            ├── line: 357
            └── column: 44
                Caused By: A custom message!
                ├── eno: -11
                ├── file: src/lib.rs
                ├── line: 356
                └── column: 44
                    Caused By:
                    ├── eno: -10
                    ├── file: src/lib.rs
                    ├── line: 355
                    └── column: 44
                        Caused By: The final error message!
                        ├── eno: -9
                        ├── file: src/lib.rs
                        ├── line: 354
                        └── column: 41
```
