use std::{
    cmp::Ordering,
    fmt::{
        Display,
        Formatter,
        Result as FormatResult
    },
    io::Result as IOResult
};
use super::{
    Marker,
    super::ensure_marker
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer
};

/// The boolean representation of AMF data types.
/// This uses 1 byte unsigned integer as a boolean value.
/// Usually, `0` is treated as `false`, else is `true`.
///
/// # Examples
///
/// ```rust
/// use sheave_core::messages::amf::v0::Boolean;
///
/// assert_eq!(false, Boolean::new(0));
/// assert_eq!(true, Boolean::new(1));
/// assert_eq!(true, Boolean::new(u8::MAX))
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Boolean(u8);

impl Boolean {
    /// Constructs an AMF's Boolean.
    pub fn new(boolean: u8) -> Self {
        Self(boolean)
    }

    /// Gets an inner value as a boolean value.
    pub fn as_boolean(&self) -> bool {
        self.0 > 0
    }
}

impl Display for Boolean {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        writeln!(f, "{}", self.0 != 0)
    }
}

impl PartialEq<bool> for Boolean {
    fn eq(&self, other: &bool) -> bool {
        self.as_boolean().eq(other)
    }
}

impl PartialEq<Boolean> for bool {
    fn eq(&self, other: &Boolean) -> bool {
        self.eq(&other.as_boolean())
    }
}

impl PartialOrd<bool> for Boolean {
    fn partial_cmp(&self, other: &bool) -> Option<Ordering> {
        self.as_boolean().partial_cmp(other)
    }
}

impl PartialOrd<Boolean> for bool {
    fn partial_cmp(&self, other: &Boolean) -> Option<Ordering> {
        self.partial_cmp(&other.as_boolean())
    }
}

impl Decoder<Boolean> for ByteBuffer {
    /// Decodes bytes into an AMF's Boolean.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When buffer isn't remained at least 2 bytes.
    ///
    /// * [`InconsistentMarker`]
    ///
    /// When a marker byte doesn't indicate the AMF Boolean.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     messages::amf::v0::{
    ///         Marker,
    ///         Boolean
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Boolean as u8);
    /// buffer.put_u8(0);
    /// assert!(Decoder::<Boolean>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Other as u8);
    /// buffer.put_u8(0);
    /// assert!(Decoder::<Boolean>::decode(&mut buffer).is_err());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<Boolean>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<Boolean> {
        self.get_u8().and_then(
            |marker| ensure_marker(Marker::Boolean as u8, marker)
        )?;

        self.get_u8().map(Boolean::new)
    }
}

impl Encoder<Boolean> for ByteBuffer {
    /// Encodes an AMF's Boolean into bytes.
    fn encode(&mut self, boolean: &Boolean) {
        self.put_u8(Marker::Boolean as u8);
        self.put_u8(boolean.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_boolean() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u8(Marker::Boolean as u8);
        buffer.put_u8(0);
        let result: IOResult<Boolean> = buffer.decode();
        assert!(result.is_ok());
        let boolean = result.unwrap();
        assert!(!boolean.as_boolean())
    }

    #[test]
    fn encode_boolean() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Boolean::new(0));
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::Boolean as u8, result[0]);
        assert_eq!(0, result[1])
    }
}
