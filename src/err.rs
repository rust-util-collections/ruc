//!
//! #  RucError
//!
//! All errors will be converted to RucError.
//!
use std::{
    error::Error,
    fmt::{Debug, Display},
};

/// Custom Result
pub type Result<T> = std::result::Result<T, Box<dyn RucError>>;

/// the major trait defination
pub trait RucError: Display + Debug + Send {
    /// compare two object
    fn eq(&self, another: &dyn RucError) -> bool {
        self.get_error() == another.get_error()
    }

    /// convert the inner error to string
    fn get_error(&self) -> String;

    /// point to a error which caused current error
    fn cause(&mut self) -> Option<Box<dyn RucError>> {
        None
    }

    /// generate the final error msg
    fn display_chain(&mut self) -> String {
        let mut res = "\nError: ".to_owned();
        res.push_str(&self.to_string());
        let mut e = self.cause();
        let mut indent_num = 0;
        while let Some(mut c) = e {
            let mut prefix = "\n".to_owned();
            (0..indent_num).for_each(|_| {
                prefix.push_str("    ");
            });
            res.push_str(&prefix);
            res.push_str("Caused By: ");
            res.push_str(&c.to_string().replace("\n", &prefix));
            indent_num += 1;
            e = c.cause();
        }
        res
    }
}

/// convert all to this
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
    /// new it
    #[inline(always)]
    pub fn new(msg: SimpleMsg<E>, cause: Option<Box<dyn RucError>>) -> Self {
        SimpleError { msg, cause }
    }
}

impl<E: Debug + Display + Send + 'static> Display for SimpleError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl<E: Debug + Display + Send + 'static> Into<Box<dyn RucError>>
    for SimpleError<E>
{
    fn into(self) -> Box<dyn RucError> {
        Box::new(self)
    }
}

impl<E: Debug + Display + Send + 'static> RucError for SimpleError<E> {
    /// get the final(lowest) error
    #[inline(always)]
    fn get_error(&self) -> String {
        if let Some(next) = self.cause.as_ref() {
            next.get_error()
        } else {
            self.msg.err.to_string()
        }
    }

    #[inline(always)]
    fn cause(&mut self) -> Option<Box<dyn RucError>> {
        self.cause.take()
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
        write!(f,
               "\x1b[01m{}\x1b[00m\n|-- \x1b[01mfile:\x1b[00m {}\n|-- \x1b[01mline:\x1b[00m {}\n`-- \x1b[01mcolumn:\x1b[00m {}",
               self.err, self.file, self.line, self.column)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
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
                .display_chain()
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
