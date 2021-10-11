//!
//! # Execute external commands
//!

use crate::*;
use std::process::Command;

/// execute an external command,
/// and return its outputs after it exits
#[inline(always)]
pub fn exec_output(cmd: &str) -> Result<String> {
    let res = Command::new("sh").arg("-c").arg(cmd).output().c(d!())?;
    if res.status.success() {
        Ok(String::from_utf8_lossy(&res.stdout).into_owned())
    } else {
        Err(eg!(String::from_utf8_lossy(&res.stderr).into_owned()))
    }
}
