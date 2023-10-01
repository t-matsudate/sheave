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

/// An error that means buffer has been empty during encoding chunks.
#[derive(Debug)]
pub struct InsufficientBufferLength {
    expected: usize,
    actual: usize
}

impl InsufficientBufferLength {
    /// Constructs this error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::InsufficientBufferLength;
    ///
    /// InsufficientBufferLength::new(1, 0);
    /// ```
    pub fn new(expected: usize, actual: usize) -> Self {
        Self { expected, actual }
    }
}

impl Display for InsufficientBufferLength {
    /// Displays this as a formatted string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::InsufficientBufferLength;
    ///
    /// println!("{}", InsufficientBufferLength::new(1, 0));
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Buffer length is insufficient. expected: {}, actual: {}", self.expected, self.actual)
    }
}

impl Error for InsufficientBufferLength {}

/// A utility function of constructing an `InsufficientBufferLength` error.
///
/// # Examples
///
/// ```rust
/// use sheave_core::insufficient_buffer_length;
///
/// println!("{}", insufficient_buffer_length(1, 0));
/// ```
///
/// [`InsufficientBufferLength`]: InsufficientBufferLength
pub fn insufficient_buffer_length(expected: usize, actual: usize) -> IOError {
    IOError::new(
        ErrorKind::InvalidInput,
        InsufficientBufferLength {
            expected,
            actual
        }
    )
}
