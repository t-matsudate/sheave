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

/// An error that some tag type is unknown/undefined.
#[derive(Debug)]
pub struct UnknownTag(u8);

impl Display for UnknownTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Unknown FLV tag: {}", self.0)
    }
}

impl Error for UnknownTag {}

/// A utility function of constructing an `UnknownTag` error.
pub fn unknown_tag(tag_type: u8) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        UnknownTag(tag_type)
    )
}
