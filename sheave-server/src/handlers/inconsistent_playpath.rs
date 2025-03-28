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

/// An error that some playpath differs the server expects.
#[derive(Debug)]
pub struct InconsistentPlaypath {
    expected: AmfString,
    actual: AmfString
}

impl Display for InconsistentPlaypath {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Requested name is inconsistent. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InconsistentPlaypath {}

/// A utility function of constructing an `InconsistentPlaypath` error.
pub fn inconsistent_playpath(expected: AmfString, actual: AmfString) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentPlaypath { expected, actual }
    )
}
