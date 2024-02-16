use std::{
    error::Error,
    fmt::{
        Display,
        Formatter,
        Result as FormatResult
    },
    io::{
        Error as IOError,
        ErrorKind
    },
    string::FromUtf8Error
};

/// An error that some string data is invalid for UTF-8.
#[derive(Debug)]
pub struct InvalidString(FromUtf8Error);

impl InvalidString {
    /// Constructs this error.
    pub fn new(e: FromUtf8Error) -> Self {
        Self(e)
    }
}

impl Display for InvalidString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        Display::fmt(&self.0, f)
    }
}

impl Error for InvalidString {}

/// A utility function of constructing an `InvalidString` error.
pub fn invalid_string(e: FromUtf8Error) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InvalidString(e)
    )
}
