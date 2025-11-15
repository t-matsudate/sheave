use std::io::Result as IOResult;
use super::{
    Marker,
    super::ensure_marker
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer
};

/// The value to mean that has no content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Null;

impl Decoder<Null> for ByteBuffer {
    /// Decodes bytes into an AMF's Null.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When buffer isn't remained at least 1 byte.
    ///
    /// * [`InconsistentMarker`]
    ///
    /// When a marker byte doesn't indicate the AMF Null.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     messages::amf::v0::{
    ///         Marker,
    ///         Null
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Null as u8);
    /// assert!(Decoder::<Null>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u8(Marker::Other as u8);
    /// assert!(Decoder::<Null>::decode(&mut buffer).is_err());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<Null>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<Null> {
        self.get_u8().and_then(
            |marker| ensure_marker(Marker::Null as u8, marker)
        )?;

        Ok(Null)
    }
}

impl Encoder<Null> for ByteBuffer {
    /// Encodes an AMF's Null into bytes.
    fn encode(&mut self, _: &Null) {
        self.put_u8(Marker::Null as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_null() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u8(Marker::Null as u8);
        assert!(Decoder::<Null>::decode(&mut buffer).is_ok())
    }

    #[test]
    fn encode_null() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        let result: Vec<u8> = buffer.into();
        assert_eq!(Marker::Null as u8, result[0])
    }
}
