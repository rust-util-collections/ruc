//!
//! # RUC
//!
//! A useful util-collections for Rust.
//!

#![deny(warnings)]
#![warn(missing_docs, unused_import_braces, unused_extern_crates)]

pub mod err;

use err::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref LOG_LK: Mutex<u64> = Mutex::new(0);
}

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
        $ops.c($crate::d!()).map_err($crate::p)
    }};
    ($ops: expr, $msg: expr) => {{
        $ops.c($crate::d!($msg)).map_err($crate::p)
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
        SimpleMsg::new($err, file!(), line!(), column!())
    }};
    (@$err: expr) => {{
        $crate::d!(format!("{:?}", $err))
    }};
    () => {{
        $crate::d!("")
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
#[macro_export]
macro_rules! ts {
    () => {{
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }};
}

/// get current native-local-datatime(+8)
#[macro_export]
macro_rules! datetime {
    ($ts: expr) => {{
        crate::gen_datetime($ts as i64)
    }};
    () => {{
        datetime!($crate::ts!())
    }};
}

/// generate a 'formated +8 datetime'
#[inline(always)]
pub fn gen_datetime(ts: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(ts).format("%F %T")
}

#[inline(always)]
#[cfg(target_os = "linux")]
fn get_pidns(pid: u32) -> Result<String> {
    std::fs::read_link(format!("/proc/{}/ns/pid", pid))
        .c(crate::d!())
        .map(|p| {
            p.to_string_lossy()
                .trim_start_matches("pid:[")
                .trim_end_matches(']')
                .to_owned()
        })
}

#[inline(always)]
#[cfg(not(target_os = "linux"))]
fn get_pidns(_pid: u32) -> Result<String> {
    Ok("NULL".to_owned())
}

/// generate the log string
pub fn genlog(mut e: Box<dyn RucError>) -> String {
    let pid = std::process::id();

    // 内部不能再调用`p`, 否则可能无限循环
    let ns = get_pidns(pid).unwrap();

    let mut logn = LOG_LK.lock().unwrap();
    let mut res = genlog_fmt(*logn, ns, pid);
    res.push_str(&e.display_chain());
    *logn += 1;

    res
}

#[cfg(not(feature = "ansi"))]
#[inline(always)]
fn genlog_fmt(idx: u64, ns: String, pid: u32) -> String {
    format!(
        "\n\x1b[31;01m# {time} [idx: {n}] [pid: {pid}] [pidns: {ns}]\x1b[00m",
        time = datetime!(),
        n = idx,
        pid = pid,
        ns = ns,
    )
}

#[cfg(feature = "ansi")]
#[inline(always)]
fn genlog_fmt(idx: u64, ns: String, pid: u32) -> String {
    format!(
        "\n# {time} [idx: {n}] [pid: {pid}] [pidns: {ns}]",
        time = datetime!(),
        n = idx,
        pid = pid,
        ns = ns,
    )
}

/// print log
#[inline(always)]
pub fn p(e: Box<dyn RucError>) {
    eprintln!("{}", genlog(e));
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

/// panic after printing `error_chain`
#[inline(always)]
pub fn pdie(e: Box<dyn RucError>) -> ! {
    p(e);
    crate::die!();
}

/// print log, and panic
#[macro_export]
macro_rules! pnk {
    ($ops: expr) => {{
        $ops.c($crate::d!()).unwrap_or_else(|e| $crate::pdie(e))
    }};
    ($ops: expr, $msg: expr) => {{
        $ops.c($crate::d!($msg)).unwrap_or_else(|e| $crate::pdie(e))
    }};
}

/// sleep in milliseconds
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
            as Box<dyn RucError>
    }};
    (@$msg: expr) => {
        $crate::eg!(format!("{:#?}", $msg))
    };
    () => {
        $crate::eg!("...")
    };
}

/// test assert in `MyUtil` style
#[macro_export]
macro_rules! so_eq {
    ($lv: expr, $rv: expr) => {{
        let l = $lv;
        let r = $rv;
        if l != r {
            return Err($crate::eg!(format!(
                "Assert failed: {:?} == {:?}",
                l, r
            )));
        }
    }};
}

/// test assert in `MyUtil` style
#[macro_export]
macro_rules! so_ne {
    ($lv: expr, $rv: expr) => {{
        let l = $lv;
        let r = $rv;
        if l == r {
            return Err($crate::eg!(format!(
                "Assert failed: {:?} != {:?}",
                l, r
            )));
        }
    }};
}

/// test assert in `MyUtil` style
#[macro_export]
macro_rules! so_le {
    ($lv: expr, $rv: expr) => {{
        let l = $lv;
        let r = $rv;
        if l > r {
            return Err($crate::eg!(format!(
                "Assert failed: {:?} <= {:?}",
                l, r
            )));
        }
    }};
}

/// an `ok` wrapper
#[macro_export]
macro_rules! ok {
    () => {{
        let ok: std::io::Result<_> = Ok(());
        ok.c(d!())
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn t_get_pidns() {
        let ns_name = pnk!(get_pidns(process::id()));
        assert!(1 < ns_name.len());
    }

    #[test]
    #[should_panic]
    fn t_display_style() {
        #[derive(Debug, Eq, PartialEq)]
        struct CustomErr(i32);

        let l1 = || -> Result<()> { Err(eg!("The final error message!")) };
        let l2 = || -> Result<()> { l1().c(d!()) };
        let l3 = || -> Result<()> { l2().c(d!("A custom message!")) };
        let l4 = || -> Result<()> { l3().c(d!("ERR_UNKNOWN")) };
        let l5 = || -> Result<()> { l4().c(d!(@CustomErr(-1))) };

        pnk!(l5());
    }

    #[test]
    fn t_map() {
        let s1 = map! {1 => 2, 2 => 4};
        let s2 = map! {B 1 => 2, 2 => 4};
        assert_eq!(s1.len(), s2.len());
        for (idx, (k, v)) in s2.into_iter().enumerate() {
            assert_eq!(1 + idx, k);
            assert_eq!(2 * k, v);
        }
    }
}
