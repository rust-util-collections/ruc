# Remote command execution and file transfer

Remote command execution and SFTP file transfer over the SSH protocol,
- required features: `ssh` or `full`
- only public key authentication is supported
- server host keys are not verified; strict server identity verification is
  outside the intended scope of this auxiliary helper
- `RUC_SSH_TIMEOUT` bounds connect, handshake/authentication, and idle
  output draining (default 20s, max 300s)
