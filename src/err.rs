//!
//! #  MyError
//!
//! All errors will be converted to MyError.
//!
use std::{
    error::Error,
    fmt::{Debug, Display},
};

pub type ErrNo = i32;

pub type Result<T> = std::result::Result<T, Box<dyn MyError>>;

pub trait MyError: Display + Debug + Send {
    fn cause(&mut self) -> Option<Box<dyn MyError>> {
        None
    }

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

pub trait MyResult<T> {
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
            let inner = SimpleMsg::new(&msg.file, msg.line, e.to_string());
            SimpleError::new(
                msg,
                Some(Box::new(SimpleError::new(inner, None))),
            )
            .into()
        })
    }
}

#[derive(Debug)]
pub struct SimpleError {
    msg: SimpleMsg,
    cause: Option<Box<dyn MyError>>,
}

impl SimpleError {
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

#[derive(Debug)]
pub struct SimpleMsg {
    pub eno: ErrNo,
    pub file: String,
    pub line: u32,
    pub info: String,
}

impl SimpleMsg {
    /// 默认错误码 -1 (Unix 风格)
    #[inline(always)]
    pub fn new(file: &str, line: u32, info: String) -> Self {
        Self::newx(-1, file, line, info)
    }

    /// 自定义错误码
    #[inline(always)]
    pub fn newx(eno: ErrNo, file: &str, line: u32, info: String) -> Self {
        SimpleMsg {
            eno,
            file: file.to_owned(),
            line,
            info,
        }
    }
}

impl Display for SimpleMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "\x1b[01m{}\x1b[00m\n├── \x1b[01meno:\x1b[00m {}\n├── \x1b[01mfile:\x1b[00m {}\n└── \x1b[01mline:\x1b[00m {}",
               self.info, self.eno, self.file, self.line)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let res: Result<i32> = Err(SimpleError::new(
            SimpleMsg::new("/tmp/xx.rs", 9, "***".to_owned()),
            None,
        )
        .into());
        println!(
            "{}",
            res.c(SimpleMsg::new("/tmp/xx.rs", 1, "cat".to_owned()))
                .c(SimpleMsg::new("/tmp/xx.rs", 2, "dog".to_owned()))
                .c(SimpleMsg::new("/tmp/xx.rs", 3, "pig".to_owned()))
                .unwrap_err()
                .display_chain()
        );
    }
}
