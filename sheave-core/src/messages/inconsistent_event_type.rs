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
use super::EventType;

/// An error that some event type differs you expect.
#[derive(Debug)]
pub struct InconsistentEventType {
    expected: EventType,
    actual: EventType
}

impl InconsistentEventType {
    /// Constructs this error.
    pub fn new(expected: EventType, actual: EventType) -> Self {
        Self { expected, actual }
    }
}

impl Display for InconsistentEventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Event type is inconsistent. expected: {:?}, actual: {:?}", self.expected, self.actual)
    }
}

impl Error for InconsistentEventType {}

/// A utility function of constructing an `InconsistentEventType` error.
pub fn inconsistent_event_type(expected: EventType, actual: u16) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentEventType {
            expected,
            actual: actual.into()
        }
    )
}
