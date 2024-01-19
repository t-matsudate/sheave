use std::io::Result as IOResult;
use super::{
    Channel,
    ChunkData,
    Command,
    ensure_command_name,
    headers::MessageType
};
use crate::{
    Decoder,
    Encoder,
    ByteBuffer,
    messages::amf::v0::{
        Number,
        AmfString,
        Null
    }
};

/// The response message for releaseStream requests.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseStreamResult {
    result: AmfString,
    transaction_id: Number
}

impl ReleaseStreamResult {
    /// Constructs a ReleaseStreamResult command.
    pub fn new(result: AmfString, transaction_id: Number) -> Self {
        Self { result, transaction_id }
    }
}

impl ChunkData for ReleaseStreamResult {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for ReleaseStreamResult {
    /// Gets the result which is either `"_result"` or `"_error"`.
    fn get_command_name(&self) -> &str {
        &**self.result
    }

    /// Gets the transaction ID in this response.
    fn get_transaction_id(&self) -> Number {
        self.transaction_id
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
    /// * [`InvalidString`]
    ///
    /// When some value is invalid for UTF-8 string.
    ///
    /// * [`InconsistentCommand`]
    ///
    /// When the command name is neither `"_result"` nor `"_error"`.
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
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("_result"));
    /// buffer.encode(&Number::new(2f64));
    /// buffer.encode(&Null);
    /// assert!(Decoder::<ReleaseStreamResult>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&Number::new(2f64));
    /// buffer.encode(&Null);
    /// assert!(Decoder::<ReleaseStreamResult>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<ReleaseStreamResult> {
        let result: AmfString = self.decode()?;
        ensure_command_name("_result", result.clone()).or(ensure_command_name("_error", result.clone()))?;
        let transaction_id: Number = self.decode()?;
        Decoder::<Null>::decode(self)?;
        Ok(ReleaseStreamResult { result, transaction_id })
    }
}

impl Encoder<ReleaseStreamResult> for ByteBuffer {
    /// Encodes a ReleaseStreamResult command into bytes.
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
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&ReleaseStreamResult::new("_result".into(), 2.into()));
    /// let command_name: AmfString = buffer.decode().unwrap();
    /// let transaction_id: Number = buffer.decode().unwrap();
    /// Decoder::<Null>::decode(&mut buffer).unwrap();
    /// assert_eq!("_result", command_name);
    /// assert_eq!(2f64, transaction_id)
    /// ```
    fn encode(&mut self, release_stream_result: &ReleaseStreamResult) {
        self.encode(&AmfString::from(release_stream_result.get_command_name()));
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
        buffer.encode(&AmfString::from("_result"));
        buffer.encode(&Number::new(2f64));
        buffer.encode(&Null);
        let result: IOResult<ReleaseStreamResult> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = ReleaseStreamResult::new("_result".into(), 2.into());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_release_stream_result() {
        let mut buffer = ByteBuffer::default();
        let expected_transaction_id = 2f64;
        let expected = ReleaseStreamResult::new("_result".into(), Number::new(expected_transaction_id));
        buffer.encode(&expected);
        let command_name: AmfString = buffer.decode().unwrap();
        assert_eq!("_result", command_name);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id)
    }
}
