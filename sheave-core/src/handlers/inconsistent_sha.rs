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

/// Tells that either digests or signatures are inconsistent in the handshake step.
///
/// The handshake step can fail.
/// Also to be inconsistent either digests or signatures is a part of causes.
/// This reports that above thing occurred to upper APIs.
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

/// A utility function for wrapping the error `InconsistentSha` into `::std::io::Error`.
///
/// # Examples
///
/// ```rust
/// use std::{
///     io::{
///         Error,
///         Result as IOResult
///     },
///     time::Instant
/// };
/// use sheave_core::{
///     handlers::inconsistent_sha,
///     handshake::{
///         EncryptionAlgorithm,
///         Handshake,
///         Version
///     }
/// };
///
///
/// fn main() -> IOResult<()> {
///     let mut handshake = Handshake::new(Instant::now().elapsed(), Version::LATEST_CLIENT);
///     handshake.imprint_digest(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY);
///     if !handshake.did_digest_match(EncryptionAlgorithm::NotEncrypted, Handshake::CLIENT_KEY) {
///         return Err(inconsistent_sha(handshake.get_digest(EncryptionAlgorithm::NotEncrypted).to_vec()))
///     }
///
///     Ok(())
/// }
/// ```
pub fn inconsistent_sha(sha: Vec<u8>) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentSha(sha)
    )
}
