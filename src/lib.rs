//!
//! # RUC
//!
//! Useful util-collections for Rust.
//!

#![deny(warnings)]
#![deny(missing_docs)]

pub mod common;
pub mod err;

#[cfg(feature = "cmd")]
pub mod cmd;

#[cfg(feature = "ssh")]
pub mod ssh;

#[cfg(feature = "uau")]
#[cfg(target_os = "linux")]
pub mod uau;

pub use err::*;

#[cfg(feature = "crypto")]
pub mod crypto;
