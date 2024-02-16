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

/// An error that specified protocol via CLI is invalid.
#[derive(Debug)]
pub struct InvalidProtocol(String);

impl InvalidProtocol {
    /// Constructs this error.
    pub fn new(protocol: String) -> Self {
        Self(protocol)
    }
}

impl Display for InvalidProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Protocol: This isn't one of RTMP: {}", self.0)
    }
}

impl Error for InvalidProtocol {}

/// A utility funciton of constructing an `InvalidProtocol` error.
pub fn invalid_protocol(protocol: String) -> IOError {
    IOError::new(
        ErrorKind::InvalidInput,
        InvalidProtocol::new(protocol)
    )
}
