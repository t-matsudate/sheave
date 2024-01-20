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
        AmfString
    }
};

/// The response message for FcPublish requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnFcPublish;

impl OnFcPublish {
    const COMMAND_NAME: &'static str = "onFCPublish";
}

impl ChunkData for OnFcPublish {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for OnFcPublish {
    /// Gets the result that's just "onFCPublish".
    fn get_command_name(&self) -> &str {
        Self::COMMAND_NAME
    }

    /// Gets the transaction ID in this response.
    fn get_transaction_id(&self) -> Number {
        unimplemented!("onFCPublish has no transaction ID.")
    }
}

impl Decoder<OnFcPublish> for ByteBuffer {
    /// Decodes bytes into a OnFcPublish command.
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
    /// When the command name isn't "onFCPublish".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         OnFcPublish,
    ///         amf::v0::AmfString
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("onFCPublish"));
    /// assert!(Decoder::<OnFcPublish>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// assert!(Decoder::<OnFcPublish>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<OnFcPublish> {
        Decoder::<AmfString>::decode(self).and_then(
            |command| ensure_command_name("onFCPublish", command)
        ).map(
            |_| OnFcPublish
        )
    }
}

impl Encoder<OnFcPublish> for ByteBuffer {
    /// Encodes a OnFcPublish command into bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         OnFcPublish,
    ///         amf::v0::AmfString
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&OnFcPublish);
    /// let command_name: AmfString = buffer.decode().unwrap();
    /// assert_eq!("onFCPublish", command_name)
    /// ```
    fn encode(&mut self, on_fc_publish: &OnFcPublish) {
        self.encode(&AmfString::from(on_fc_publish.get_command_name()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_on_fc_publish() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("onFCPublish"));
        let result: IOResult<OnFcPublish> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(OnFcPublish, actual)
    }

    #[test]
    fn encode_on_fc_publish() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&OnFcPublish);
        let command_name: AmfString = buffer.decode().unwrap();
        assert_eq!("onFCPublish", command_name)
    }
}
