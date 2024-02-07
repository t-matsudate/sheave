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

#[derive(Debug)]
pub struct InvalidPublishingType(AmfString);

impl InvalidPublishingType {
    pub fn new(actual: AmfString) -> Self {
        Self(actual)
    }
}

impl Display for InvalidPublishingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "Publishing type is neither \"live\", \"record\", nor \"append\". actual: {}", self.0)
    }
}

impl Error for InvalidPublishingType {}

pub fn invalid_publishing_type(actual: AmfString) -> IOError {
    IOError::new(
        ErrorKind::InvalidData,
        InvalidPublishingType(actual)
    )
}
