//!
//! #  MyError
//!
//! All errors will be converted to MyError.
//!
use std::{
    error::Error,
    fmt::{Debug, Display},
};

/// error number
pub type ErrNo = i32;

/// Custom Result
pub type Result<T> = std::result::Result<T, Box<dyn MyError>>;

/// the major trait defination
pub trait MyError: Display + Debug + Send {
    /// point to a error which caused current error
    fn cause(&mut self) -> Option<Box<dyn MyError>> {
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
pub trait MyResult<T> {
    /// alias for 'chain_error'
    fn c(self, msg: SimpleMsg) -> Result<T>;
}

impl<T> MyResult<T> for Result<T> {
    #[inline(always)]
    fn c(self, msg: SimpleMsg) -> Result<T> {
        self.map_err(|e| SimpleError::new(msg, Some(e)).into())
    }
}

impl<T> MyResult<T> for Option<T> {
    #[inline(always)]
    fn c(self, msg: SimpleMsg) -> Result<T> {
        self.ok_or_else(|| SimpleError::new(msg, None).into())
    }
}

impl<T, E: Error> MyResult<T> for std::result::Result<T, E> {
    #[inline(always)]
    fn c(self, msg: SimpleMsg) -> Result<T> {
        self.map_err(|e| {
            let inner =
                SimpleMsg::new(&msg.file, msg.line, msg.column, e.to_string());
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
pub struct SimpleError {
    msg: SimpleMsg,
    cause: Option<Box<dyn MyError>>,
}

impl SimpleError {
    /// new it
    #[inline(always)]
    pub fn new(msg: SimpleMsg, cause: Option<Box<dyn MyError>>) -> Self {
        SimpleError { msg, cause }
    }
}

impl Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Into<Box<dyn MyError>> for SimpleError {
    fn into(self) -> Box<dyn MyError> {
        Box::new(self)
    }
}

impl MyError for SimpleError {
    #[inline(always)]
    fn cause(&mut self) -> Option<Box<dyn MyError>> {
        self.cause.take()
    }
}

/// msg + errno + <file + line + column>
#[derive(Debug)]
pub struct SimpleMsg {
    /// errno
    pub eno: ErrNo,
    /// file path
    pub file: String,
    /// line number
    pub line: u32,
    /// column number
    pub column: u32,
    /// error msg
    pub info: String,
}

impl SimpleMsg {
    /// 默认错误码 -1 (Unix 风格)
    #[inline(always)]
    pub fn new(file: &str, line: u32, column: u32, info: String) -> Self {
        Self::newx(-1, file, line, column, info)
    }

    /// 自定义错误码
    #[inline(always)]
    pub fn newx(
        eno: ErrNo,
        file: &str,
        line: u32,
        column: u32,
        info: String,
    ) -> Self {
        SimpleMsg {
            eno,
            file: file.to_owned(),
            line,
            column,
            info,
        }
    }
}

impl Display for SimpleMsg {
    #[cfg(feature = "ansi")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "\x1b[01m{}\x1b[00m\n|-- \x1b[01meno:\x1b[00m {}\n|-- \x1b[01mfile:\x1b[00m {}\n|-- \x1b[01mline:\x1b[00m {}\n`-- \x1b[01mcolumn:\x1b[00m {}",
               self.info, self.eno, self.file, self.line, self.column)
    }

    #[cfg(not(feature = "ansi"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "\x1b[01m{}\x1b[00m\n├── \x1b[01meno:\x1b[00m {}\n├── \x1b[01mfile:\x1b[00m {}\n├── \x1b[01mline:\x1b[00m {}\n└── \x1b[01mcolumn:\x1b[00m {}",
               self.info, self.eno, self.file, self.line, self.column)
    }
}

impl From<SimpleMsg> for Box<dyn MyError> {
    fn from(m: SimpleMsg) -> Self {
        SimpleError::new(m, None).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let res: Result<i32> = Err(SimpleError::new(
            SimpleMsg::new("/tmp/xx.rs", 9, 90, "***".to_owned()),
            None,
        )
        .into());
        println!(
            "{}",
            res.c(SimpleMsg::new("/tmp/xx.rs", 1, 10, "cat".to_owned()))
                .c(SimpleMsg::new("/tmp/xx.rs", 2, 20, "dog".to_owned()))
                .c(SimpleMsg::new("/tmp/xx.rs", 3, 30, "pig".to_owned()))
                .unwrap_err()
                .display_chain()
        );
    }
}
