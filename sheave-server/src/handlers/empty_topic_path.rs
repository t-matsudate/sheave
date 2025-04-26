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

/// An error that the topic path sent to a server is empty.
#[derive(Debug)]
pub struct EmptyTopicPath;

impl Display for EmptyTopicPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "The topic path must not be empty.")
    }
}

impl Error for EmptyTopicPath {}


/// A utility function of constructing an `EmptyTopicPath` error.
pub fn empty_topic_path() -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        EmptyTopicPath
    )
}
