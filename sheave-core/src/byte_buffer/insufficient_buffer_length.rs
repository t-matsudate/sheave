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

/// An error that buffer has been empty during decoding chunks.
#[derive(Debug)]
pub struct InsufficientBufferLength {
    expected: usize,
    actual: usize
}

impl InsufficientBufferLength {
    /// Constructs this error.
    pub fn new(expected: usize, actual: usize) -> Self {
        Self { expected, actual }
    }
}

impl Display for InsufficientBufferLength {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Buffer length is insufficient. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InsufficientBufferLength {}

/// A utility function of constructing an `InsufficientBufferLength` error.
pub fn insufficient_buffer_length(expected: usize, actual: usize) -> IOError {
    IOError::new(
        ErrorKind::InvalidInput,
        InsufficientBufferLength {
            expected,
            actual
        }
    )
}
