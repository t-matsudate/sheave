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
use sheave_core::messages::amf::v0::AmfString;

/// An error that topic is requested when its stream is unpublished yet.
#[derive(Debug)]
pub struct StreamIsUnpublished(AmfString);

impl Display for StreamIsUnpublished {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "A stream of {} is unpublished.", self.0)
    }
}

impl Error for StreamIsUnpublished {}

/// A utility function of constructing a `StreamIsUnpublished` error.
pub fn stream_is_unpublished(topic_path: AmfString) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        StreamIsUnpublished(topic_path)
    )
}
