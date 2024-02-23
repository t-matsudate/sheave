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
use crate::messages::amf::v0::Object;

/// An error that publishing has failed by something cause.
#[derive(Debug)]
pub struct PublishingFailure(Object);

impl PublishingFailure {
    /// Constructs this error.
    pub fn new(info_object: Object) -> Self {
        Self(info_object)
    }
}

impl Display for PublishingFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(
            f,
            "Publishing failed. Code: {:?}, Description: {:?}",
            self.0.get_properties().get("code"),
            self.0.get_properties().get("description")
        )
    }
}

impl Error for PublishingFailure {}

/// A utility function of constructing a `PublishingFailure` error.
pub fn publishing_failure(info_object: Object) -> IOError {
    IOError::new(
        ErrorKind::Other,
        PublishingFailure(info_object)
    )
}
