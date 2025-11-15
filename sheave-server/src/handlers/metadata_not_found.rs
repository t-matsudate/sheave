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
    },
};
use sheave_core::messages::amf::v0::AmfString;

/// An error that published topic doesn't have any metadata in its file.
#[derive(Debug)]
pub struct MetadataNotFound(AmfString);

impl Display for MetadataNotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Metadata didn't find in specified topic path: {}", &self.0)
    }
}

impl Error for MetadataNotFound {}

/// A utility function of constructing a `MetadataNotFound` error.
pub fn metadata_not_found(topic_path: AmfString) -> IOError {
    IOError::new(
        ErrorKind::UnexpectedEof,
        MetadataNotFound(topic_path)
    )
}
