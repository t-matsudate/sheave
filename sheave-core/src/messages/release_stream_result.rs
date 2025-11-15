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
    messages::amf::v0::Null,
};

/// The response message for ReleaseStream requests.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseStreamResult;

impl ChunkData for ReleaseStreamResult {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for ReleaseStreamResult {}

impl Decoder<ReleaseStreamResult> for ByteBuffer {
    /// Decodes bytes into a ReleaseStreamResult command.
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
    ///         ReleaseStreamResult,
    ///         amf::v0::Null,
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// assert!(Decoder::<ReleaseStreamResult>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<ReleaseStreamResult>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<ReleaseStreamResult> {
        Decoder::<Null>::decode(self)?;
        Ok(ReleaseStreamResult)
    }
}

impl Encoder<ReleaseStreamResult> for ByteBuffer {
    /// Encodes a ReleaseStreamResult command into bytes.
    fn encode(&mut self, _: &ReleaseStreamResult) {
        self.encode(&Null);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_release_stream_result() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        let result: IOResult<ReleaseStreamResult> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = ReleaseStreamResult;
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_release_stream_result() {
        let mut buffer = ByteBuffer::default();
        let expected = ReleaseStreamResult;
        buffer.encode(&expected);
        assert!(Decoder::<ReleaseStreamResult>::decode(&mut buffer).is_ok())
    }
}
