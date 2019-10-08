//! # The errors
//!
//! The RTMP connection can also occur some error.
//! These are the structs for errors.
use std::{
    error::{
        Error
    },
    fmt::{
        Display,
        Formatter,
        Result as FormatResult
    }
};

/// # The chunk length error
///
/// This will be used when the message length didn't match with the message length in the chunk.
#[derive(Debug)]
pub struct ChunkLengthError {
    description: String,
    source: Option<&'static dyn Error>
}

impl ChunkLengthError {
    /// Constructs a new `ChunkLengthError`.
    ///
    /// # Parameters
    ///
    /// * `description: String`
    ///
    /// The message for this error.
    ///
    /// * `source: Option<&'static dyn Error>`
    ///
    /// More detailed error information.
    /// Pass the `None` if no more information exists.
    /// Otherwise pass its error as the reference.
    pub fn new(description: String, source: Option<&'static dyn Error>) -> Self {
        ChunkLengthError { description, source }
    }
}

impl Display for ChunkLengthError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        writeln!(f, "ChunkLengthError: description {}, source {:?}", self.description, self.source)
    }
}

impl Error for ChunkLengthError {}
unsafe impl Send for ChunkLengthError {}
unsafe impl Sync for ChunkLengthError {}

/// # The chunk format error
///
/// This will be used when some field in the chunk was invalid.
#[derive(Debug)]
pub struct ChunkFormatError {
    description: String,
    source: Option<&'static dyn Error>
}

impl ChunkFormatError {
    /// Constructs a new `ChunkFormatError`.
    ///
    /// # Parameters
    ///
    /// * `description: String`
    ///
    /// The message for this error.
    ///
    /// * `source: Option<&'static dyn Error>`
    ///
    /// More detailed error information.
    /// Pass the `None` if no more information exists.
    /// Otherwise pass its error as the reference.
    pub fn new(description: String, source: Option<&'static dyn Error>) -> Self {
        ChunkFormatError { description, source }
    }
}

impl Display for ChunkFormatError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        writeln!(f, "ChunkFormatError: description {}, source {:?}", self.description, self.source)
    }
}

impl Error for ChunkFormatError {}
unsafe impl Send for ChunkFormatError {}
unsafe impl Sync for ChunkFormatError {}

/// # The digest verification error
///
/// This will be used when the HMAC-SHA256 digest didn't find in the client RTMP handshake chunk.
#[derive(Debug)]
pub struct DigestVerificationError {
    description: String,
    source: Option<&'static dyn Error>
}

impl DigestVerificationError {
    /// Constructs a new `DigestVerificationError`.
    ///
    /// # Parameters
    ///
    /// * `description: String`
    ///
    /// The message for this error.
    ///
    /// * `source: Option<&'static dyn Error>`
    ///
    /// More detailed error information.
    /// Pass the `None` if no more information exists.
    /// Otherwise pass its error as the reference.
    pub fn new(description: String, source: Option<&'static dyn Error>) -> Self {
        DigestVerificationError { description, source }
    }
}

impl Display for DigestVerificationError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        writeln!(f, "DigestVerificationError: description {}, source {:?}", self.description, self.source)
    }
}

impl Error for DigestVerificationError {}
unsafe impl Send for DigestVerificationError {}
unsafe impl Sync for DigestVerificationError {}

/// # The digest offset error
///
/// This will be used when the offset for HMAC-SHA256 digest wasn't invalid.
#[derive(Debug)]
pub struct DigestOffsetError {
    description: String,
    source: Option<&'static dyn Error>
}

impl DigestOffsetError {
    /// Constructs a new `DigestOffsetError`.
    ///
    /// # Parameters
    ///
    /// * `description: String`
    ///
    /// The message for this error.
    ///
    /// * `source: Option<&'static dyn Error>`
    ///
    /// More detailed error information.
    /// Pass the `None` if no more information exists.
    /// Otherwise pass its error as the reference.
    pub fn new(description: String, source: Option<&'static dyn Error>) -> Self {
        DigestOffsetError { description, source }
    }
}

impl Display for DigestOffsetError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        writeln!(f, "DigestOffsetError: description {}, source {:?}", self.description, self.source)
    }
}

impl Error for DigestOffsetError {}
unsafe impl Sync for DigestOffsetError {}
unsafe impl Send for DigestOffsetError {}

/// # The RTMP state error
///
/// This will be used when the RTMP connection hasn't been done exactly. 
#[derive(Debug)]
pub struct RtmpStateError {
    description: String,
    source: Option<&'static dyn Error>
}

impl RtmpStateError {
    /// Constructs a new `RtmpStateError`
    ///
    /// # Parameters
    ///
    /// * `description: String`
    ///
    /// The message for this error.
    ///
    /// * `source: Option<&'static dyn Error>`
    ///
    /// More detailed error information.
    /// Pass the `None` if no more information exists.
    /// Otherwise pass its error as the reference.
    pub fn new(description: String, source: Option<&'static dyn Error>) -> Self {
        RtmpStateError { description, source }
    }
}

impl Display for RtmpStateError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        writeln!(f, "RtmpStateError: description {}, source {:?}", self.description, self.source)
    }
}

impl Error for RtmpStateError {}
unsafe impl Send for RtmpStateError {}
unsafe impl Sync for RtmpStateError {}

/// # The signature does not match error
///
/// This will be used when the HMAC-SHA256 signature didn't match with stored one in server.
#[derive(Debug)]
pub struct SignatureDoesNotMatchError {
    description: String,
    source: Option<&'static dyn Error>
}

impl SignatureDoesNotMatchError {
    /// Constructs a new `SignatureDoesNotMatchError`.
    ///
    /// # Parameters
    ///
    /// * `description: String`
    ///
    /// The message for this error.
    ///
    /// * `source: Option<&'static dyn Error>`
    ///
    /// More detailed error information.
    /// Pass the `None` if no more information exists.
    /// Otherwise pass its error as the reference.
    pub fn new(description: String, source: Option<&'static dyn Error>) -> Self {
        SignatureDoesNotMatchError { description, source }
    }
}

impl Display for SignatureDoesNotMatchError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        writeln!(f, "SignatureDoesNotMatchError: description {}, source: {:?}", self.description, self.source)
    }
}

impl Error for SignatureDoesNotMatchError {}
unsafe impl Send for SignatureDoesNotMatchError {}
unsafe impl Sync for SignatureDoesNotMatchError {}
