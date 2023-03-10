//!
//! # RUC
//!
//! Useful util-collections for Rust.
//!

#![cfg_attr(feature = "nostd", no_std)]
#![deny(warnings)]
#![deny(missing_docs)]

pub mod common;
pub mod err;

#[cfg(feature = "cmd")]
#[cfg(not(feature = "no_std"))]
#[cfg(not(target_arch = "wasm32"))]
pub mod cmd;

#[cfg(feature = "ssh")]
#[cfg(not(feature = "no_std"))]
#[cfg(not(target_arch = "wasm32"))]
pub mod ssh;

#[cfg(feature = "uau")]
#[cfg(not(feature = "no_std"))]
#[cfg(not(target_arch = "wasm32"))]
#[cfg(target_os = "linux")]
pub mod uau;

pub use err::*;

#[cfg(feature = "crypto")]
pub mod crypto;
