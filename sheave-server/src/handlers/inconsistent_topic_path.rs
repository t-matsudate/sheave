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
pub struct InconsistentTopicPath {
    expected: AmfString,
    actual: AmfString
}

impl Display for InconsistentTopicPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Requested topic path is inconsistent. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InconsistentTopicPath {}

/// A utility function of constructing an `InconsistentTopicPath` error.
pub fn inconsistent_topic_path(expected: AmfString, actual: AmfString) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentTopicPath { expected, actual }
    )
}
