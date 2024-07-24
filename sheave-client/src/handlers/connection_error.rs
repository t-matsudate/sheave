use std::{
    error::Error,
    fmt::{
        Display,
        Formatter,
        Result as FormatResult
    },
    io::{
        Error as IOError,
        ErrorKind,
    }
};
use sheave_core::messages::amf::v0::Object;

/// An error that handling connect command has been failed.
#[derive(Debug)]
pub struct ConnectionError(Object);

impl ConnectionError {
    /// Constructs this error.
    pub fn new(information: Object) -> Self {
        Self(information)
    }
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "The \"connect\" step got failed by following cause:\n{:?}", self.0)
    }
}

impl Error for ConnectionError {}

/// A utility function of constructing an `ConnectionError` error.
pub fn connection_error(information: Object) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        ConnectionError(information)
    )
}
