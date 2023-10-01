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

/// An error means that some command name differs you expect.
#[derive(Debug)]
pub struct InconsistentCommand {
    expected: AmfString,
    actual: AmfString
}

impl InconsistentCommand {
    /// Constructs this error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::InconsistentCommand;
    ///
    /// InconsistentCommand::new("connect".into(), "something bad".into());
    /// ```
    pub fn new(expected: AmfString, actual: AmfString) -> Self {
        Self { expected, actual }
    }
}

impl Display for InconsistentCommand {
    /// Displays this as a formatted string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::InconsistentCommand;
    ///
    /// println!("{}", InconsistentCommand::new("connect".into(), "something bad".into()));
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Command name is inconsistent. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InconsistentCommand {}

/// A utility function of constructing an `InconsistentCommand` error.
///
/// # Examples
///
/// ```rust
/// use sheave_core::messages::inconsistent_command;
///
/// println!("{}", inconsistent_command("connect".into(), "something bad".into()));
/// ```
///
/// [`InconsistentMarker`]: InconsistentMarker
pub fn inconsistent_command(expected: &str, actual: AmfString) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentCommand {
            expected: AmfString::from(expected),
            actual
        }
    )
}
