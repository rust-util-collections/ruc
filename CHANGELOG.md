# Change log

#### v7.x

- Reorganize modules
- Add new toolkits
    - http(no https support)
    - compress/uncompress: zlib
    - encode/decode: message pack, json, etc.

#### v6.x

- Remove the trie related functions
- Rename the `SerDe` feature to `ser_de`

#### v5.x

- Add more crypto/codec utils
- Remove features related to `no_std` and `wasm32`

#### v4.x

- Add new moduler: hash
- Add new feature: `no_std`

#### v3.x

- Add support for remote command execution based on the SSH protocol
- Remove syntactic sugar for printing information in the 'Debug' format
    - eg, `d!(@a_struct)` should be replaced with `d!("{:?}", a_struct)` in v3.0.x
- Optimize documentations
