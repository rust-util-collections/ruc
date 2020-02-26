///
///! # All errors will be converted to MyError.
///
use std::{error::Error, fmt};

pub type Result<T> = std::result::Result<T, Box<dyn MyError>>;

pub trait MyError: fmt::Display + Send {
    fn cause(&mut self) -> Option<Box<dyn MyError>> {
        None
    }

    fn display_chain(&mut self) -> String {
        let mut res = format!("\nError: {}", self);
        let mut e = self.cause();
        while let Some(mut c) = e {
            res.push_str(&format!("\nCaused By: {}", c));
            e = c.cause();
        }
        res
    }
}

pub trait MyResult<T> {
    fn c(self, msg: impl fmt::Display) -> Result<T>;
}

impl<T> MyResult<T> for Result<T> {
    fn c(self, msg: impl fmt::Display) -> Result<T> {
        self.map_err(|e| MiniError::new(msg, Some(e)).into())
    }
}

impl<T> MyResult<T> for Option<T> {
    fn c(self, msg: impl fmt::Display) -> Result<T> {
        self.ok_or_else(|| MiniError::new(msg, None).into())
    }
}

impl<T, E: Error> MyResult<T> for std::result::Result<T, E> {
    fn c(self, msg: impl fmt::Display) -> Result<T> {
        self.map_err(|e| {
            MiniError::new(msg, Some(Box::new(MiniError::new(e, None)))).into()
        })
    }
}

struct MiniError {
    msg: String,
    cause: Option<Box<dyn MyError>>,
}

impl MiniError {
    fn new(msg: impl fmt::Display, cause: Option<Box<dyn MyError>>) -> Self {
        MiniError {
            msg: format!("{}", msg),
            cause,
        }
    }
}

impl fmt::Display for MiniError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Into<Box<dyn MyError>> for MiniError {
    fn into(self) -> Box<dyn MyError> {
        Box::new(self)
    }
}

impl MyError for MiniError {
    fn cause(&mut self) -> Option<Box<dyn MyError>> {
        self.cause.take()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let res: Result<i32> = Err(MiniError::new("***", None).into());
        println!(
            "{}",
            res.c("dog").c("cat").c("pig").unwrap_err().display_chain()
        );
    }
}
