# Interprocess communication

Interprocess communication based on Unix abstract sockets,
- required features: `uau` or `full`
- Linux only
- UDP only
- `recv_buf::<N>`/`recvonly_buf::<N>` error on datagrams larger than `N`;
  the low-level `recv(buf)` truncates silently
