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

/// An error that the topic id sent to a server is empty.
#[derive(Debug)]
pub struct EmptyTopicId;

impl Display for EmptyTopicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "The topic id must not be empty.")
    }
}

impl Error for EmptyTopicId {}


/// A utility function of constructing an `EmptyTopicId` error.
pub fn empty_topic_id() -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        EmptyTopicId
    )
}
