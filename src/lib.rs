//!
//! # MyUtil
//!
//! A useful util-collections for Rust.
//!

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
    ($eno: expr, $info: expr) => {{
        SimpleMsg::newx($eno, file!(), line!(), $info.to_string())
    }};
    ($info: expr) => {{
        SimpleMsg::new(file!(), line!(), $info.to_string())
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
macro_rules! datetime_local {
    ($ts: expr) => {{
        crate::gen_datetime_local($ts as i64)
    }};
    () => {{
        datetime_local!($crate::ts!())
    }};
}

pub fn gen_datetime_local(ts: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(ts)
        .to_offset(time::offset!(+8))
        .format("%F %T")
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
pub fn genlog(mut e: Box<dyn MyError>) -> String {
    let pid = std::process::id();

    // 内部不能再调用`p`, 否则可能无限循环
    let ns = get_pidns(pid).unwrap();

    let mut logn = LOG_LK.lock().unwrap();
    let mut res = format!(
        "\n\x1b[31;01m{n:>0width$} [pidns: {ns}][pid: {pid}] {time}\x1b[00m",
        width = 6,
        n = logn,
        ns = ns,
        pid = pid,
        time = datetime_local!(),
    );
    res.push_str(&e.display_chain());
    *logn += 1;

    res
}

/// print log
#[inline(always)]
pub fn p(e: Box<dyn MyError>) {
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
pub fn pdie(e: Box<dyn MyError>) -> ! {
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
            as Box<dyn MyError>
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
        let l1 = || -> Result<()> { Err(eg!("Some error occur!")) };

        let l2 = || -> Result<()> { l1().c(d!()) };

        let l3 = || -> Result<()> { l2().c(d!()) };

        let l4 = || -> Result<()> { l3().c(d!()) };

        pnk!(l4());
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
