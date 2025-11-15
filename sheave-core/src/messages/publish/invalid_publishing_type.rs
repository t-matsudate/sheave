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
use crate::messages::amf::v0::AmfString;

/// An error that some publishing type differs you expect.
#[derive(Debug)]
pub struct InvalidPublishingType(AmfString);

impl InvalidPublishingType {
    /// Constructs this error.
    pub fn new(actual: AmfString) -> Self {
        Self(actual)
    }
}

impl Display for InvalidPublishingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Publishing type is neither \"live\", \"record\", nor \"append\". actual: {}", self.0)
    }
}

impl Error for InvalidPublishingType {}

/// A utility function of constructing an `InvalidPublishingType` error.
pub fn invalid_publishing_type(actual: AmfString) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InvalidPublishingType(actual)
    )
}
