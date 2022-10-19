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
#[cfg(not(target_arch = "wasm32"))]
pub mod cmd;

#[cfg(feature = "ssh")]
#[cfg(not(target_arch = "wasm32"))]
pub mod ssh;

#[cfg(feature = "uau")]
#[cfg(not(target_arch = "wasm32"))]
#[cfg(target_os = "linux")]
pub mod uau;

pub use err::*;
