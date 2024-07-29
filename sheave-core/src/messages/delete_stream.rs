use std::io::Result as IOResult;
use super::{
    Channel,
    ChunkData,
    Command,
    headers::MessageType
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer,
    messages::amf::v0::{
        Number,
        Null
    }
};

/// The command to request to delete its message ID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeleteStream(Number);

impl DeleteStream {
    /// Constructs a DeleteStream command.
    pub fn new(message_id: Number) -> Self {
        Self(message_id)
    }

    /// Gets the message ID.
    pub fn get_message_id(&self) -> Number {
        self.0
    }
}

impl From<DeleteStream> for u32 {
    fn from(delete_stream: DeleteStream) -> Self {
        delete_stream.0.as_integer() as u32
    }
}

impl ChunkData for DeleteStream {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for DeleteStream {}

impl Decoder<DeleteStream> for ByteBuffer {
    /// Decodes bytes into a DeleteStream command.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When some field misses.
    ///
    /// * [`InconsistentMarker`]
    ///
    /// When some value is inconsistent with its marker.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         DeleteStream,
    ///         amf::v0::{
    ///             Number,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// buffer.encode(&Number::default());
    /// assert!(Decoder::<DeleteStream>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<DeleteStream>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<DeleteStream> {
        Decoder::<Null>::decode(self)?;
        let message_id: Number = self.decode()?;

        Ok(DeleteStream(message_id))
    }
}

impl Encoder<DeleteStream> for ByteBuffer {
    /// Encodes a DeleteStream command into bytes.
    fn encode(&mut self, delete_stream: &DeleteStream) {
        self.encode(&Null);
        self.encode(&delete_stream.get_message_id());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_delete_stream() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(&Number::default());

        let result: IOResult<DeleteStream> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = DeleteStream::new(Number::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_delete_stream() {
        let mut buffer = ByteBuffer::default();
        let expected_message_id = 0f64;
        let expected = DeleteStream::new(Number::new(expected_message_id));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_message_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_message_id, actual_message_id)
    }
}
