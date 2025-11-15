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

/// An error that stream has no data to write.
///
/// Note this is currently used as to mean sucessful termination.
#[derive(Debug)]
pub struct StreamGotExhausted;

impl Display for StreamGotExhausted {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Stream got exhausted.")
    }
}

impl Error for StreamGotExhausted {}

/// A utility function of constructing an `StreamGotExhausted` error.
pub fn stream_got_exhausted() -> IOError {
    IOError::new(
        ErrorKind::Other,
        StreamGotExhausted
    )
}

