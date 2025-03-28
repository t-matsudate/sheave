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

/// An error that the Play Path sent to a server is empty.
#[derive(Debug)]
pub struct EmptyPlaypath;

impl Display for EmptyPlaypath {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Playpath must not be empty.")
    }
}

impl Error for EmptyPlaypath {}


/// A utility function of constructing an `EmptyPlaypath` error.
pub fn empty_playpath() -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        EmptyPlaypath
    )
}
