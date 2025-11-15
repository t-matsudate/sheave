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

/// The response message for GetStreamLength requests.
///
/// Note this command name starts with GetStreamLength but actual input is a duration in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GetStreamLengthResult(Number);

impl GetStreamLengthResult {
    /// Constructs a GetStreamLengthResult command.
    pub fn new(duration: Number) -> Self {
        Self(duration)
    }

    /// Gets the duration.
    pub fn get_duration(&self) -> Number {
        self.0
    }
}

impl ChunkData for GetStreamLengthResult {
    const CHANNEL: Channel = Channel::Source;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for GetStreamLengthResult {}

impl Decoder<GetStreamLengthResult> for ByteBuffer {
    /// Decodes bytes into a GetStreamLengthResult command.
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
    ///         GetStreamLengthResult,
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
    /// assert!(Decoder::<GetStreamLengthResult>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<GetStreamLengthResult>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<GetStreamLengthResult> {
        Decoder::<Null>::decode(self)?;
        let duration: Number = self.decode()?;
        Ok(GetStreamLengthResult(duration))
    }
}

impl Encoder<GetStreamLengthResult> for ByteBuffer {
    /// Encodes a GetStreamLengthResult command into bytes.
    fn encode(&mut self, get_stream_length_result: &GetStreamLengthResult) {
        self.encode(&Null);
        self.encode(&get_stream_length_result.get_duration());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_get_stream_length_result() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(&Number::default());
        let result: IOResult<GetStreamLengthResult> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = GetStreamLengthResult::new(Number::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_get_stream_length_result() {
        let mut buffer = ByteBuffer::default();
        let expected_duration = 0f64;
        buffer.encode(&GetStreamLengthResult::new(Number::new(expected_duration)));
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_duration: Number = buffer.decode().unwrap();
        assert_eq!(expected_duration, actual_duration)
    }
}
