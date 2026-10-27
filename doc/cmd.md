# Local command execution

Synchronous local command execution via `bash -c`,
- required features: `cmd` or `full`
- `exec` returns stdout; on non-zero exit the error carries stderr
- `exec_timeout` additionally bounds the run time (0 disables the timeout)
- Both output streams are drained during execution; output collection shares
  the timeout, including pipes inherited by background processes
- On timeout/error, the shell is killed and reaped; on Unix its process group
  is killed too (descendants that leave the group are outside this cleanup)
- Output uses lossy UTF-8 conversion; do not pass unsanitized user input
