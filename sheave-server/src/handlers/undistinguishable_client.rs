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
};

/// An error that the server can't distinguish whether some client is a publisher or a subscriber.
#[derive(Debug)]
pub struct UndistinguishableClient;

impl Display for UndistinguishableClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Server couldn't get distinguish its client is either publisher or subscriber.")
    }
}

impl Error for UndistinguishableClient {}

/// A utility function of constructing an `UndistinguishableClient` error.
pub fn undistinguishable_client() -> IOError {
    IOError::new(
        ErrorKind::InvalidInput,
        UndistinguishableClient
    )
}
