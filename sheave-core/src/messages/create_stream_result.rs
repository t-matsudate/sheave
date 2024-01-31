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
            AmfString,
            Null
        },
        ensure_command_name,
        headers::MessageType
    }
};

/// The response message for CreateStream requests.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreateStreamResult {
    result: AmfString,
    transaction_id: Number,
    message_id: Number
}

impl CreateStreamResult {
    /// Constructs a CreateStreamResult command.
    pub fn new(result: AmfString, transaction_id: Number, message_id: Number) -> Self {
        Self { result, transaction_id, message_id }
    }

    /// Gets the message ID which is assigned to this stream.
    pub fn get_message_id(&self) -> Number {
        self.message_id
    }
}

impl From<CreateStreamResult> for Number {
    fn from(create_stream_result: CreateStreamResult) -> Self {
        create_stream_result.message_id
    }
}

impl ChunkData for CreateStreamResult {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for CreateStreamResult {
    /// Gets the result which is either `"_result"` or `"_error"`.
    fn get_command_name(&self) -> &str {
        &**self.result
    }

    /// Gets the transaction ID in this response.
    fn get_transaction_id(&self) -> Number {
        self.transaction_id
    }
}

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
    /// * [`InvalidString`]
    ///
    /// When the command name is invalid for UTF-8 string.
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
    ///         CreateStreamResult,
    ///         amf::v0::{
    ///             Number,
    ///             AmfString,
    ///             Null
    ///         };
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("_result"));
    /// buffer.encode(&Number::new(4f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&Number::default());
    /// assert!(Decoder::<CreateStreamResult>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&Number::new(4f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&Number::default());
    /// assert!(Decoder::<CreateStreamResult>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<CreateStreamResult> {
        let result: AmfString = self.decode()?;
        ensure_command_name("_result", result.clone()).or(ensure_command_name("_error", result.clone()))?;
        let transaction_id: Number = self.decode()?;
        Decoder::<Null>::decode(self)?;
        let message_id: Number = self.decode()?;
        Ok(CreateStreamResult { result, transaction_id, message_id })
    }
}

impl Encoder<CreateStreamResult> for ByteBuffer {
    /// Encodes a CreateStreamResult command into bytes.
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
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// let expected_transaction_id = 4f64;
    /// let expected_message_id = f64::default();
    /// buffer.encode(&CreateStreamResult::new(expected_transaction_id, Number::default()));
    /// let command_name: AmfString = buffer.decode().unwrap();
    /// let actual_transaction_id: Number = buffer.decode().unwrap();
    /// Decoder::<Null>::decode(&mut buffer).unwrap();
    /// let actual_message_id: Number = buffer.decode().unwrap();
    /// assert_eq!("_result", command_name);
    /// assert_eq!(expected_transaction_id, actual_transaction_id);
    /// assert_eq!(expected_message_id, actual_message_id)
    /// ```
    fn encode(&mut self, create_stream_result: &CreateStreamResult) {
        self.encode(&AmfString::from(create_stream_result.get_command_name()));
        self.encode(&create_stream_result.get_transaction_id());
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
        buffer.encode(&AmfString::from("_result"));
        buffer.encode(&Number::new(4f64));
        buffer.encode(&Null);
        buffer.encode(&Number::default());
        let result: IOResult<CreateStreamResult> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = CreateStreamResult::new("_result".into(), 4.into(), Number::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_create_stream_result() {
        let mut buffer = ByteBuffer::default();
        let expected_transaction_id = 4f64;
        let expected_message_id = 0f64;
        buffer.encode(&CreateStreamResult::new("_result".into(), Number::new(expected_transaction_id), Number::new(expected_message_id)));
        let command_name: AmfString = buffer.decode().unwrap();
        assert_eq!("_result", command_name);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_message_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_message_id, actual_message_id)
    }
}
