//!
//! # Common
//!
//! Common and lightweight utils.
//!

/// HashMap/BTreeMap operations
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

/// BTreeMap operations
#[macro_export]
macro_rules! bmap {
    () => {{
        std::collections::BTreeMap::new()
    }};
    ($($k: expr => $v: expr),+ $(,)*) => {{
        let mut m = bmap! {};
        $(m.insert($k, $v);)*
        m
    }};
}

/// HashSet/BTreeSet operations
#[macro_export]
macro_rules! set {
    () => {{
        std::collections::HashSet::new()
    }};
    (B) => {{
        std::collections::BTreeSet::new()
    }};
    ($(||)+) => {{
        std::collections::HashSet::new
    }};
    (B $(||)+) => {{
        std::collections::BTreeSet::new
    }};
    ($($k: expr),+ $(,)*) => {{
        let mut m = std::collections::HashSet::with_capacity([$(&$k),*].len());
        $(m.insert($k);)*
        m
    }};
    (B $($k: expr),+ $(,)*) => {{
        let mut m = set! {B};
        $(m.insert($k);)*
        m
    }};
}

/// BTreeSet operations
#[macro_export]
macro_rules! bset {
    () => {{
        std::collections::BTreeSet::new()
    }};
    ($($k: expr),+ $(,)*) => {{
        let mut m = bset! {};
        $(m.insert($k);)*
        m
    }};
}

/// optimize readable in high-level-functions
#[macro_export]
macro_rules! alt {
    ($condition: expr, $ops: block, $ops2: block $(,)*) => {{
        if $condition $ops else $ops2
    }};
    ($condition: expr, $ops: block $(,)*) => {{
        if $condition $ops
    }};
    ($condition: expr, $ops: expr, $ops2: expr $(,)*) => {{
        if $condition { $ops } else { $ops2 }
    }};
    ($condition: expr, $ops: expr $(,)*) => {{
        if $condition { $ops }
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

/// Cached local UTC offset, detected once at first access.
/// Falls back to UTC if detection fails (e.g., in multi-threaded context).
static LOCAL_OFFSET: std::sync::LazyLock<time::UtcOffset> =
    std::sync::LazyLock::new(|| {
        time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC)
    });

/// Generate a formatted DateTime string
#[inline(always)]
pub fn gen_datetime(ts: i64) -> String {
    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]").unwrap();
    time::OffsetDateTime::from_unix_timestamp(ts)
        .unwrap()
        .to_offset(*LOCAL_OFFSET)
        .format(&format)
        .unwrap()
}

/// get current DateTime
#[macro_export]
macro_rules! datetime {
    ($ts: expr) => {{ $crate::common::gen_datetime($ts as i64) }};
    () => {{ $crate::datetime!($crate::ts!()) }};
}

#[cfg(test)]
mod tests {
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

        let b = set! {B 1, 2, 3};
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn t_alt() {
        let a = alt!(true, 1, 2,);
        let b = alt!(true, 1, 2);
        let c = alt!(false, 1, 2,);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a * 2, c);
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
        // Test bmap!
        // Single element
        let m1 = bmap! {"a" => 1};
        assert_eq!(m1.len(), 1);
        assert_eq!(m1.get("a"), Some(&1));

        // Multiple elements
        let m2 = bmap! {"a" => 1, "b" => 2};
        assert_eq!(m2.len(), 2);
        assert_eq!(m2.get("b"), Some(&2));

        // Multiple elements with trailing comma
        let m3 = bmap! {
            "a" => 1,
            "b" => 2,
            "c" => 3,
        };
        assert_eq!(m3.len(), 3);
        assert_eq!(m3.get("c"), Some(&3));

        // Test bset!
        // Single element
        let s1 = bset! {1};
        assert_eq!(s1.len(), 1);
        assert!(s1.contains(&1));

        // Multiple elements
        let s2 = bset! {1, 2};
        assert_eq!(s2.len(), 2);
        assert!(s2.contains(&2));

        // Multiple elements with trailing comma
        let s3 = bset! {1, 2, 3,};
        assert_eq!(s3.len(), 3);
        assert!(s3.contains(&3));
    }
}
