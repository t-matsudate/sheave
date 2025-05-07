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

/// Tells that either digests or signatures is inconsistent in the handshake step.
#[derive(Debug)]
pub struct InconsistentSha(Vec<u8>);

impl<'a> InconsistentSha {
    /// Constructs this error.
    pub fn new(sha: Vec<u8>) -> Self {
        Self(sha)
    }
}

impl Display for InconsistentSha {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Invalid SHA digest/signature: {:?}", self.0)
    }
}

impl Error for InconsistentSha {}

/// A utility function for constructing an `InconsistentSha` error.
pub fn inconsistent_sha(sha: Vec<u8>) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentSha(sha)
    )
}
