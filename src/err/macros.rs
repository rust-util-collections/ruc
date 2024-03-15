//!
//! # Macros
//!
//! Useful macros for chained error managements.
//!

/// print infomation only
#[macro_export]
macro_rules! info {
    ($ops: expr, $fmt: expr, $($arg:tt)*) => {{
        $ops.c($crate::d!($fmt, $($arg)*)).map_err(|e| {
            if "INFO" == $crate::LOG_LEVEL.as_str() {
                e.print(Some("INFO"));
            }
            e
        })
    }};
    ($ops: expr, $msg: expr) => {{
        $crate::info!($ops, "{}", $msg)
    }};
    ($ops: expr) => {{
        $crate::info!($ops, "")
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
    ($ops: expr, $fmt: expr, $($arg:tt)*) => {{
        $crate::omit!($crate::info!($ops, $fmt, $($arg)*));
    }};
    ($ops: expr, $msg: expr) => {{
        $crate::info_omit!($ops, "{}", $msg)
    }};
    ($ops: expr) => {{
        $crate::info_omit!($ops, "")
    }};
}

/// print debug-info, eg: modular and file path, line number ...
#[macro_export]
macro_rules! d {
    ($fmt: expr, $($arg:tt)*) => {{
        let err = format!("{}", format_args!($fmt, $($arg)*));
        $crate::err::SimpleMsg::new(err, file!(), line!(), column!())
    }};
    ($err: expr) => {{
        $crate::d!("{}", $err)
    }};
    () => {{
        $crate::d!("")
    }};
}

/// print custom msg
#[macro_export]
macro_rules! print_msg {
    ($fmt: expr, $($arg:tt)*) => {{
        println!("\n{}", $crate::d!($fmt, $($arg)*));
    }};
    ($msg: expr) => {{
        $crate::print_msg!("{}", $msg)
    }};
}

/// eprint custom msg
#[macro_export]
macro_rules! eprint_msg {
    ($fmt: expr, $($arg:tt)*) => {{
        eprintln!("\n{}", $crate::d!($fmt, $($arg)*));
    }};
    ($msg: expr) => {{
        $crate::eprint_msg!("{}", $msg)
    }};
}

/// Just a panic
#[macro_export]
macro_rules! die {
    ($fmt: expr, $($arg:tt)*) => {{
        $crate::print_msg!($fmt, $($arg)*);
        panic!();
    }};
    ($msg:expr) => {{
        $crate::die!("{}", $msg);
    }};
    () => {
        $crate::die!("");
    };
}

/// Print log, and panic
#[macro_export]
macro_rules! pnk {
    ($ops: expr, $fmt: expr, $($arg:tt)*) => {{
        $ops.c($crate::d!($fmt, $($arg)*)).unwrap_or_else(|e| e.print_die())
    }};
    ($ops: expr, $msg: expr) => {{
        $crate::pnk!($ops, "{}", $msg)
    }};
    ($ops: expr) => {{
        $crate::pnk!($ops, "")
    }};
}

/// Generate error with debug info
#[macro_export]
macro_rules! eg {
    ($fmt: expr, $($arg:tt)*) => {{
        Box::new($crate::err::SimpleError::new($crate::d!($fmt, $($arg)*), None))
            as Box<dyn $crate::err::RucError>
    }};
    ($err: expr) => {{
        $crate::eg!("{}", $err)
    }};
    () => {
        $crate::eg!("")
    };
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn display_style_inner() -> Result<()> {
        #[derive(Debug, Eq, PartialEq)]
        struct CustomErr(i32);

        let l1 = || -> Result<()> { Err(eg!("The final error message!")) };
        let l2 = || -> Result<()> { l1().c(d!()).or_else(|e| l1().c(d!(e))) };
        let l3 = || -> Result<()> { l2().c(d!("A custom message!")) };
        let l4 = || -> Result<()> { l3().c(d!("ERR_UNKNOWN")) };
        let l5 = || -> Result<()> { l4().c(d!("{:?}", CustomErr(-1))) };

        l5().c(d!())
    }

    #[test]
    #[should_panic]
    fn t_display_style() {
        pnk!(display_style_inner());
    }

    #[test]
    fn t_macros() {
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
        print_msg!("{:?}", ts!());
        eprint_msg!("{:?}", ts!());
    }
}
