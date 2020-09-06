# myutil

A simple and friendly `error-chain`.

## Usage

```rust
use myutil::{err::*, *};

fn dog() -> Result<()> {}
fn pig() -> Result<()> {}
fn monkey() -> Result<()> {}

fn main() {
    let res = dog().c(d!())
                 .and_then(|_| pig().c(d!()) )
                 .and_then(|_| monkey().c(d!()) );
    pnk!(res);
}
```

# Sample

![myutil-error-chain](./sample.png)
