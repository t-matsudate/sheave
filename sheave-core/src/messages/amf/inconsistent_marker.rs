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

/// An error means that some AMF type marker differes you expect.
#[derive(Debug)]
pub struct InconsistentMarker {
    expected: u8,
    actual: u8
}

impl InconsistentMarker {
    /// Constructs this error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::{
    ///     InconsistentMarker,
    ///     v0::Marker
    /// };
    ///
    /// InconsistentMarker::new(Marker::Number as u8, Marker::Boolean as u8);
    /// ```
    pub fn new(expected: u8, actual: u8) -> Self {
        Self { expected, actual }
    }
}

impl Display for InconsistentMarker {
    /// Displays this as a formatted string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::{
    ///     InconsistentMarker,
    ///     v0::Marker
    /// };
    ///
    /// println!("{}", InconsistentMarker::new(Marker::Number as u8, Marker::Boolean as u8));
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Marker is inconsistent. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InconsistentMarker {}

/// A utility function of constructing an `InconsistentMarker` error.
///
/// # Examples
///
/// ```rust
/// use sheave_core::messages::amf::{
///     inconsistent_marker,
///     v0::Marker
/// };
///
/// println!("{}", inconsistent_marker(Marker::Number as u8, Marker::Boolean as u8));
/// ```
///
/// [`InconsistentMarker`]: InconsistentMarker
pub fn inconsistent_marker(expected: u8, actual: u8) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InconsistentMarker {
            expected,
            actual
        }
    )
}
