# Local command execution

Local command execution based on the Rust standard library,
- required features: `cmd` or `full`
- `exec` returns stdout; on non-zero exit the error carries stderr
- `exec_timeout` additionally bounds the run time (0 disables the timeout)
