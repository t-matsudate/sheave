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

/// The command to request emitting a message ID to a server.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CreateStream(Number);

impl CreateStream {
    const COMMAND_NAME: &'static str = "createStream";

    /// Constructs a CreateStream command.
    pub fn new(transaction_id: Number) -> Self {
        Self(transaction_id)
    }
}

impl ChunkData for CreateStream {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for CreateStream {
    fn get_command_name(&self) -> &str {
        Self::COMMAND_NAME
    }

    fn get_transaction_id(&self) -> Number {
        self.0
    }
}

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
    /// * [`InvalidString`]
    ///
    /// When the command name is invalid for UTF-8 string.
    ///
    /// * [`InconsistentCommand`]
    ///
    /// When the command name isn't `"createStream"`.
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
    ///         amf::v0::{
    ///             Number,
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("createStream"));
    /// buffer.encode(&Number::new(4f64));
    /// buffer.encode(&Null);
    /// assert!(Decoder::<CreateStream>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&Number::new(4f64));
    /// buffer.encode(&Null);
    /// assert!(Decoder::<CreateStream>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<CreateStream> {
        Decoder::<AmfString>::decode(self).and_then(
            |command| ensure_command_name("createStream", command)
        )?;

        let transaction_id: Number = self.decode()?;
        Decoder::<Null>::decode(self)?;
        Ok(CreateStream(transaction_id))
    }
}

impl Encoder<CreateStream> for ByteBuffer {
    /// Encodes a CreateSteam command into bytes.
    fn encode(&mut self, create_stream: &CreateStream) {
        self.encode(&AmfString::from(create_stream.get_command_name()));
        self.encode(&create_stream.get_transaction_id());
        self.encode(&Null);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_create_stream() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("createStream"));
        buffer.encode(&Number::new(4f64));
        buffer.encode(&Null);
        let result: IOResult<CreateStream> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = CreateStream::new(4.into());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_create_stream() {
        let mut buffer = ByteBuffer::default();
        let expected_transaction_id = 4f64;
        let expected = CreateStream::new(Number::new(expected_transaction_id));
        buffer.encode(&expected);
        let command_name: AmfString = buffer.decode().unwrap();
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!("createStream", command_name);
        assert_eq!(expected_transaction_id, actual_transaction_id)
    }
}
