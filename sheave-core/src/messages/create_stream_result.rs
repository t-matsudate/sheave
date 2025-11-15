use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        Channel,
        ChunkData,
        Command,
        amf::v0::{
            Number,
            Null
        },
        headers::MessageType
    }
};

/// The response message for CreateStream requests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreateStreamResult(Number);

impl CreateStreamResult {
    /// Constructs a CreateStreamResult command.
    pub fn new(message_id: Number) -> Self {
        Self(message_id)
    }

    /// Gets the message ID which is assigned to this stream.
    pub fn get_message_id(&self) -> Number {
        self.0
    }
}

impl From<CreateStreamResult> for u32 {
    fn from(create_stream_result: CreateStreamResult) -> Self {
        create_stream_result.0.as_integer() as u32
    }
}

impl ChunkData for CreateStreamResult {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for CreateStreamResult {}

impl Decoder<CreateStreamResult> for ByteBuffer {
    /// Decodes bytes into a CreateStreamResult command.
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
    ///         CreateStreamResult,
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
    /// assert!(Decoder::<CreateStreamResult>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<CreateStreamResult>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<CreateStreamResult> {
        Decoder::<Null>::decode(self)?;
        let message_id: Number = self.decode()?;
        Ok(CreateStreamResult(message_id))
    }
}

impl Encoder<CreateStreamResult> for ByteBuffer {
    /// Encodes a CreateStreamResult command into bytes.
    fn encode(&mut self, create_stream_result: &CreateStreamResult) {
        self.encode(&Null);
        self.encode(&create_stream_result.get_message_id());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_create_stream_result() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(&Number::default());
        let result: IOResult<CreateStreamResult> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = CreateStreamResult::new(Number::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_create_stream_result() {
        let mut buffer = ByteBuffer::default();
        let expected_message_id = 0f64;
        buffer.encode(&CreateStreamResult::new(Number::new(expected_message_id)));
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_message_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_message_id, actual_message_id)
    }
}
