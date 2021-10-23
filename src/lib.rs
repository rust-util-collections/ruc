//!
//! # RUC
//!
//! A useful util-collections for Rust.
//!
//! ## Example
//!
//! ```rust
//! use ruc::{err::*, *};
//!
//! #[derive(Debug, Eq, PartialEq)]
//! struct CustomErr(i32);
//!
//! fn will_panic() {
//!     let l1 = || -> Result<()> { Err(eg!("The final error message!")) };
//!     let l2 = || -> Result<()> { l1().c(d!()) };
//!     let l3 = || -> Result<()> { l2().c(d!("A custom message!")) };
//!     let l4 = || -> Result<()> { l3().c(d!("ERR_UNKNOWN")) };
//!     let l5 = || -> Result<()> { l4().c(d!(@CustomErr(-1))) };
//!
//!     pnk!(l5());
//! }
//! ```

#![deny(warnings)]
#![warn(missing_docs, unused_import_braces, unused_extern_crates)]

#[cfg(feature = "cmd")]
pub mod cmd;
pub mod err;
#[cfg(feature = "uau")]
#[cfg(not(target_arch = "wasm32"))]
#[cfg(target_os = "linux")]
pub mod uau;

pub use err::*;

/// map operations
#[macro_export]
macro_rules! map {
    () => {{
        std::collections::HashMap::new()
    }};
    (B) => {{
        std::collections::BTreeMap::new()
    }};
    ($(||)+) => {{
        std::collections::HashMap::new
    }};
    (B $(||)+) => {{
        std::collections::BTreeMap::new
    }};
    ($($k: expr => $v: expr),+ $(,)*) => {{
        let mut m = std::collections::HashMap::with_capacity([$(&$k),*].len());
        $(m.insert($k, $v);)*
        m
    }};
    (B $($k: expr => $v: expr),+ $(,)*) => {{
        let mut m = map! {B};
        $(m.insert($k, $v);)*
        m
    }};
}

/// vector operations
#[macro_export]
macro_rules! vct {
    () => {
        Vec::new()
    };
    ($(||)+) => {
        Vec::new
    };
    ($($v: expr),+ $(,)*) => {{
        vec![$($v),*]
    }};
    ($elem:expr; $n:expr) => {{
        vec![$elem; $n]
    }};
}

/// optimize readable in high-level-functions
#[macro_export]
macro_rules! alt {
    ($condition: expr, $ops: block, $ops2: block) => {{
        if $condition $ops else $ops2
    }};
    ($condition: expr, $ops: block) => {{
        if $condition $ops
    }};
    ($condition: expr, $ops: expr, $ops2: expr) => {{
        if $condition { $ops } else { $ops2 }
    }};
    ($condition: expr, $ops: expr) => {{
        if $condition { $ops }
    }};
}

/// print infomation only
#[macro_export]
macro_rules! info {
    ($ops: expr) => {{
        $ops.c($crate::d!()).map_err(|e| {
            e.print(Some("INFO"));
            e
        })
    }};
    ($ops: expr, $msg: expr) => {{
        $ops.c($crate::d!($msg)).map_err(|e| {
            e.print(Some("INFO"));
            e
        })
    }};
}

/// omit the result without printing any message
#[macro_export]
macro_rules! omit {
    ($ops: expr) => {{
        let _ = $ops;
    }};
}

/// drop the result afeter printing the message
#[macro_export]
macro_rules! info_omit {
    ($ops: expr) => {{
        $crate::omit!($crate::info!($ops));
    }};
    ($ops: expr, $msg: expr) => {{
        $crate::omit!($crate::info!($ops, $msg));
    }};
}

/// print debug-info, eg: modular and file path, line number ...
#[macro_export]
macro_rules! d {
    ($err: expr) => {{
        $crate::err::SimpleMsg::new($err, file!(), line!(), column!())
    }};
    (@$err: expr) => {{
        $crate::d!(format!("{:?}", $err))
    }};
    () => {{
        $crate::d!("...")
    }};
}

/// print msg those impl `fmt::Display`
#[macro_export]
macro_rules! pd {
    ($x: expr) => {{
        eprintln!("\n{}", $crate::d!($x));
    }};
}

/// get current UTC-timestamp
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! ts {
    () => {{
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }};
}

/// get current UTC-timestamp
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! ts {
    () => {
        0
    };
}

/// get current DateTime
#[macro_export]
macro_rules! datetime {
    ($ts: expr) => {{
        $crate::gen_datetime($ts as i64)
    }};
    () => {{
        $crate::datetime!($crate::ts!())
    }};
}

/// generate a 'formated DateTime'
#[cfg(not(target_arch = "wasm32"))]
#[inline(always)]
pub fn gen_datetime(ts: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(ts).format("%F %T")
}

/// generate a 'formated DateTime'
#[cfg(target_arch = "wasm32")]
#[inline(always)]
pub fn gen_datetime(_ts: i64) -> String {
    "0000-00-00 00:00:00".to_owned()
}

/// Just a panic
#[macro_export]
macro_rules! die {
    ($e:expr) => {{
        $crate::pd!($e);
        $crate::die!();
    }};
    () => {
        panic!();
    };
}

/// Print log, and panic
#[macro_export]
macro_rules! pnk {
    ($ops: expr) => {{
        $ops.c($crate::d!()).unwrap_or_else(|e| e.print_die())
    }};
    ($ops: expr, $msg: expr) => {{
        $ops.c($crate::d!($msg)).unwrap_or_else(|e| e.print_die())
    }};
    (@$ops: expr) => {{
        $ops.c($crate::d!()).unwrap_or_else(|e| e.print_die_debug())
    }};
    (@$ops: expr, $msg: expr) => {{
        $ops.c($crate::d!($msg))
            .unwrap_or_else(|e| e.print_die_debug())
    }};
}

/// Sleep in milliseconds
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! sleep_ms {
    ($n: expr) => {{
        std::thread::sleep(std::time::Duration::from_millis($n));
    }};
}

/// Generate error with debug info
#[macro_export]
macro_rules! eg {
    ($msg: expr) => {{
        Box::new($crate::err::SimpleError::new($crate::d!($msg), None))
            as Box<dyn $crate::err::RucError>
    }};
    (@$msg: expr) => {
        $crate::eg!(format!("{:#?}", $msg))
    };
    () => {
        $crate::eg!("...")
    };
}

/// find the max value of multi values
#[macro_export]
macro_rules! max {
    ($v1: expr, $v2: expr) => {{
        alt!($v1 > $v2, { $v1 }, { $v2 })
    }};
    ($v1: expr, $v2:expr, $($v: expr),+ $(,)*) => {{
        let len = 2 + [$(&$v),*].len();
        let mut m = Vec::with_capacity(len);
        m.push($v1);
        m.push($v2);
        $(m.push($v);)*
        m.sort_unstable();
        m[len - 1]
    }};
}

/// find the min value of multi values
#[macro_export]
macro_rules! min {
    ($v1: expr, $v2: expr) => {{
        alt!($v1 > $v2, { $v2 }, { $v1 })
    }};
    ($v1: expr, $v2:expr, $($v: expr),+ $(,)*) => {{
        let len = 2 + [$(&$v),*].len();
        let mut m = Vec::with_capacity(len);
        m.push($v1);
        m.push($v2);
        $(m.push($v);)*
        m.sort_unstable();
        m[0]
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t_display_style_inner() -> Result<()> {
        #[derive(Debug, Eq, PartialEq)]
        struct CustomErr(i32);

        let l1 = || -> Result<()> { Err(eg!("The final error message!")) };
        let l2 = || -> Result<()> { l1().c(d!()).or_else(|e| l1().c(d!(e))) };
        let l3 = || -> Result<()> { l2().c(d!("A custom message!")) };
        let l4 = || -> Result<()> { l3().c(d!("ERR_UNKNOWN")) };
        let l5 = || -> Result<()> { l4().c(d!(@CustomErr(-1))) };

        l5().c(d!())
    }

    #[test]
    #[should_panic]
    fn t_display_style() {
        pnk!(t_display_style_inner());
    }

    #[test]
    #[should_panic]
    fn t_display_style_debug() {
        pnk!(@t_display_style_inner());
    }

    #[test]
    fn t_macro() {
        let s1 = map! {1 => 2, 2 => 4};
        let s2 = map! {B 1 => 2, 2 => 4};
        assert_eq!(s1.len(), s2.len());
        for (idx, (k, v)) in s2.into_iter().enumerate() {
            assert_eq!(1 + idx, k);
            assert_eq!(2 * k, v);
        }

        let _ = info!(Err::<u8, _>(eg!()));
        omit!(Err::<u8, _>(eg!()));
        info_omit!(Err::<u8, _>(eg!()));
        pd!(ts!());

        sleep_ms!(1);
    }

    #[test]
    fn t_max_min() {
        assert_eq!(10, max!(1, 10));
        assert_eq!(10, max!(1, 2, 3, 4, 5, 10));
        assert_eq!(1, min!(1, 10));
        assert_eq!(1, min!(1, 2, 3, 4, 5, 10));
    }
}
