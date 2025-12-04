//!
//! # Error management
//!
//! All errors will be converted to RucError.
//!
//! ## Example
//!
//! ```rust
//! use ruc::*;
//!
//! #[derive(Debug, Eq, PartialEq)]
//! struct CustomErr(i32);
//!
//! fn will_panic() {
//!     let l1 = || -> Result<()> { Err(eg!("The final error message!")) };
//!     let l2 = || -> Result<()> { l1().c(d!()) };
//!     let l3 = || -> Result<()> { l2().c(d!("A custom message!")) };
//!     let l4 = || -> Result<()> { l3().c(d!("ERR_UNKNOWN")) };
//!     let l5 = || -> Result<()> { l4().c(d!("{:?}", CustomErr(-1))) };
//!
//!     pnk!(l5());
//! }
//! ```
//!

mod macros;

use core::{
    any::{Any, TypeId},
    fmt::{Debug, Display},
};

use std::{collections::HashSet, error::Error, sync::LazyLock, sync::Mutex};

// avoid out-of-order printing
static LOG_LK: Mutex<()> = Mutex::new(());

/// `INFO` or `ERROR`, if mismatch, default to `INFO`
pub static LOG_LEVEL: LazyLock<String> = LazyLock::new(|| {
    if let Ok(l) = std::env::var("RUC_LOG_LEVEL")
        && "ERROR" == l
    {
        return "ERROR".to_owned();
    }
    "INFO".to_owned()
});

/// Custom Result
pub type Result<T> = core::result::Result<T, Box<dyn RucError>>;

/// the major trait defination
pub trait RucError: Display + Debug + Send {
    /// type id of current error type
    fn type_id(&self) -> TypeId;

    /// type ids of errors of each level(from top to bottom).
    fn type_ids(&self) -> Vec<TypeId> {
        let mut res = Vec::new();
        res.push(self.type_id());
        while let Some(c) = self.cause() {
            res.push(c.type_id());
        }
        res
    }

    /// get the type of the lowest(bottom) error
    fn lowest_type_id(&self) -> TypeId {
        *self.type_ids().last().unwrap()
    }

    /// check the type of the lowest error
    fn lowest_is_type(&self, e: &dyn Any) -> bool {
        self.lowest_type_id() == e.type_id()
    }

    /// check if an error exists in the error chain
    fn contains_type(&self, e: &dyn Any) -> bool {
        self.type_ids().contains(&e.type_id())
    }

    /// compare two object
    fn msg_eq(&self, another: &dyn RucError) -> bool {
        self.get_lowest_msg() == another.get_lowest_msg()
    }

    /// check if any node from the error_chain matches the given error
    fn msg_has_overloop(&self, another: &dyn RucError) -> bool {
        let mut b;

        let mut self_list = HashSet::new();
        self_list.insert(self.get_top_msg());
        b = self.cause();
        while let Some(next) = b {
            self_list.insert(next.get_top_msg());
            b = next.cause();
        }

        let mut target_list = HashSet::new();
        target_list.insert(another.get_top_msg());
        b = another.cause();
        while let Some(next) = b {
            target_list.insert(next.get_top_msg());
            b = next.cause();
        }

        !self_list.is_disjoint(&target_list)
    }

    /// convert the error of current level to string
    fn get_top_msg(&self) -> String;

    /// convert the error of lowest level to string
    fn get_lowest_msg(&self) -> String;

    /// Get the original error object,
    /// used to match its original type by `Any`.
    fn get_lowest_err(&self) -> &dyn RucError;

    /// "error msg" + "debug info"
    fn get_top_msg_with_dbginfo(&self) -> String;

    /// point to a error which caused current error
    fn cause(&self) -> Option<&dyn RucError> {
        None
    }

    /// generate the final error msg
    fn stringify_chain(&self, prefix: Option<&str>) -> String {
        let mut res =
            format!("{}{}: ", delimiter(), prefix.unwrap_or("ERROR"));
        res.push_str(&self.get_top_msg_with_dbginfo());
        let mut e = self.cause();
        let mut indent_num = 0;
        while let Some(c) = e {
            let mut prefix = delimiter().to_owned();
            (0..indent_num).for_each(|_| {
                prefix.push_str(indent());
            });
            res.push_str(&prefix);
            res.push_str("Caused By: ");
            res.push_str(&c.get_top_msg_with_dbginfo().replace('\n', &prefix));
            indent_num += 1;
            e = c.cause();
        }
        res
    }

    /// Panic after printing `error_chain`
    #[inline(always)]
    fn print_die(&self) -> ! {
        self.print(None);
        panic!();
    }

    /// Generate the log string
    #[inline(always)]
    fn generate_log(&self, prefix: Option<&str>) -> String {
        self.generate_log_custom(prefix)
    }

    /// Generate the log string with custom mode
    fn generate_log_custom(&self, prefix: Option<&str>) -> String {
        #[cfg(not(feature = "ansi"))]
        #[inline(always)]
        fn generate_log_header(ns: String, pid: u32) -> String {
            format!(
                "\n\x1b[31;01m# {time} [pid: {pid}] [pidns: {ns}]\x1b[00m",
                time = crate::datetime!(),
                pid = pid,
                ns = ns,
            )
        }

        #[cfg(feature = "ansi")]
        #[inline(always)]
        fn generate_log_header(ns: String, pid: u32) -> String {
            format!(
                "\n# {time} [pid: {pid}] [pidns: {ns}]",
                time = crate::datetime!(),
                pid = pid,
                ns = ns,
            )
        }

        let pid = std::process::id();

        // can not call `p` in the inner,
        // or will cause a infinite loop
        let ns = get_pidns(pid).unwrap_or_else(|_| "UNKNOWN".to_owned());

        let mut res = generate_log_header(ns, pid);
        res.push_str(&self.stringify_chain(prefix));
        res
    }

    /// Print log
    #[inline(always)]
    fn print(&self, prefix: Option<&str>) {
        if LOG_LK.lock().is_ok() {
            eprintln!("{}", self.generate_log(prefix));
        }
    }
}

/// Convert all `Result` to this
pub trait RucResult<T, E: Debug + Display + Send> {
    /// shorter alias for 'chain_error'
    fn c(self, msg: SimpleMsg<E>) -> Result<T>;
}

impl<T, E: Debug + Display + Send> RucResult<T, E> for Result<T> {
    #[inline(always)]
    fn c(self, msg: SimpleMsg<E>) -> Result<T> {
        self.map_err(|e| SimpleError::new(msg, Some(e)).into())
    }
}

impl<T, E: Debug + Display + Send> RucResult<T, E> for Option<T> {
    #[inline(always)]
    fn c(self, msg: SimpleMsg<E>) -> Result<T> {
        self.ok_or_else(|| SimpleError::new(msg, None).into())
    }
}

impl<T, E: Debug + Display + Send, ERR: Error> RucResult<T, E>
    for core::result::Result<T, ERR>
{
    #[inline(always)]
    fn c(self, msg: SimpleMsg<E>) -> Result<T> {
        self.map_err(|e| {
            let inner =
                SimpleMsg::new(e.to_string(), &msg.file, msg.line, msg.column);
            SimpleError::new(
                msg,
                Some(Box::new(SimpleError::new(inner, None))),
            )
            .into()
        })
    }
}

/// A pre-impled Error
#[derive(Debug)]
pub struct SimpleError<E: Debug + Display + Send + 'static> {
    msg: SimpleMsg<E>,
    cause: Option<Box<dyn RucError>>,
}

impl<E: Debug + Display + Send + 'static> SimpleError<E> {
    #[allow(missing_docs)]
    #[inline(always)]
    pub fn new(msg: SimpleMsg<E>, cause: Option<Box<dyn RucError>>) -> Self {
        SimpleError { msg, cause }
    }
}

impl<E: Debug + Display + Send + 'static> Display for SimpleError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.generate_log(None))
    }
}

impl<E: Debug + Display + Send + 'static> From<SimpleError<E>>
    for Box<dyn RucError>
{
    fn from(e: SimpleError<E>) -> Box<dyn RucError> {
        Box::new(e)
    }
}

impl<E: Debug + Display + Send + 'static> RucError for SimpleError<E> {
    fn type_id(&self) -> TypeId {
        TypeId::of::<E>()
    }

    /// get the top-level error message
    #[inline(always)]
    fn get_top_msg(&self) -> String {
        self.msg.err.to_string()
    }

    /// get the final(lowest) error message
    #[inline(always)]
    fn get_lowest_msg(&self) -> String {
        if let Some(next) = self.cause.as_ref() {
            next.get_lowest_msg()
        } else {
            self.msg.err.to_string()
        }
    }

    fn get_lowest_err(&self) -> &dyn RucError {
        if let Some(next) = self.cause.as_ref() {
            next.get_lowest_err()
        } else {
            self
        }
    }

    #[inline(always)]
    fn get_top_msg_with_dbginfo(&self) -> String {
        self.msg.to_string()
    }

    #[inline(always)]
    fn cause(&self) -> Option<&dyn RucError> {
        self.cause.as_deref()
    }
}

/// error + <file + line + column>
#[derive(Debug)]
pub struct SimpleMsg<E: Debug + Display + Send + 'static> {
    /// actual error
    pub err: E,
    /// file path
    pub file: String,
    /// line number
    pub line: u32,
    /// column number
    pub column: u32,
}

impl<E: Debug + Display + Send + 'static> SimpleMsg<E> {
    /// create new error
    #[inline(always)]
    pub fn new(err: E, file: &str, line: u32, column: u32) -> Self {
        SimpleMsg {
            err,
            file: file.to_owned(),
            line,
            column,
        }
    }
}

impl<E: Debug + Display + Send + 'static> Display for SimpleMsg<E> {
    #[cfg(feature = "ansi")]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{0}{4}{5}file: {1}{4}{5}line: {2}{4}{6}column: {3}",
            self.err,
            self.file,
            self.line,
            self.column,
            delimiter(),
            pretty()[0],
            pretty()[1]
        )
    }

    #[cfg(not(feature = "ansi"))]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "\x1b[01m{0}\x1b[00m{4}{5}\x1b[01mfile:\x1b[00m {1}{4}{5}\x1b[01mline:\x1b[00m {2}{4}{6}\x1b[01mcolumn:\x1b[00m {3}",
            self.err,
            self.file,
            self.line,
            self.column,
            delimiter(),
            pretty()[0],
            pretty()[1]
        )
    }
}

impl<E: Debug + Display + Send + 'static> From<SimpleMsg<E>>
    for Box<dyn RucError>
{
    fn from(m: SimpleMsg<E>) -> Self {
        SimpleError::new(m, None).into()
    }
}

#[inline(always)]
#[cfg(target_os = "linux")]
fn get_pidns(pid: u32) -> Result<String> {
    std::fs::read_link(format!("/proc/{pid}/ns/pid"))
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
#[allow(clippy::unnecessary_wraps)]
fn get_pidns(_pid: u32) -> Result<String> {
    Ok("NULL".to_owned())
}

#[cfg(not(feature = "compact"))]
const fn delimiter() -> &'static str {
    "\n"
}

#[cfg(feature = "compact")]
const fn delimiter() -> &'static str {
    " 》"
}

#[cfg(not(feature = "compact"))]
const fn indent() -> &'static str {
    "    "
}

#[cfg(feature = "compact")]
const fn indent() -> &'static str {
    ""
}

#[cfg(all(not(feature = "compact"), feature = "ansi"))]
const fn pretty() -> [&'static str; 2] {
    ["|--", "`--"]
}

#[cfg(all(not(feature = "compact"), not(feature = "ansi")))]
const fn pretty() -> [&'static str; 2] {
    ["├──", "└──"]
}

#[cfg(feature = "compact")]
const fn pretty() -> [&'static str; 2] {
    ["", ""]
}

#[cfg(test)]
mod test {
    use super::*;
    use std::process;

    #[test]
    fn t_get_pidns() {
        let ns_name = crate::pnk!(get_pidns(process::id()));
        assert!(1 < ns_name.len());
    }

    #[test]
    fn t_error_chain() {
        let res: Result<i32> = Err(SimpleError::new(
            SimpleMsg::new("***", "/tmp/xx.rs", 9, 90),
            None,
        )
        .into());
        println!(
            "{}",
            res.c(SimpleMsg::new("cat", "/tmp/xx.rs", 1, 10))
                .c(SimpleMsg::new("dog", "/tmp/xx.rs", 2, 20))
                .c(SimpleMsg::new("pig", "/tmp/xx.rs", 3, 30))
                .unwrap_err()
                .stringify_chain(None)
        );

        let e1: Box<dyn RucError> =
            SimpleError::new(SimpleMsg::new("***", "/tmp/xx.rs", 9, 90), None)
                .into();
        let e2: Box<dyn RucError> =
            SimpleError::new(SimpleMsg::new("***", "/tmp/xx.rs", 9, 90), None)
                .into();

        assert!(e1.msg_eq(e2.as_ref()));
        assert!(e1.lowest_is_type(&""));
        assert!(e2.lowest_is_type(&""));
        assert_eq!(e2.lowest_type_id(), TypeId::of::<&str>());
    }
}
