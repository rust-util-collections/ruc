//!
//! # SSH
//!
//! Remote command execution based on the SSH protocol.
//!
//! NOTE: only public key authentication is supported.
//!

use crate::*;
use ssh2::{Channel, FileStat, OpenFlags, OpenType, Session};
use std::{
    env, fs, io,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

type HostAddr = String;
type HostAddrRef<'a> = &'a str;

type User = String;
type UserRef<'a> = &'a str;

type Port = u16;

// Clamped `RUC_SSH_TIMEOUT` in seconds: default 20, upper bound 300.
#[inline(always)]
fn ssh_timeout_secs() -> u32 {
    env::var("RUC_SSH_TIMEOUT")
        .ok()
        .and_then(|t| info!(t.parse::<u32>(), t).ok())
        .unwrap_or(20)
        .min(300)
}

// Drain stdout and stderr concurrently (non-blocking interleaved reads)
// so a remote process can never stall on a full SSH channel window,
// regardless of how much it writes to either stream.
//
// The timeout works as an IDLE timeout: it only fires after a full
// `RUC_SSH_TIMEOUT` window with no incoming data, so long-running
// commands that keep streaming output are never cut off.
//
// The session is switched back to blocking mode before returning.
fn drain_channel(
    sess: &Session,
    channel: &mut Channel,
) -> Result<(Vec<u8>, Vec<u8>)> {
    fn drain_stream(
        r: &mut impl Read,
        dst: &mut Vec<u8>,
        buf: &mut [u8],
    ) -> Result<bool> {
        let mut progressed = false;
        loop {
            match r.read(buf) {
                Ok(0) => return Ok(progressed), // EOF (for now)
                Ok(n) => {
                    dst.extend_from_slice(&buf[..n]);
                    progressed = true;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    return Ok(progressed);
                }
                Err(e) => return Err(eg!(e)),
            }
        }
    }

    let idle_timeout = Duration::from_secs(ssh_timeout_secs() as u64);
    let mut deadline = Instant::now() + idle_timeout;
    let mut stdout = vec![];
    let mut stderr = vec![];
    let mut buf = [0u8; 16 * 1024];

    sess.set_blocking(false);
    let ret = loop {
        let out_progress = match drain_stream(channel, &mut stdout, &mut buf) {
            Ok(p) => p,
            Err(e) => break Err(e),
        };
        let err_progress =
            match drain_stream(&mut channel.stderr(), &mut stderr, &mut buf) {
                Ok(p) => p,
                Err(e) => break Err(e),
            };

        if out_progress || err_progress {
            deadline = Instant::now() + idle_timeout;
        } else {
            if channel.eof() {
                break Ok((stdout, stderr));
            }
            if deadline < Instant::now() {
                break Err(eg!("channel-drain timeout(no data incoming)"));
            }
            sleep_ms!(1);
        }
    };
    sess.set_blocking(true);

    ret.c(d!())
}

/// Config an instance ref.
#[derive(Debug)]
pub struct RemoteHost<'a> {
    /// The address of the remote host, eg, "8.8.8.8".
    pub addr: HostAddrRef<'a>,
    /// The user name of the remote host, eg, "bob".
    pub user: UserRef<'a>,
    /// The sshd listening port of the remote host.
    pub port: Port,
    /// Path list of the ssh secret keys(rsa/ed25519 key).
    pub local_sk: &'a Path,
}

impl RemoteHost<'_> {
    fn id(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.addr,
            self.user,
            self.port,
            self.local_sk.to_str().unwrap_or_default()
        )
    }

    fn gen_session(&self) -> Result<Session> {
        let timeout = ssh_timeout_secs();

        let mut sess = Session::new().c(d!())?;
        let endpoint = format!("{}:{}", &self.addr, self.port);
        // try every resolved address (e.g. both v6 and v4 of a hostname)
        let mut tcp = Err(eg!("no address resolved from `{}`", &endpoint));
        for addr in endpoint.to_socket_addrs().c(d!(&endpoint))? {
            match TcpStream::connect_timeout(
                &addr,
                Duration::from_secs(timeout as u64),
            ) {
                Ok(s) => {
                    tcp = Ok(s);
                    break;
                }
                Err(e) => tcp = Err(eg!("{}: {}", addr, e)),
            }
        }
        sess.set_tcp_stream(tcp.c(d!(&endpoint))?);

        // bound handshake/auth blocking time BEFORE performing them
        sess.set_timeout(timeout * 1000);
        sess.set_blocking(true);

        sess.handshake().c(d!()).and_then(|_| {
            let p = PathBuf::from(self.local_sk);
            sess.userauth_pubkey_file(self.user, None, p.as_path(), None)
                .c(d!())
        })?;

        Ok(sess)
    }

    /// Execute a cmd on a remote host and get its stdout;
    /// stderr is embedded in the error on non-zero exit.
    ///
    /// The `cmd` string is passed directly to the remote shell.
    /// Do not pass unsanitized user input.
    pub fn exec_cmd(&self, cmd: &str) -> Result<Vec<u8>> {
        let sess = self.gen_session().c(d!())?;
        let mut channel = sess.channel_session().c(d!())?;
        channel.exec(cmd).c(d!())?;
        channel.send_eof().c(d!())?;
        let (stdout, stderr) = drain_channel(&sess, &mut channel).c(d!())?;
        channel.wait_eof().c(d!())?;
        channel.close().c(d!())?;
        channel.wait_close().c(d!())?;

        let stderr = String::from_utf8_lossy(&stderr);
        match channel.exit_status() {
            Ok(code) => {
                if 0 == code {
                    Ok(stdout)
                } else {
                    Err(eg!(
                        "STDOUT: {}; STDERR: [{}] {stderr}",
                        String::from_utf8_lossy(&stdout),
                        self.id(),
                    ))
                }
            }
            Err(e) => {
                info!(Err(eg!(
                    "STDOUT: {}; STDERR: [{}] {stderr}\n{}",
                    String::from_utf8_lossy(&stdout),
                    self.id(),
                    e,
                )))
            }
        }
    }

    /// Execute a cmd on a remote host and get its exit code.
    ///
    /// The `cmd` string is passed directly to the remote shell.
    /// Do not pass unsanitized user input.
    pub fn exec_exit_code(&self, cmd: &str) -> Result<i32> {
        let sess = self.gen_session().c(d!())?;
        let mut channel = sess.channel_session().c(d!())?;
        channel.exec(cmd).c(d!())?;
        channel.send_eof().c(d!())?;
        // drain both streams so the remote process is never
        // blocked on a full channel window
        drain_channel(&sess, &mut channel).c(d!())?;
        channel.wait_eof().c(d!())?;
        channel.close().c(d!())?;
        channel.wait_close().c(d!())?;

        channel.exit_status().c(d!(self.id()))
    }

    /// Get the attributes of a file based on the SFTP protocol
    pub fn file_stat<P: AsRef<Path>>(&self, path: P) -> Result<FileStat> {
        let sess = self.gen_session().c(d!())?;
        let sftp = sess.sftp().c(d!())?;
        sftp.stat(path.as_ref()).c(d!(self.id()))
    }

    /// Read the contents of a target file from the remote host via SFTP.
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let sess = self.gen_session().c(d!())?;
        let sftp = sess.sftp().c(d!())?;
        let mut file = sftp.open(path.as_ref()).c(d!(self.id()))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).c(d!())?;
        Ok(buf)
    }

    /// Fill the target file on the remote host with the local contents
    /// (create it if absent, truncate it if present), via SFTP.
    pub fn replace_file<P: AsRef<Path>>(
        &self,
        remote_path: P,
        contents: &[u8],
    ) -> Result<()> {
        let sess = self.gen_session().c(d!(self.id()))?;
        let sftp = sess.sftp().c(d!())?;
        let mut remote_file = sftp
            .open_mode(
                remote_path.as_ref(),
                OpenFlags::CREATE | OpenFlags::WRITE | OpenFlags::TRUNCATE,
                0o644,
                OpenType::File,
            )
            .c(d!(self.id()))?;
        remote_file.write_all(contents).c(d!(self.id()))?;
        remote_file.fsync().c(d!())
    }

    /// Write(append) local contents to the target file on the remote host
    pub fn append_file<P: AsRef<Path>>(
        &self,
        remote_path: P,
        contents: &[u8],
    ) -> Result<()> {
        let sess = self.gen_session().c(d!(self.id()))?;
        let sftp = sess.sftp().c(d!())?;
        let mut remote_file = sftp
            .open_mode(
                remote_path.as_ref(),
                OpenFlags::CREATE | OpenFlags::WRITE | OpenFlags::APPEND,
                0o644,
                OpenType::File,
            )
            .c(d!())?;
        remote_file.write_all(contents).c(d!(self.id()))?;
        remote_file.fsync().c(d!())
    }

    /// Send a local file to the target path on the remote host.
    #[inline(always)]
    pub fn put_file<LP: AsRef<Path>, RP: AsRef<Path>>(
        &self,
        local_path: LP,
        remote_path: RP,
    ) -> Result<()> {
        self.scp(local_path, remote_path, true).c(d!())
    }

    /// Download a remote file to a local path.
    #[inline(always)]
    pub fn get_file<RP: AsRef<Path>, LP: AsRef<Path>>(
        &self,
        remote_path: RP,
        local_path: LP,
    ) -> Result<()> {
        self.scp(local_path, remote_path, false).c(d!())
    }

    /// Copy files between local host and the remote host.
    pub fn scp<LP: AsRef<Path>, RP: AsRef<Path>>(
        &self,
        local_path: LP,
        remote_path: RP,
        direction_is_out: bool,
    ) -> Result<()> {
        if direction_is_out {
            fs::read(local_path.as_ref()).c(d!()).and_then(|contents| {
                self.replace_file(remote_path, &contents).c(d!())
            })
        } else {
            self.read_file(remote_path)
                .c(d!())
                .and_then(|contents| fs::write(local_path, contents).c(d!()))
        }
    }
}

/// Config an owned instance.
#[derive(Debug)]
pub struct RemoteHostOwned {
    /// The address of the remote host, eg, "8.8.8.8".
    pub addr: HostAddr,
    /// The user name of the remote host, eg, "bob".
    pub user: User,
    /// The sshd listening port of the remote host.
    pub port: Port,
    /// Path list of the ssh secret keys(rsa/ed25519 key).
    pub local_sk: PathBuf,
}

impl RemoteHostOwned {
    /// Create a new instance with default port and key path.
    #[inline(always)]
    pub fn new_default(addr: HostAddr, remote_user: User) -> Result<Self> {
        let home = env::var("HOME").c(d!())?;
        let rsa_key_path = PathBuf::from(format!("{}/.ssh/id_rsa", &home));
        let ed25519_key_path = PathBuf::from(home + "/.ssh/id_ed25519");

        let local_sk;
        if ed25519_key_path.exists() {
            local_sk = ed25519_key_path;
        } else if rsa_key_path.exists() {
            local_sk = rsa_key_path;
        } else {
            return Err(eg!(
                "Private key not found, neither RSA nor ED25519."
            ));
        };

        Ok(Self {
            addr,
            user: remote_user,
            port: 22,
            local_sk,
        })
    }
}

impl<'a> From<&'a RemoteHostOwned> for RemoteHost<'a> {
    fn from(o: &'a RemoteHostOwned) -> RemoteHost<'a> {
        Self {
            addr: o.addr.as_str(),
            user: o.user.as_str(),
            port: o.port,
            local_sk: o.local_sk.as_path(),
        }
    }
}
