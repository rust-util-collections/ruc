//!
//! # All errors will be converted to MyError.
//!
use std::{
    error::Error,
    fmt::{Debug, Display},
};

pub type Result<T> = std::result::Result<T, Box<dyn MyError>>;

pub trait MyError: Display + Debug + Send {
    fn cause(&mut self) -> Option<Box<dyn MyError>> {
        None
    }

    fn display_chain(&mut self) -> String {
        let mut res = "\nError: ".to_owned();
        res.push_str(&self.to_string());
        let mut e = self.cause();
        while let Some(mut c) = e {
            res.push_str("\nCaused By: ");
            res.push_str(c.to_string().as_str());
            e = c.cause();
        }
        res
    }
}

pub trait MyResult<T> {
    fn c(self, msg: impl Display) -> Result<T>;
    fn cd(self, msg: impl Debug) -> Result<T>
    where
        Self: Sized,
    {
        self.c(format!("{:?}", msg))
    }
}

impl<T> MyResult<T> for Result<T> {
    fn c(self, msg: impl Display) -> Result<T> {
        self.map_err(|e| SimpleError::new(msg, Some(e)).into())
    }
}

impl<T> MyResult<T> for Option<T> {
    fn c(self, msg: impl Display) -> Result<T> {
        self.ok_or_else(|| SimpleError::new(msg, None).into())
    }
}

impl<T, E: Error> MyResult<T> for std::result::Result<T, E> {
    fn c(self, msg: impl Display) -> Result<T> {
        self.map_err(|e| {
            SimpleError::new(msg, Some(Box::new(SimpleError::new(e, None))))
                .into()
        })
    }
}

#[derive(Debug)]
pub struct SimpleError {
    msg: String,
    cause: Option<Box<dyn MyError>>,
}

impl SimpleError {
    pub fn new(msg: impl Display, cause: Option<Box<dyn MyError>>) -> Self {
        SimpleError {
            msg: msg.to_string(),
            cause,
        }
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
    fn cause(&mut self) -> Option<Box<dyn MyError>> {
        self.cause.take()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let res: Result<i32> = Err(SimpleError::new("***", None).into());
        println!(
            "{}",
            res.c("dog").c("cat").c("pig").unwrap_err().display_chain()
        );
    }
}
