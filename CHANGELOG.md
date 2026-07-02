# Change log

#### v11.x

- **Breaking** remove APIs deprecated since v10.0
    - `alt!`, `map!{B ...}`, `set!{B ...}` (use `bmap!`/`bset!`)
    - `rand_jwt`/`rand_jwt_n` (use `rand_hex`/`rand_hex_n`)
    - `UauSock::recv_64/..._1024`, `recvonly_64/..._1024` (use `recv_buf::<N>`/`recvonly_buf::<N>`)
- **Breaking** `UauSock::recv_buf::<N>` now errors on datagrams larger than `N` instead of silently truncating
- **Breaking** `RucResult` gains a required `c_with` method (only affects external impls of the trait)
- Security: `ssh::replace_file`/`put_file` switched from SCP to SFTP (no remote shell involved)
- Fix: ssh timeout now also bounds connect/handshake/auth (previously unbounded)
- Fix: ssh command output draining can no longer deadlock on window-filling stderr
- Fix: `cmd::exec_timeout` kills AND reaps timed-out children (no more zombies)
- Fix: `map!`/`set!` no longer evaluate element expressions twice
- Fix: `gen_datetime` saturates out-of-range timestamps instead of panicking
- Fix: `UauSock` no longer double-closes its fd
- Fix: collection macros and `info!`/`pnk!` now work via full paths (`ruc::bmap!{...}`) without extra imports
- Add: `.c_with(|| d!(...))` — lazy error context, zero cost on the `Ok` path
- Add: `zlib_uncompress_bounded`/`zstd_uncompress_bounded` for untrusted input

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
