use std;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum SlicedError {
    Generic(GenericError),
    FromUtf8(std::string::FromUtf8Error),
    ParseInt(std::num::ParseIntError),
}

impl SlicedError {
    pub fn generic(message: &str) -> SlicedError {
        SlicedError::Generic(GenericError::new(message))
    }
}

impl From<std::string::FromUtf8Error> for SlicedError {
    fn from(err: std::string::FromUtf8Error) -> SlicedError {
        SlicedError::FromUtf8(err)
    }
}

impl From<std::num::ParseIntError> for SlicedError {
    fn from(err: std::num::ParseIntError) -> SlicedError {
        SlicedError::ParseInt(err)
    }
}

impl fmt::Display for SlicedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            SlicedError::Generic(ref err) => write!(f, "{}", err),
            SlicedError::FromUtf8(ref err) => write!(f, "{}", err),
            SlicedError::ParseInt(ref err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for SlicedError {
    fn description(&self) -> &str {
        // Both underlying errors already impl `Error`, so we defer to their
        // implementations.
        match *self {
            SlicedError::Generic(ref err) => err.description(),
            SlicedError::FromUtf8(ref err) => err.description(),
            SlicedError::ParseInt(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            // N.B. Both of these implicitly cast `err` from their concrete
            // types (either `&io::Error` or `&num::ParseIntError`)
            // to a trait object `&Error`. This works because both error types
            // implement `Error`.
            SlicedError::Generic(ref err) => Some(err),
            SlicedError::FromUtf8(ref err) => Some(err),
            SlicedError::ParseInt(ref err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub struct GenericError {
    message: String,
}

impl GenericError {
    pub fn new(message: &str) -> GenericError {
        GenericError {
            message: String::from(message),
        }
    }
}

impl<'a> fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Store error: {}", self.message)
    }
}

impl<'a> error::Error for GenericError {
    fn description(&self) -> &str {
        self.message.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
