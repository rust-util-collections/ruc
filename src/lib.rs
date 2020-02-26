///
///! # All errors will be converted to MyError.
///
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Box<dyn MyError>>;

pub trait MyError: Display + Send {
    fn wrap(self: Box<Self>, msg: &str) -> Box<dyn MyError>;

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
    fn c(self, msg: impl Display) -> Result<T>;
}

impl<T, E: Into<Box<dyn MyError>>> MyResult<T> for std::result::Result<T, E> {
    fn c(self, msg: impl Display) -> Result<T> {
        self.map_err(|e| e.into().wrap(&format!("{}", msg)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct DogError {
        msg: String,
        cause: Option<Box<dyn MyError>>,
    }

    impl DogError {
        fn new(msg: impl Display, cause: Option<Box<dyn MyError>>) -> Self {
            DogError {
                msg: format!("{}", msg),
                cause,
            }
        }
    }

    impl std::fmt::Display for DogError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl MyError for DogError {
        fn wrap(self: Box<Self>, msg: &str) -> Box<dyn MyError> {
            Box::new(DogError::new(msg, Some(self)))
        }

        fn cause(&mut self) -> Option<Box<dyn MyError>> {
            self.cause.take()
        }
    }

    impl<T> MyResult<T> for std::result::Result<T, DogError> {
        fn c(self, msg: impl Display) -> Result<T> {
            self.map_err(|e| {
                Box::new(DogError::new(msg, Some(Box::new(e))))
                    as Box<dyn MyError>
            })
        }
    }

    struct CatError {
        msg: String,
        cause: Option<Box<dyn MyError>>,
    }

    impl CatError {
        fn new(msg: impl Display, cause: Option<Box<dyn MyError>>) -> Self {
            CatError {
                msg: format!("{}", msg),
                cause,
            }
        }
    }

    impl std::fmt::Display for CatError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl MyError for CatError {
        fn wrap(self: Box<Self>, msg: &str) -> Box<dyn MyError> {
            Box::new(CatError::new(msg, Some(self)))
        }

        fn cause(&mut self) -> Option<Box<dyn MyError>> {
            self.cause.take()
        }
    }

    impl<T> MyResult<T> for std::result::Result<T, CatError> {
        fn c(self, msg: impl Display) -> Result<T> {
            self.map_err(|e| {
                Box::new(CatError::new(msg, Some(Box::new(e))))
                    as Box<dyn MyError>
            })
        }
    }

    #[test]
    fn test() {
        let e: Result<()> = Err(DogError::new("dog dog dog", None))
            .c(CatError::new("cat cat cat", None));
        println!("{}", e.unwrap_err().display_chain());
    }
}
