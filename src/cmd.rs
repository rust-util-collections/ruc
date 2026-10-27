//!
//! # cmd
//!
//! Synchronous local command execution via `bash -c`.
//!

use crate::*;
use std::{
    io,
    process::Command,
    time::{Duration, Instant},
};
use subprocess::{Exec, Job, Redirection};
#[cfg(unix)]
use subprocess::{ExecExt, JobExt};

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

// Signal the group before reaping its leader, so the PID cannot be reused.
fn kill_and_reap(job: &Job) {
    #[cfg(unix)]
    info_omit!(job.send_signal_group(libc::SIGKILL));
    info_omit!(job.kill());
    info_omit!(job.wait());
}

/// Execute an external command via `bash -c` and return its stdout,
/// or an error if it fails or exceeds the timeout.
///
/// A `timeout_milliseconds` of 0 disables the timeout.
/// On non-zero exit, the returned error carries the command's stderr.
/// Both output streams are drained within the timeout, including pipes
/// inherited by background processes. On Unix, timeout/error cleanup kills
/// the shell's process group; elsewhere it kills the shell itself.
///
/// The `cmd` string is passed directly to the shell.
/// Do not pass unsanitized user input.
pub fn exec_timeout(cmd: &str, timeout_milliseconds: u64) -> Result<String> {
    if 0 == timeout_milliseconds {
        return exec(cmd).c(d!());
    }

    let command = Exec::cmd("bash")
        .args(["-c", cmd])
        .stdin(Redirection::Null)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe);
    #[cfg(unix)]
    let command = command.setpgid();
    let mut job = command.start().c(d!())?;
    let started = Instant::now();
    let timeout = Duration::from_millis(timeout_milliseconds);
    let ret = (|| {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        // Limit each batch so continuously readable pipes cannot hide
        // the deadline. Short waits also avoid Instant overflow for u64::MAX.
        let mut comm = job.communicate().c(d!())?.limit_size(64 * 1024);
        loop {
            let remaining = timeout
                .checked_sub(started.elapsed())
                .c(d!("Process time out!"))?;
            comm = comm.limit_time(remaining.min(Duration::from_secs(1)));
            let before = stdout.len() + stderr.len();
            match comm.read_to(&mut stdout, &mut stderr) {
                Ok(()) if before == stdout.len() + stderr.len() => break,
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {}
                Err(e) => return Err(e).c(d!()),
            }
        }
        let status = loop {
            let remaining = timeout
                .checked_sub(started.elapsed())
                .c(d!("Process time out!"))?;
            if let Some(status) = job
                .wait_timeout(remaining.min(Duration::from_secs(1)))
                .c(d!())?
            {
                break status;
            }
        };
        if status.success() {
            Ok(String::from_utf8_lossy(&stdout).into_owned())
        } else {
            Err(eg!(String::from_utf8_lossy(&stderr).into_owned()))
        }
    })();
    if ret.is_err() {
        kill_and_reap(&job);
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_exec_timeout() {
        assert!(super::exec_timeout("sleep 0.3", 100).is_err());
        assert!(super::exec_timeout("sleep 0.3", 500).is_ok());
    }

    #[test]
    fn t_exec_timeout_large_output() {
        let output = pnk!(exec_timeout(
            "head -c 1048576 /dev/zero; head -c 1048576 /dev/zero >&2",
            5000,
        ));
        assert_eq!(output.as_bytes(), vec![0u8; 1048576]);
        let err = exec_timeout("head -c 1048576 /dev/zero >&2; exit 1", 5000)
            .unwrap_err();
        assert_eq!(err.get_lowest_msg().as_bytes(), vec![0u8; 1048576]);
    }

    #[test]
    fn t_exec_timeout_inherited_pipes() {
        let started = Instant::now();
        assert!(exec_timeout("sleep 3 &", 100).is_err());
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn t_exec_timeout_closed_pipes() {
        let started = Instant::now();
        assert!(exec_timeout("exec >/dev/null 2>&1; sleep 3", 100).is_err());
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn t_exec_timeout_streaming() {
        let started = Instant::now();
        assert!(
            exec_timeout("while :; do printf 'output'; done", 100).is_err()
        );
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn t_exec_timeout_disabled_and_max() {
        assert_eq!(pnk!(exec_timeout("printf zero", 0)), "zero");
        assert_eq!(pnk!(exec_timeout("printf max", u64::MAX)), "max");
    }

    #[cfg(unix)]
    #[test]
    fn t_cleanup_kills_descendants() {
        use std::io::Read;

        let marker = std::env::temp_dir()
            .join(format!("ruc-cmd-descendant-{}", std::process::id()));
        let mut job = Exec::cmd("bash")
            .args([
                "-c",
                "(printf ready; sleep 0.5; printf survived > \"$1\") & wait",
                "ruc-test",
            ])
            .arg(marker.as_os_str())
            .stdout(Redirection::Pipe)
            .setpgid()
            .start()
            .unwrap();
        let mut ready = [0u8; 5];
        job.stdout.as_mut().unwrap().read_exact(&mut ready).unwrap();
        assert_eq!(&ready, b"ready");
        kill_and_reap(&job);
        sleep_ms!(700);
        let survived = marker.exists();
        let _ = std::fs::remove_file(marker);
        assert!(!survived, "a descendant survived process-group cleanup");
    }
}
