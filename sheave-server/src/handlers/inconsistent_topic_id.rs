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

/// An error that some topic path differs the server expects.
#[derive(Debug)]
pub struct InconsistentTopicId {
    expected: AmfString,
    actual: AmfString
}

impl Display for InconsistentTopicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Requested topic ID is inconsistent. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InconsistentTopicId {}

/// A utility function of constructing an `InconsistentTopicId` error.
pub fn inconsistent_topic_id(expected: AmfString, actual: AmfString) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentTopicId { expected, actual }
    )
}
