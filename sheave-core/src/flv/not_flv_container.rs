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

/// An error that specified file isn't the FLV format.
#[derive(Debug)]
pub struct NotFlvContainer(String);

impl Display for NotFlvContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Signature bytes are inconsistent: expected \"FLV\", actual {}.", &self.0)
    }
}

impl Error for NotFlvContainer {}

/// A utility function of constructing a `NotFlvContainer` error.
pub fn not_flv_container<'a>(signature_bytes: &'a [u8]) -> IOError {
    let signature = String::from_utf8(signature_bytes.into()).unwrap();
    IOError::new(
        ErrorKind::InvalidInput,
        NotFlvContainer(signature)
    )
}
