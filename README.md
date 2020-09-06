# myutil

A simple and friendly `error-chain`.

## Usage

```rust
use myutil::{err::*, *};

fn a() -> Result<()> {}
fn b() -> Result<()> {}
fn b() -> Result<()> {}

fn main() {
    let res = a().c(d!())
                 .and_then(|_| b().c(d!()) )
                 .and_then(|_| c().c(d!()) );
    pnk!(res);
}
```

# Sample

![myutil-error-chain](./sample.png)
