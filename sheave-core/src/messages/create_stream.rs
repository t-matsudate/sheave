use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        Channel,
        ChunkData,
        Command,
        amf::v0::Null,
        headers::MessageType
    }
};

/// The command to request emitting a message ID to a server.
///
/// Following format is required:
///
/// |Field|AMF Type|Value|
/// | :- | :- | :- |
/// ||[`Null`]|Nothing but an AMF's type marker is in.|
///
/// [`Null`]: crate::messages::amf::v0::Null
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CreateStream;

impl ChunkData for CreateStream {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for CreateStream {}

impl Decoder<CreateStream> for ByteBuffer {
    /// Decodes bytes into a CreateStream command.
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
    ///         CreateStream,
    ///         amf::v0::Null,
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// assert!(Decoder::<CreateStream>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<CreateStream>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<CreateStream> {
        Decoder::<Null>::decode(self)?;
        Ok(CreateStream)
    }
}

impl Encoder<CreateStream> for ByteBuffer {
    /// Encodes a CreateSteam command into bytes.
    fn encode(&mut self, _: &CreateStream) {
        self.encode(&Null);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_create_stream() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        let result: IOResult<CreateStream> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = CreateStream;
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_create_stream() {
        let mut buffer = ByteBuffer::default();
        let expected = CreateStream;
        buffer.encode(&expected);
        assert!(Decoder::<CreateStream>::decode(&mut buffer).is_ok())
    }
}
