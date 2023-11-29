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

/// Tells that specified protocol via CLI is invalid.
#[derive(Debug)]
pub struct InvalidProtocol(String);

impl InvalidProtocol {
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

/// A utility funciton for wrapping the error `InvalidProtocol` into `::std::io::Error`.
pub fn invalid_protocol(protocol: String) -> IOError {
    IOError::new(
        ErrorKind::InvalidInput,
        InvalidProtocol::new(protocol)
    )
}
