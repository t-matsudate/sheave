//! # The Action Message Formats
//!
//! These are data types which are defined as the Action Message Format.
//! There are two formats which are version 0 and version 3 in AMF.
//! Currently the RTMP uses only AMF version 0.

pub mod v0;
mod inconsistent_marker;
mod invalid_string;

use std::io::Result as IOResult;
pub use self::{
    inconsistent_marker::*,
    invalid_string::*
};

#[doc(hidden)]
pub(self) fn ensure_marker(expected: u8, actual: u8) -> IOResult<()> {
    (expected == actual).then_some(()).ok_or(inconsistent_marker(expected, actual))
}
