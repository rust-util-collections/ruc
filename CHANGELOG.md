# Change log

#### v5.x

- Add more crypto utils
- Remove features related to `no_std` and `wasm32`

#### v4.x

- Add new moduler: hash
- Add new feature: `no_std`

#### v3.x

- Add support for remote command execution based on the SSH protocol
- Remove syntactic sugar for printing information in the 'Debug' format
    - eg, `d!(@a_struct)` should be replaced with `d!("{:?}", a_struct)` in v3.0.x
- Optimize documentations
