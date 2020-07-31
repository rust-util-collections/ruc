//!
//! # 通用工具集
//!

pub mod err;

use err::*;
use lazy_static::lazy_static;
use parking_lot::Mutex;

lazy_static! {
    static ref LOG_LK: Mutex<u64> = Mutex::new(0);
}

/// hashmap operations
#[macro_export]
macro_rules! map {
    () => {{
        std::collections::HashMap::new()
    }};
    ($(||)+) => {{
        std::collections::HashMap::new
    }};
    ($($k: expr => $v: expr),+ $(,)*) => {{
        let mut m = std::collections::HashMap::with_capacity([$(&$k),*].len());
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

/// 高阶函数中提搞可读性.
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

/// 用于非关键环节打印提示性信息.
#[macro_export]
macro_rules! info {
    ($ops: expr) => {{
        $ops.c($crate::d!()).map_err($crate::p)
    }};
    ($ops: expr, $msg: expr) => {{
        $ops.c($crate::d!($msg)).map_err($crate::p)
    }};
}

/// 完全忽略返回值
#[macro_export]
macro_rules! omit {
    ($ops: expr) => {{
        let _ = $ops;
    }};
}

/// 打印消息后直接丢弃返回值
#[macro_export]
macro_rules! info_omit {
    ($ops: expr) => {{
        $crate::omit!($crate::info!($ops));
    }};
    ($ops: expr, $msg: expr) => {{
        $crate::omit!($crate::info!($ops, $msg));
    }};
}

/// 打印模块路径, 文件名称, 行号等 Debug 信息
#[macro_export]
macro_rules! d {
    ($x: expr) => {{
        format!("\x1b[31m{:?}\x1b[00m\n├── \x1b[01mfile:\x1b[00m {}\n└── \x1b[01mline:\x1b[00m {}",
            $x, file!(), line!())
    }};
    () => {{
        format!("...\n├── \x1b[01mfile:\x1b[00m {}\n└── \x1b[01mline:\x1b[00m {}",
                file!(), line!())
    }};
}

/// 打印平面信息
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

/// get current native-local-datatime
#[macro_export]
macro_rules! datetime_local {
    ($ts: expr) => {{
        time::OffsetDateTime::from_unix_timestamp($ts as i64)
            .to_offset(time::offset!(+8))
            .format("%F %T")
    }};
    () => {{
        datetime_local!($crate::ts!())
    }};
}

#[inline(always)]
fn get_pidns(pid: u32) -> Result<String> {
    std::fs::read_link(format!("/proc/{}/ns/pid", pid))
        .c(crate::d!())
        .map(|p| p.to_string_lossy().into_owned())
}

/// 生成日志内容
pub fn genlog(mut e: Box<dyn MyError>) -> String {
    let pid = std::process::id();

    // 内部不能再调用`p`, 否则可能无限循环
    let ns = get_pidns(pid).unwrap();

    let mut logn = LOG_LK.lock();
    let res = format!(
        "\n\x1b[31;01m{n:>0width$} [ns: {ns}][pid: {pid}] {ts}\x1b[00m{msg}",
        width = 6,
        n = logn,
        ns = ns,
        pid = pid,
        ts = datetime_local!(),
        msg = e.display_chain(),
    );
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

/// 打印 error_chain 之后 Panic
#[inline(always)]
pub fn pdie(e: Box<dyn MyError>) -> ! {
    p(e);
    crate::die!();
}

/// 打印错误信息并 Panic
#[macro_export]
macro_rules! pnk {
    ($ops: expr) => {{
        $ops.c($crate::d!()).unwrap_or_else(|e| $crate::pdie(e))
    }};
    ($ops: expr, $msg: expr) => {{
        $ops.c($crate::d!($msg)).unwrap_or_else(|e| $crate::pdie(e))
    }};
}

/// sleep
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
    () => {{
        $crate::eg!("...")
    }};
}

/// Generate sys-error with debug info
#[macro_export]
macro_rules! eg_sys {
    () => {{
        let e = $crate::eg!($crate::get_errdesc());
        reset_errno();
        e
    }};
}

/// test assert
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

/// test assert
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

/// test assert
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

/// ok
#[macro_export]
macro_rules! ok {
    () => {{
        let ok: std::io::Result<_> = Ok(());
        ok.c(d!())
    }};
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use std::process;

    #[test]
    fn T_get_pidns() {
        let ns_name = pnk!(get_pidns(process::id()));
        assert!(1 < ns_name.len());
    }
}
