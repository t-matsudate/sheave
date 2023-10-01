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
    },
    string::FromUtf8Error
};

/// An error means that some string data is invalid for UTF-8.
#[derive(Debug)]
pub struct InvalidString(FromUtf8Error);

impl InvalidString {
    /// Constructs this error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::InvalidString;
    ///
    /// // This is a missing sequence of the "sparkle heart(💖)".
    /// let bytes = vec![0, 159, 146, 150];
    /// let e = String::from_utf8(bytes).err().unwrap();
    /// InvalidString::new(e);
    /// ```
    pub fn new(e: FromUtf8Error) -> Self {
        Self(e)
    }
}

impl Display for InvalidString {
    /// Displays this as a formatted string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::messages::amf::InvalidString;
    ///
    /// // This is a missing sequence of the "sparkle heart(💖)".
    /// let bytes = vec![0, 159, 146, 150];
    /// let e = String::from_utf8(bytes).err().unwrap();
    /// println!("{}", InvalidString::new(e));
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        Display::fmt(&self.0, f)
    }
}

impl Error for InvalidString {}

/// A utility function of constructing an `InvalidString` error.
///
/// # Examples
///
/// ```rust
/// use sheave_core::messages::amf::invalid_string;
///
/// // This is a missing sequence of the "sparkle heart(💖)".
/// let bytes = vec![0, 159, 146, 150];
/// let e = String::from_utf8(bytes).err().unwrap();
/// println!("{}", invalid_string(e));
/// ```
///
/// [`InvalidString`]: InvalidString
pub fn invalid_string(e: FromUtf8Error) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InvalidString(e)
    )
}
