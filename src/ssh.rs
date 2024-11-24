//!
//! # SSH
//!
//! Remote command execution based on the SSH protocol.
//!
//! NOTE: only public key authentication is supported.
//!

use crate::*;
use ssh2::{FileStat, OpenFlags, OpenType, Session};
use std::{
    env, fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

type HostAddr = String;
type HostAddrRef<'a> = &'a str;

type User = String;
type UserRef<'a> = &'a str;

type Port = u16;

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

impl<'a> RemoteHost<'a> {
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
        let mut sess = Session::new().c(d!())?;
        let endpoint = format!("{}:{}", &self.addr, self.port);
        let tcp = TcpStream::connect(&endpoint).c(d!(&endpoint))?;
        sess.set_tcp_stream(tcp);
        sess.handshake().c(d!()).and_then(|_| {
            let p = PathBuf::from(self.local_sk);
            sess.userauth_pubkey_file(self.user, None, p.as_path(), None)
                .c(d!())
        })?;

        let timeout = env::var("RUC_SSH_TIMEOUT")
            .ok()
            .and_then(|t| info!(t.parse::<u32>(), t).ok())
            .unwrap_or(20);
        sess.set_timeout(timeout.max(300) * 1000);
        sess.set_blocking(true);

        Ok(sess)
    }

    /// Execute a cmd on a remote host and get its outputs.
    pub fn exec_cmd(&self, cmd: &str) -> Result<Vec<u8>> {
        let mut stdout = vec![];
        let mut stderr = String::new();

        let sess = self.gen_session().c(d!())?;
        let mut channel = sess.channel_session().c(d!())?;
        channel.exec(cmd).c(d!())?;
        channel.send_eof().c(d!())?;
        channel.read_to_end(&mut stdout).c(d!())?;
        channel.stderr().read_to_string(&mut stderr).c(d!())?;
        channel.wait_eof().c(d!())?;
        channel.close().c(d!())?;
        channel.wait_close().c(d!())?;

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

    /// Execute a cmd on a remote host and get its outputs.
    pub fn exec_exit_code(&self, cmd: &str) -> Result<i32> {
        let sess = self.gen_session().c(d!())?;
        let channel =
            sess.channel_session().c(d!()).and_then(|mut channel| {
                channel
                    .exec(cmd)
                    .c(d!())
                    .and_then(|_| channel.send_eof().c(d!()))
                    .and_then(|_| channel.wait_eof().c(d!()))
                    .and_then(|_| channel.close().c(d!()))
                    .and_then(|_| channel.wait_close().c(d!()))
                    .map(|_| channel)
            })?;

        channel.exit_status().c(d!(self.id()))
    }

    /// Get the attributes of a file based on the SFTP protocol
    pub fn file_stat<P: AsRef<Path>>(&self, path: P) -> Result<FileStat> {
        let sess = self.gen_session().c(d!())?;
        let sftp = sess.sftp().c(d!())?;
        sftp.stat(path.as_ref()).c(d!(self.id()))
    }

    /// Read the contents of a target file from the remote host.
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        path.as_ref()
            .to_str()
            .c(d!())
            .map(|p| format!("cat {p}"))
            .and_then(|cmd| self.exec_cmd(&cmd).c(d!()))
    }

    /// Fill the target file on the remote host with the local contents
    pub fn replace_file<P: AsRef<Path>>(
        &self,
        remote_path: P,
        contents: &[u8],
    ) -> Result<()> {
        let mut remote_file = self.gen_session().c(d!()).and_then(|sess| {
            sess.scp_send(
                remote_path.as_ref(),
                0o644,
                contents.len() as u64,
                None,
            )
            .c(d!(self.id()))
        })?;
        remote_file
            .write_all(contents)
            .c(d!())
            .and_then(|_| remote_file.send_eof().c(d!()))
            .and_then(|_| remote_file.wait_eof().c(d!()))
            .and_then(|_| remote_file.close().c(d!()))
            .and_then(|_| remote_file.wait_close().c(d!()))
            .c(d!(self.id()))
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
        let home = env::var("HOME").unwrap();
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
    fn from(o: &'a RemoteHostOwned) -> RemoteHost {
        Self {
            addr: o.addr.as_str(),
            user: o.user.as_str(),
            port: o.port,
            local_sk: o.local_sk.as_path(),
        }
    }
}
