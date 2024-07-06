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

/// The response message for ReleaseStream requests.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseStreamResult(Number);

impl ReleaseStreamResult {
    /// Constructs a ReleaseStreamResult command.
    pub fn new(transaction_id: Number) -> Self {
        Self(transaction_id)
    }
}

impl ChunkData for ReleaseStreamResult {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for ReleaseStreamResult {
    /// Gets the transaction ID in this response.
    fn get_transaction_id(&self) -> Number {
        self.0
    }
}

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
    ///         amf::v0::{
    ///             Number,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Number::new(2f64));
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
        let transaction_id: Number = self.decode()?;
        Decoder::<Null>::decode(self)?;
        Ok(ReleaseStreamResult(transaction_id))
    }
}

impl Encoder<ReleaseStreamResult> for ByteBuffer {
    /// Encodes a ReleaseStreamResult command into bytes.
    fn encode(&mut self, release_stream_result: &ReleaseStreamResult) {
        self.encode(&release_stream_result.get_transaction_id());
        self.encode(&Null);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_release_stream_result() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Number::new(2f64));
        buffer.encode(&Null);
        let result: IOResult<ReleaseStreamResult> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = ReleaseStreamResult::new(2.into());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_release_stream_result() {
        let mut buffer = ByteBuffer::default();
        let expected_transaction_id = 2f64;
        let expected = ReleaseStreamResult::new(Number::new(expected_transaction_id));
        buffer.encode(&expected);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id)
    }
}
