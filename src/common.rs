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
        $crate::alt!($v1 > $v2, { $v1 }, { $v2 })
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
    ($v1: expr, $v2: expr $(,)*) => {{
        $crate::alt!($v1 > $v2, { $v2 }, { $v1 })
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

/// generate a 'formated DateTime'
#[inline(always)]
pub fn gen_datetime(ts: i64) -> String {
    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]").unwrap();
    time::OffsetDateTime::from_unix_timestamp(ts)
        .unwrap()
        .to_offset(time::UtcOffset::from_hms(8, 0, 0).unwrap())
        .format(&format)
        .unwrap()
}

/// get current DateTime
#[macro_export]
macro_rules! datetime {
    ($ts: expr) => {{
        $crate::common::gen_datetime($ts as i64)
    }};
    () => {{
        $crate::datetime!($crate::ts!())
    }};
}

#[cfg(test)]
mod tests {
    use crate::*;

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
}
