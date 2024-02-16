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

/// An error that some AMF type marker differes you expect.
#[derive(Debug)]
pub struct InconsistentMarker {
    expected: u8,
    actual: u8
}

impl InconsistentMarker {
    /// Constructs this error.
    pub fn new(expected: u8, actual: u8) -> Self {
        Self { expected, actual }
    }
}

impl Display for InconsistentMarker {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Marker is inconsistent. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InconsistentMarker {}

/// A utility function of constructing an `InconsistentMarker` error.
pub fn inconsistent_marker(expected: u8, actual: u8) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentMarker {
            expected,
            actual
        }
    )
}
