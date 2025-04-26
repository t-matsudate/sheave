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

/// An error that some _error command got sent from the server.
#[derive(Debug)]
pub struct ErrorResponse(Object);

impl ErrorResponse {
    /// Constructs this error.
    pub fn new(information: Object) -> Self {
        Self(information)
    }
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "_error response got handled.\ninformation: {:?}", self.0)
    }
}

impl Error for ErrorResponse {}

/// A utility function of constructing an `ErrorResponse` error.
pub fn error_response(information: Object) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        ErrorResponse(information)
    )
}
