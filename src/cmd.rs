//!
//! # cmd
//!
//! Local command execution based on rust standard library
//!

use crate::*;
use std::process::{Child, Command, Output, Stdio};

enum PollRet {
    Ready(Output),
    NotReady(Child),
}

/// Execute an external command,
/// and return its outputs after it exits
#[inline(always)]
pub fn exec(cmd: &str) -> Result<String> {
    let ret = Command::new("bash").arg("-c").arg(cmd).output().c(d!())?;
    if ret.status.success() {
        Ok(String::from_utf8_lossy(&ret.stdout).into_owned())
    } else {
        Err(eg!(String::from_utf8_lossy(&ret.stderr).into_owned()))
    }
}

/// Execute an external command,
/// and return its outputs after it exits
#[deprecated(since = "9.0.0", note = "please use `exec` instead")]
#[inline(always)]
pub fn exec_output(cmd: &str) -> Result<String> {
    exec(cmd).c(d!())
}

#[inline(always)]
fn exec_spawn(cmd: &str) -> Result<Child> {
    Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .c(d!())
}

#[inline(always)]
fn exec_spawn_poll(mut child: Child) -> Result<PollRet> {
    match child.try_wait() {
        Ok(Some(_)) => Ok(PollRet::Ready(child.wait_with_output().c(d!())?)),
        Ok(None) => Ok(PollRet::NotReady(child)), // Status not ready yet
        Err(e) => Err(eg!(e)),                    // Error occur!
    }
}

/// Execute an external command,
/// and return its outputs after `exit`,
/// or panic with a `timeout` error
pub fn exec_timeout(cmd: &str, timeout_milliseconds: u64) -> Result<String> {
    if 0 == timeout_milliseconds {
        return exec(cmd).c(d!());
    }

    let mut child = exec_spawn(cmd).c(d!())?;
    let mut try_times = timeout_milliseconds / 100;

    let rm = timeout_milliseconds % 100;
    let pre_wait_time = if 0 < rm {
        rm
    } else {
        try_times -= 1;
        100
    };
    sleep_ms!(pre_wait_time);

    loop {
        match exec_spawn_poll(child).c(d!())? {
            PollRet::Ready(o) => {
                if o.status.success() {
                    return Ok(String::from_utf8_lossy(&o.stdout).into_owned());
                } else {
                    return Err(eg!(
                        String::from_utf8_lossy(&o.stderr).into_owned()
                    ));
                }
            }
            PollRet::NotReady(c) => {
                child = c;
                if 0 < try_times {
                    try_times -= 1;
                    sleep_ms!(100);
                } else {
                    info_omit!(child.kill());
                    return Err(eg!("Process time out!"));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn t_exec_timeout() {
        assert!(super::exec_timeout("sleep 0.3", 100).is_err());
        assert!(super::exec_timeout("sleep 0.3", 500).is_ok());
    }
}
