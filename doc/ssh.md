# Remote command execution and file transfer

Remote command execution and SFTP file transfer over the SSH protocol,
- required features: `ssh` or `full`
- only public key authentication is supported
- `RUC_SSH_TIMEOUT` bounds connect, handshake/authentication, and idle
  output draining (default 20s, max 300s)
