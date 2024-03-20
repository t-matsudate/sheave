use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        Channel,
        ChunkData,
        headers::MessageType
    }
};

/// The message to tell that some message length has exceeded the server-side bandwidth range.
/// Note this must be input the total message length in receiving. (it's not bytes received.)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Acknowledgement(u32);

impl Acknowledgement {
    /// Constucts a Acknowledgement message.
    pub fn new(acknowledgement: u32) -> Self {
        Self(acknowledgement)
    }
}

impl PartialEq<u32> for Acknowledgement {
    fn eq(&self, other: &u32) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Acknowledgement> for u32 {
    fn eq(&self, other: &Acknowledgement) -> bool {
        self.eq(&other.0)
    }
}

impl ChunkData for Acknowledgement {
    const CHANNEL: Channel = Channel::Network;
    const MESSAGE_TYPE: MessageType = MessageType::Acknowledgement;
}

impl Decoder<Acknowledgement> for ByteBuffer {
    /// Decodes bytes into a Acknowledgement message.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When some field misses.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     messages::{
    ///         Acknowledgement
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u32_be(u32::default());
    /// assert!(Decoder::<Acknowledgement>::decode(&mut buffer).is_ok())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    fn decode(&mut self) -> IOResult<Acknowledgement> {
        self.get_u32_be().map(Acknowledgement::new)
    }
}

impl Encoder<Acknowledgement> for ByteBuffer {
    /// Encodes a Acknowledgement message into bytes.
    fn encode(&mut self, acknowledgement: &Acknowledgement) {
        self.put_u32_be(acknowledgement.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_acknowledgement() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u32_be(u32::default());
        let result: IOResult<Acknowledgement> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = Acknowledgement::default();
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_acknowledgement() {
        let mut buffer = ByteBuffer::default();
        let expected_bytes = 0u32;
        buffer.encode(&Acknowledgement::new(expected_bytes));
        let actual_bytes = buffer.get_u32_be().unwrap();
        assert_eq!(expected_bytes, actual_bytes)
    }
}
