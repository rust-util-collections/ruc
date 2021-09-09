//!
//! #  RucError
//!
//! All errors will be converted to RucError.
//!
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::{
    collections::HashSet,
    error::Error,
    fmt::{Debug, Display},
};

lazy_static! {
    // avoid out-of-order printing
    static ref LOG_LK: Mutex<()> = Mutex::new(());
}

/// Custom Result
pub type Result<T> = std::result::Result<T, Box<dyn RucError>>;

/// the major trait defination
pub trait RucError: Display + Debug + Send {
    /// compare two object
    fn eq(&self, another: &dyn RucError) -> bool {
        self.get_lowest_msg() == another.get_lowest_msg()
    }

    /// check if any node from the error_chain matches the given error
    fn eq_any(&self, another: &dyn RucError) -> bool {
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

    /// "error msg" + "debug info"
    fn get_top_error(&self) -> String;

    /// point to a error which caused current error
    fn cause(&self) -> Option<&dyn RucError> {
        None
    }

    /// generate the final error msg
    fn stringify_chain(&self, prefix: Option<&str>) -> String {
        let mut res = format!("\n{}: ", prefix.unwrap_or("ERROR"));
        res.push_str(&self.get_top_error());
        let mut e = self.cause();
        let mut indent_num = 0;
        while let Some(c) = e {
            let mut prefix = "\n".to_owned();
            (0..indent_num).for_each(|_| {
                prefix.push_str("    ");
            });
            res.push_str(&prefix);
            res.push_str("Caused By: ");
            res.push_str(&c.get_top_error().replace("\n", &prefix));
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

    /// Panic after printing `error_chain`
    #[inline(always)]
    fn print_die_debug(&self) -> ! {
        self.print_debug();
        panic!();
    }

    /// Generate the log string
    #[inline(always)]
    fn generate_log(&self, prefix: Option<&str>) -> String {
        self.generate_log_custom(false, prefix)
    }

    /// Generate log in the original `rust debug` format
    #[inline(always)]
    fn generate_log_debug(&self) -> String {
        self.generate_log_custom(true, None)
    }

    /// Generate the log string with custom mode
    fn generate_log_custom(
        &self,
        debug_mode: bool,
        prefix: Option<&str>,
    ) -> String {
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

        #[cfg(target_arch = "wasm32")]
        let pid = 0;

        #[cfg(not(target_arch = "wasm32"))]
        let pid = std::process::id();

        // can not call `p` in the inner,
        // or will cause a infinite loop
        let ns = get_pidns(pid).unwrap();

        let mut res = generate_log_header(ns, pid);

        if debug_mode {
            res.push_str(&format!(" {:#?}", self));
        } else {
            res.push_str(&self.stringify_chain(prefix));
        }

        res
    }

    /// Print log
    #[inline(always)]
    fn print(&self, prefix: Option<&str>) {
        if LOG_LK.lock().is_ok() {
            eprintln!("{}", self.generate_log(prefix));
        }
    }

    /// Print log in `rust debug` format
    #[inline(always)]
    fn print_debug(&self) {
        if LOG_LK.lock().is_ok() {
            eprintln!("{}", self.generate_log_debug());
        }
    }
}

/// Convert all `Result` to this
pub trait RucResult<T, E: Debug + Display + Send> {
    /// alias for 'chain_error'
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
    for std::result::Result<T, ERR>
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    #[inline(always)]
    fn get_top_error(&self) -> String {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n|-- file: {}\n|-- line: {}\n`-- column: {}",
            self.err, self.file, self.line, self.column
        )
    }

    #[cfg(not(feature = "ansi"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "\x1b[01m{}\x1b[00m\n├── \x1b[01mfile:\x1b[00m {}\n├── \x1b[01mline:\x1b[00m {}\n└── \x1b[01mcolumn:\x1b[00m {}",
               self.err, self.file, self.line, self.column)
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
#[allow(clippy::unnecessary_wraps)]
fn get_pidns(_pid: u32) -> Result<String> {
    Ok("NULL".to_owned())
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

        assert!(e1.eq(e2.as_ref()));
    }
}
