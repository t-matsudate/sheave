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
use sheave_core::messages::amf::v0::Object;

/// An error that handling publish command has been failed.
#[derive(Debug)]
pub struct PublicationError(Object);

impl PublicationError {
    /// Constructs this error.
    pub fn new(information: Object) -> Self {
        Self(information)
    }
}

impl Display for PublicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "The \"publish\" step got failed by following cause: {:?}", self.0)
    }
}

impl Error for PublicationError {}

/// A utility function of constructing an `PublicationError` error.
pub fn publication_error(information: Object) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        PublicationError(information)
    )
}
