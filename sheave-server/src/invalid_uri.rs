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
    }
};

/// An error that an URI in the command line option is invalid.
#[derive(Debug)]
pub struct InvalidUri(String);

impl Display for InvalidUri {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "{}", self.0)
    }
}

impl Error for InvalidUri {}

/// A utility function of constructing an `InvalidUri` error.
pub fn invalid_uri(message: String) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InvalidUri(message)
    )
}
