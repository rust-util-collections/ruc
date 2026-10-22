//!
//! # cmd
//!
//! Local command execution based on the Rust standard library.
//!

use crate::*;
use std::process::{Child, Command, Stdio};

/// Execute an external command via `bash -c` and return its stdout.
///
/// On non-zero exit, the returned error carries the command's stderr.
///
/// The `cmd` string is passed directly to the shell.
/// Do not pass unsanitized user input.
#[inline(always)]
pub fn exec(cmd: &str) -> Result<String> {
    let ret = Command::new("bash").arg("-c").arg(cmd).output().c(d!())?;
    if ret.status.success() {
        Ok(String::from_utf8_lossy(&ret.stdout).into_owned())
    } else {
        Err(eg!(String::from_utf8_lossy(&ret.stderr).into_owned()))
    }
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

// Kill the child and reap it, avoiding zombie/orphan processes.
#[inline(always)]
fn kill_and_reap(child: &mut Child) {
    info_omit!(child.kill());
    info_omit!(child.wait());
}

/// Execute an external command via `bash -c` and return its stdout,
/// or an error if it fails or exceeds the timeout.
///
/// A `timeout_milliseconds` of 0 disables the timeout.
/// On non-zero exit, the returned error carries the command's stderr.
///
/// The `cmd` string is passed directly to the shell.
/// Do not pass unsanitized user input.
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
        match child.try_wait() {
            Ok(Some(_)) => {
                let o = child.wait_with_output().c(d!())?;
                if o.status.success() {
                    return Ok(String::from_utf8_lossy(&o.stdout).into_owned());
                } else {
                    return Err(eg!(
                        String::from_utf8_lossy(&o.stderr).into_owned()
                    ));
                }
            }
            Ok(None) => {
                // Status not ready yet
                if 0 < try_times {
                    try_times -= 1;
                    sleep_ms!(100);
                } else {
                    kill_and_reap(&mut child);
                    return Err(eg!("Process time out!"));
                }
            }
            Err(e) => {
                kill_and_reap(&mut child);
                return Err(eg!(e));
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
