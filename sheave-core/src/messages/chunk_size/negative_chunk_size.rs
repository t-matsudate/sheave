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

/// An error that some chunk size is negative.
#[derive(Debug)]
pub struct NegativeChunkSize(u32);

impl NegativeChunkSize {
    /// Constructs this error.
    pub fn new(chunk_size: u32) -> Self {
        Self(chunk_size)
    }
}

impl Display for NegativeChunkSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Negative chunk size in signed 32 bits: {}", self.0)
    }
}

impl Error for NegativeChunkSize {}

/// A utility function of constructing a `NegativeChunkSize` error.
pub fn negative_chunk_size(chunk_size: u32) -> IOError {
    IOError::new(
        ErrorKind::InvalidInput,
        NegativeChunkSize(chunk_size)
    )
}
