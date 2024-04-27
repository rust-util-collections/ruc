//!
//! # UAU
//!
//! Unix(Unix Domain Socket) Abstract Udp
//!
//! An abstract socket address is distinguished (from a pathname socket)
//! by the fact that sun_path[0] is a null byte ('\0').
//! The socket's address in this namespace is given
//! by the additional bytes in sun_path that are covered
//! by the specified length of the address structure.
//! Null bytes in the name have no special  significance.
//! The name has no connection with filesystem pathnames.
//! When the address of an abstract socket is returned,
//! the returned addrlen is greater than sizeof(sa_family_t) (i.e., greater than 2),
//! and the name of the socket is contained in the first (addrlen - sizeof(sa_family_t)) bytes of sun_path.
//!
//! `man unix(7)` for more infomation.
//!

use crate::*;
use nix::{
    sys::{
        socket::{
            bind, recvfrom, sendto, setsockopt, socket, sockopt,
            AddressFamily, MsgFlags, SockFlag, SockType, UnixAddr,
        },
        time::{TimeVal, TimeValLike},
    },
    unistd::close,
};
use std::os::{fd::OwnedFd, unix::io::AsRawFd};

/// Wrap raw data
pub struct UauSock {
    fd: OwnedFd,
    sa: UnixAddr,
}

impl Drop for UauSock {
    fn drop(&mut self) {
        info_omit!(close(self.fd.as_raw_fd()));
    }
}

impl UauSock {
    /// NOTE:
    ///
    /// The Unix Socket that needs to receive messages must be bound to an explicit address;
    /// If you send msg anonymously, you will not receive the reply msg from the peer.
    ///
    /// So the `addr` parameter should not be empty.
    pub fn new(addr: &[u8], recv_timeout: Option<i64>) -> Result<Self> {
        let fd = socket(
            AddressFamily::Unix,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        )
        .c(d!())?;

        setsockopt(&fd, sockopt::ReuseAddr, &true).c(d!())?;
        setsockopt(&fd, sockopt::ReusePort, &true).c(d!())?;
        if let Some(to) = recv_timeout {
            setsockopt(
                &fd,
                sockopt::ReceiveTimeout,
                &TimeVal::milliseconds(to),
            )
            .c(d!())?;
        }

        let sa = UnixAddr::new_abstract(addr).c(d!())?;
        bind(fd.as_raw_fd(), &sa).c(d!())?;

        Ok(UauSock { fd, sa })
    }

    /// Generate a random instance
    #[inline(always)]
    pub fn gen(recv_timeout: Option<i64>) -> Result<Self> {
        let addr = (ts!() as u32 ^ rand::random::<u32>()).to_ne_bytes();
        Self::new(&addr, recv_timeout).c(d!())
    }

    /// Get the addr of UauSock
    #[inline(always)]
    pub fn addr(&self) -> &UnixAddr {
        &self.sa
    }

    /// Send msg to another peer
    #[inline(always)]
    pub fn send(&self, msg: &[u8], peeraddr: &UnixAddr) -> Result<()> {
        sendto(self.fd.as_raw_fd(), msg, peeraddr, MsgFlags::empty())
            .c(d!())
            .map(|_| ())
    }

    /// Receive msg with a 64-bytes buffer
    #[inline(always)]
    pub fn recv_64(&self) -> Result<(Vec<u8>, UnixAddr)> {
        let mut buf = [0u8; 64];
        self.recv(&mut buf)
            .c(d!())
            .map(|(n, peer)| (buf[..n].to_vec(), peer))
    }

    /// Receive msg with a 128-bytes buffer
    #[inline(always)]
    pub fn recv_128(&self) -> Result<(Vec<u8>, UnixAddr)> {
        let mut buf = [0u8; 128];
        self.recv(&mut buf)
            .c(d!())
            .map(|(n, peer)| (buf[..n].to_vec(), peer))
    }

    /// Receive msg with a 256-bytes buffer
    #[inline(always)]
    pub fn recv_256(&self) -> Result<(Vec<u8>, UnixAddr)> {
        let mut buf = [0u8; 256];
        self.recv(&mut buf)
            .c(d!())
            .map(|(n, peer)| (buf[..n].to_vec(), peer))
    }

    /// Receive msg with a 512-bytes buffer
    #[inline(always)]
    pub fn recv_512(&self) -> Result<(Vec<u8>, UnixAddr)> {
        let mut buf = [0u8; 512];
        self.recv(&mut buf)
            .c(d!())
            .map(|(n, peer)| (buf[..n].to_vec(), peer))
    }

    /// Receive msg with a 1024-bytes buffer
    #[inline(always)]
    pub fn recv_1024(&self) -> Result<(Vec<u8>, UnixAddr)> {
        let mut buf = [0u8; 1024];
        self.recv(&mut buf)
            .c(d!())
            .map(|(n, peer)| (buf[..n].to_vec(), peer))
    }

    /// Receive msg with a 64-bytes buffer
    #[inline(always)]
    pub fn recvonly_64(&self) -> Result<Vec<u8>> {
        self.recv_64().map(|(b, _)| b)
    }

    /// Receive msg with a 128-bytes buffer
    #[inline(always)]
    pub fn recvonly_128(&self) -> Result<Vec<u8>> {
        self.recv_128().map(|(b, _)| b)
    }

    /// Receive msg with a 256-bytes buffer
    #[inline(always)]
    pub fn recvonly_256(&self) -> Result<Vec<u8>> {
        self.recv_256().map(|(b, _)| b)
    }

    /// Receive msg with a 512-bytes buffer
    #[inline(always)]
    pub fn recvonly_512(&self) -> Result<Vec<u8>> {
        self.recv_512().map(|(b, _)| b)
    }

    /// Receive msg with a 1024-bytes buffer
    #[inline(always)]
    pub fn recvonly_1024(&self) -> Result<Vec<u8>> {
        self.recv_1024().map(|(b, _)| b)
    }

    /// Receive msg with a given buffer
    #[inline(always)]
    pub fn recv(&self, buf: &mut [u8]) -> Result<(usize, UnixAddr)> {
        match recvfrom::<UnixAddr>(self.fd.as_raw_fd(), buf) {
            Ok((n, Some(peer))) => Ok((n, peer)),
            Err(e) => Err(eg!(e)),
            _ => Err(eg!("peer address is unknown")),
        }
    }

    /// Try to convert a user-given addr to SockAddr(unix sock)
    pub fn addr_to_sock(addr: &[u8]) -> Result<UnixAddr> {
        UnixAddr::new_abstract(addr).c(d!())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn t_send_recv() {
        let sender = pnk!(UauSock::gen(None));
        let receiver = pnk!(UauSock::gen(None));

        pnk!(sender.send(&987654321_u32.to_ne_bytes()[..], receiver.addr()));
        assert_eq!(
            &987654321_u32.to_ne_bytes()[..],
            &pnk!(receiver.recvonly_64())
        );

        pnk!(sender.send(&987654321_u32.to_ne_bytes()[..], receiver.addr()));
        assert_eq!(
            &987654321_u32.to_ne_bytes()[..],
            &pnk!(receiver.recvonly_128())
        );

        pnk!(sender.send(&987654321_u32.to_ne_bytes()[..], receiver.addr()));
        assert_eq!(
            &987654321_u32.to_ne_bytes()[..],
            &pnk!(receiver.recvonly_256())
        );

        pnk!(sender.send(&987654321_u32.to_ne_bytes()[..], receiver.addr()));
        assert_eq!(
            &987654321_u32.to_ne_bytes()[..],
            &pnk!(receiver.recvonly_512())
        );

        pnk!(sender.send(&987654321_u32.to_ne_bytes()[..], receiver.addr()));
        assert_eq!(
            &987654321_u32.to_ne_bytes()[..],
            &pnk!(receiver.recvonly_1024())
        );
    }
}
