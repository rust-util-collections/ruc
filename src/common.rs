//!
//! # Common
//!
//! Common and lightweight utils.
//!

use crate::*;
use std::{fmt::Display, path::Path};

/// HashMap literal, e.g. `map!{1 => 2}`.
///
/// `map!(||)` returns the `HashMap::new` constructor itself.
#[macro_export]
macro_rules! map {
    () => {{
        std::collections::HashMap::new()
    }};
    ($(||)+) => {{
        std::collections::HashMap::new
    }};
    ($($k: expr => $v: expr),+ $(,)*) => {{
        let mut m = std::collections::HashMap::with_capacity(
            [$(stringify!($k)),*].len(),
        );
        $(m.insert($k, $v);)*
        m
    }};
}

/// BTreeMap literal, e.g. `bmap!{1 => 2}`.
#[macro_export]
macro_rules! bmap {
    () => {{
        std::collections::BTreeMap::new()
    }};
    ($($k: expr => $v: expr),+ $(,)*) => {{
        let mut m = $crate::bmap! {};
        $(m.insert($k, $v);)*
        m
    }};
}

/// HashSet literal, e.g. `set!{1, 2}`.
///
/// `set!(||)` returns the `HashSet::new` constructor itself.
#[macro_export]
macro_rules! set {
    () => {{
        std::collections::HashSet::new()
    }};
    ($(||)+) => {{
        std::collections::HashSet::new
    }};
    ($($k: expr),+ $(,)*) => {{
        let mut m = std::collections::HashSet::with_capacity(
            [$(stringify!($k)),*].len(),
        );
        $(m.insert($k);)*
        m
    }};
}

/// BTreeSet literal, e.g. `bset!{1, 2}`.
#[macro_export]
macro_rules! bset {
    () => {{
        std::collections::BTreeSet::new()
    }};
    ($($k: expr),+ $(,)*) => {{
        let mut m = $crate::bset! {};
        $(m.insert($k);)*
        m
    }};
}

/// find the max value of multi values
#[macro_export]
macro_rules! max {
    ($v1: expr, $v2: expr $(,)*) => {{
        let (a, b) = ($v1, $v2);
        if a > b { a } else { b }
    }};
    ($v1: expr, $v2: expr, $($v: expr),+ $(,)*) => {{
        let mut ret = $v1;
        { let v = $v2; if v > ret { ret = v; } }
        $({ let v = $v; if v > ret { ret = v; } })*
        ret
    }};
}

/// find the min value of multi values
#[macro_export]
macro_rules! min {
    ($v1: expr, $v2: expr $(,)*) => {{
        let (a, b) = ($v1, $v2);
        if a < b { a } else { b }
    }};
    ($v1: expr, $v2: expr, $($v: expr),+ $(,)*) => {{
        let mut ret = $v1;
        { let v = $v2; if v < ret { ret = v; } }
        $({ let v = $v; if v < ret { ret = v; } })*
        ret
    }};
}

/// Sleep in milliseconds
#[macro_export]
macro_rules! sleep_ms {
    ($n: expr) => {{
        std::thread::sleep(std::time::Duration::from_millis($n));
    }};
}

/// get current UTC-timestamp in seconds
#[macro_export]
macro_rules! ts {
    () => {{
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }};
}

/// get current UTC-timestamp in milliseconds
#[macro_export]
macro_rules! ts_ms {
    () => {{
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }};
}

/// Cached local UTC offset, detected once at first access.
/// Falls back to UTC if detection fails (e.g., in multi-threaded context).
static LOCAL_OFFSET: std::sync::LazyLock<time::UtcOffset> =
    std::sync::LazyLock::new(|| {
        time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC)
    });

static DATETIME_FORMAT: std::sync::LazyLock<
    Vec<time::format_description::FormatItem<'static>>,
> = std::sync::LazyLock::new(|| {
    time::format_description::parse_borrowed::<2>("[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]").unwrap()
});

/// Generate a formatted DateTime string
///
/// Timestamps outside the representable datetime range are saturated
/// to the minimum/maximum representable datetime instead of panicking.
#[inline(always)]
pub fn gen_datetime(ts: i64) -> String {
    // 1-day margin keeps the later `to_offset` conversion
    // in range for any possible local offset
    let margin = 86_400;
    let min =
        time::PrimitiveDateTime::MIN.assume_utc().unix_timestamp() + margin;
    let max =
        time::PrimitiveDateTime::MAX.assume_utc().unix_timestamp() - margin;

    // safe: ts is clamped into the representable range
    time::OffsetDateTime::from_unix_timestamp(ts.clamp(min, max))
        .unwrap()
        .to_offset(*LOCAL_OFFSET)
        .format(&*DATETIME_FORMAT)
        .unwrap()
}

/// get current DateTime
#[macro_export]
macro_rules! datetime {
    ($ts: expr) => {{ $crate::common::gen_datetime($ts as i64) }};
    () => {{ $crate::datetime!($crate::ts!()) }};
}

/// Get an environment variable or return a default value
#[inline(always)]
pub fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

/// Read a file to string, wrapping errors with ruc error chain
#[inline(always)]
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    std::fs::read_to_string(path.as_ref()).c(d!())
}

/// Write contents to a file, wrapping errors with ruc error chain
#[inline(always)]
pub fn write_file<P: AsRef<Path>>(
    path: P,
    content: impl AsRef<[u8]>,
) -> Result<()> {
    std::fs::write(path.as_ref(), content).c(d!())
}

/// Retry a fallible operation with fixed delay between attempts.
///
/// Returns the first successful result, or the last error wrapped in ruc error chain.
pub fn retry<T, E, F>(times: usize, delay_ms: u64, mut f: F) -> Result<T>
where
    F: FnMut() -> core::result::Result<T, E>,
    E: Display + Send + 'static,
{
    let total = times.max(1);
    let mut last_err = None;
    for i in 0..total {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = Some(e);
                if i + 1 < total && delay_ms > 0 {
                    sleep_ms!(delay_ms);
                }
            }
        }
    }
    Err(eg!(last_err.unwrap()))
}

/// Ensure a condition is true, otherwise return an error.
///
/// Similar to `anyhow::ensure!` but returns `ruc::Result`.
///
/// # Examples
///
/// ```
/// use ruc::*;
///
/// fn check(x: i32) -> Result<()> {
///     ensure!(x > 0);
///     ensure!(x < 100, "x must be less than 100");
///     ensure!(x != 42, "x must not be {}", 42);
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {{
        if !$cond {
            return Err($crate::eg!($fmt, $($arg)*));
        }
    }};
    ($cond:expr, $msg:expr) => {{
        $crate::ensure!($cond, "{}", $msg)
    }};
    ($cond:expr) => {{
        $crate::ensure!($cond, concat!("ensure failed: ", stringify!($cond)))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_max_min() {
        assert_eq!(10, max!(1, 10,));
        assert_eq!(10, max!(1, 2, 3, 4, 5, 10));
        assert_eq!(1, min!(1, 10));
        assert_eq!(1, min!(1, 2, 3, 4, 5, 10));
    }

    #[test]
    fn t_sleep_ms() {
        sleep_ms!(1);
    }

    #[test]
    fn t_map_set() {
        let mut a = set! {};
        a.insert(1);
        assert_eq!(a.len(), 1);

        let b = set! {1, 2, 3};
        assert_eq!(b.len(), 3);

        let b = bset! {1, 2, 3};
        assert_eq!(b.len(), 3);
    }

    // element expressions must be evaluated exactly once
    #[test]
    fn t_map_set_single_evaluation() {
        let mut i = 0;
        let m = map! {
            { i += 1; i } => 10,
            { i += 1; i } => 20,
        };
        assert_eq!(i, 2);
        assert_eq!(m.len(), 2);
        assert_eq!(m[&1], 10);
        assert_eq!(m[&2], 20);

        let mut j = 0;
        let s = set! { { j += 1; j }, { j += 1; j } };
        assert_eq!(j, 2);
        assert_eq!(s.len(), 2);
        assert!(s.contains(&1) && s.contains(&2));
    }

    #[cfg(unix)]
    #[test]
    fn t_local_offset() {
        let output = std::process::Command::new("date")
            .arg("+%z")
            .output()
            .expect("failed to execute `date`");
        let tz_str = std::str::from_utf8(&output.stdout).unwrap().trim();

        // Parse "+0800" or "-0500" format
        let sign: i32 = if tz_str.starts_with('-') { -1 } else { 1 };
        let hours: i32 = tz_str[1..3].parse().unwrap();
        let minutes: i32 = tz_str[3..5].parse().unwrap();
        let expected = sign * (hours * 3600 + minutes * 60);

        let detected = super::LOCAL_OFFSET.whole_seconds();
        assert_eq!(
            detected, expected,
            "LOCAL_OFFSET ({}s) does not match OS timezone ({})",
            detected, tz_str
        );
    }

    #[test]
    fn t_bmap_bset() {
        let m1 = bmap! {"a" => 1};
        assert_eq!(m1.len(), 1);
        assert_eq!(m1.get("a"), Some(&1));

        let m2 = bmap! {"a" => 1, "b" => 2};
        assert_eq!(m2.len(), 2);
        assert_eq!(m2.get("b"), Some(&2));

        let m3 = bmap! {
            "a" => 1,
            "b" => 2,
            "c" => 3,
        };
        assert_eq!(m3.len(), 3);
        assert_eq!(m3.get("c"), Some(&3));

        let s1 = bset! {1};
        assert_eq!(s1.len(), 1);
        assert!(s1.contains(&1));

        let s2 = bset! {1, 2};
        assert_eq!(s2.len(), 2);
        assert!(s2.contains(&2));

        let s3 = bset! {1, 2, 3,};
        assert_eq!(s3.len(), 3);
        assert!(s3.contains(&3));
    }

    #[test]
    fn t_ts_ms() {
        let ms = ts_ms!();
        let s = ts!();
        // ms should be roughly s * 1000 (within 2 seconds)
        assert!(ms / 1000 >= s - 2);
        assert!(ms / 1000 <= s + 2);
    }

    // out-of-range timestamps saturate instead of panicking
    #[test]
    fn t_gen_datetime_extreme_ts() {
        assert!(!gen_datetime(0).is_empty());
        assert!(!gen_datetime(i64::MAX).is_empty());
        assert!(!gen_datetime(i64::MIN).is_empty());
        // in-range boundary values must survive `to_offset` too
        assert!(!gen_datetime(253402300799).is_empty()); // 9999-12-31 UTC
        assert!(!gen_datetime(-377705116800).is_empty()); // 0000-01-01 UTC
    }

    #[test]
    fn t_env_or() {
        // SAFETY: test runs single-threaded, no concurrent env access
        unsafe { std::env::set_var("RUC_TEST_ENV_OR", "hello") };
        assert_eq!(env_or("RUC_TEST_ENV_OR", "default"), "hello");
        unsafe { std::env::remove_var("RUC_TEST_ENV_OR") };
        assert_eq!(env_or("RUC_TEST_ENV_OR_NONEXIST", "default"), "default");
    }

    #[test]
    fn t_retry_success() {
        let mut count = 0;
        let result = retry(3, 0, || -> core::result::Result<i32, String> {
            count += 1;
            if count < 3 {
                Err("not yet".to_owned())
            } else {
                Ok(42)
            }
        });
        assert_eq!(result.unwrap(), 42);
        assert_eq!(count, 3);
    }

    #[test]
    fn t_retry_failure() {
        let result = retry(2, 0, || -> core::result::Result<i32, String> {
            Err("fail".to_owned())
        });
        assert!(result.is_err());
    }

    #[test]
    fn t_ensure() {
        fn check_ok() -> crate::err::Result<()> {
            ensure!(true);
            ensure!(1 > 0, "positive");
            ensure!(2 + 2 == 4, "math: {} + {} = {}", 2, 2, 4);
            Ok(())
        }
        fn check_fail() -> crate::err::Result<()> {
            ensure!(false, "should fail");
            Ok(())
        }
        assert!(check_ok().is_ok());
        assert!(check_fail().is_err());
    }

    #[test]
    fn t_file_utils() {
        let dir = std::env::temp_dir();
        let path = dir.join("ruc_test_file_utils.txt");
        write_file(&path, "hello ruc").unwrap();
        let content = read_file(&path).unwrap();
        assert_eq!(content, "hello ruc");
        std::fs::remove_file(&path).ok();
    }
}
