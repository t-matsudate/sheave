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

#[derive(Debug)]
pub(crate) struct ChunkLengthError {
    description: String,
    source: Option<&'static Error>
}

impl ChunkLengthError {
    pub(crate) fn new(description: String, source: Option<&'static Error>) -> Self {
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

#[derive(Debug)]
pub(crate) struct DigestVerificationError {
    description: String,
    source: Option<&'static Error>
}

impl DigestVerificationError {
    pub(crate) fn new(description: String, source: Option<&'static Error>) -> Self {
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

#[derive(Debug)]
pub(crate) struct DigestOffsetError {
    description: String,
    source: Option<&'static Error>
}

impl DigestOffsetError {
    pub(crate) fn new(description: String, source: Option<&'static Error>) -> Self {
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

#[derive(Debug)]
pub(crate) struct RtmpStateError {
    description: String,
    source: Option<&'static Error>
}

impl RtmpStateError {
    pub(crate) fn new(description: String, source: Option<&'static Error>) -> Self {
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

#[derive(Debug)]
pub(crate) struct SignatureDoesNotMatchError {
    description: String,
    source: Option<&'static Error>
}

impl SignatureDoesNotMatchError {
    pub(crate) fn new(description: String, source: Option<&'static Error>) -> Self {
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
