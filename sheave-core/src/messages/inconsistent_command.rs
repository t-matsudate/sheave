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

/// An error that some command name differs you expect.
#[derive(Debug)]
pub struct InconsistentCommand {
    expected: AmfString,
    actual: AmfString
}

impl InconsistentCommand {
    /// Constructs this error.
    pub fn new(expected: AmfString, actual: AmfString) -> Self {
        Self { expected, actual }
    }
}

impl Display for InconsistentCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Command name is inconsistent. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InconsistentCommand {}

/// A utility function of constructing an `InconsistentCommand` error.
pub fn inconsistent_command(expected: &str, actual: AmfString) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentCommand {
            expected: AmfString::from(expected),
            actual
        }
    )
}
