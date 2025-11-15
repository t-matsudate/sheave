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

/// An error that some app path differs the server expects.
#[derive(Debug)]
pub struct InconsistentAppPath {
    expected: AmfString,
    actual: AmfString
}

impl Display for InconsistentAppPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Requested app path is inconsistent. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InconsistentAppPath {}

/// A utility function of constructing an `InconsistentAppPath` error.
pub fn inconsistent_app_path(expected: AmfString, actual: AmfString) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentAppPath { expected, actual }
    )
}

