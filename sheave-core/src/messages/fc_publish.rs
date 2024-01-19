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

/// The command to tell the Play Path.
/// Typically, this becomes same as the releaseStream's one.
#[derive(Debug, Clone, PartialEq)]
pub struct FcPublish {
    transaction_id: Number,
    play_path: AmfString
}

impl FcPublish {
    const COMMAND_NAME: &'static str = "FCPublish";

    /// Constructs a FcPublish command.
    pub fn new(transaction_id: Number, play_path: AmfString) -> Self {
        Self { transaction_id, play_path }
    }

    /// Gets the Play Path.
    pub fn get_play_path(&self) -> &AmfString {
        &self.play_path
    }
}

impl From<FcPublish> for AmfString {
    fn from(fc_publish: FcPublish) -> Self {
        fc_publish.play_path
    }
}

impl ChunkData for FcPublish {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for FcPublish {
    fn get_command_name(&self) -> &str {
        Self::COMMAND_NAME
    }

    fn get_transaction_id(&self) -> Number {
        self.transaction_id
    }
}

impl Decoder<FcPublish> for ByteBuffer {
    /// Decodes bytes into a FcPublish command.
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
    /// When the command name isn't `"FCPublish"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         FcPublish,
    ///         amf::v0::{
    ///             Number,
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("FCPublish"));
    /// buffer.encode(&Number::new(3f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// assert!(Decoder::<FcPublish>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&Number::new(3f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// assert!(Decoder::<FcPublish>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<FcPublish> {
        Decoder::<AmfString>::decode(self).and_then(
            |command| ensure_command_name("FCPublish", command)
        )?;

        let transaction_id: Number = self.decode()?;
        Decoder::<Null>::decode(self)?;
        let play_path: AmfString = self.decode()?;
        Ok(FcPublish { transaction_id, play_path })
    }
}

impl Encoder<FcPublish> for ByteBuffer {
    /// Encodes a FcPublish command into bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         FcPublish,
    ///         amf::v0::{
    ///             Number,
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&FcPublish::new(3.into(), AmfString::default()));
    /// let command_name: AmfString = buffer.decode().unwrap();
    /// let transaction_id: Number = buffer.decode().unwrap();
    /// Decoder::<Null>::decode(&mut buffer).unwrap();
    /// let play_path: AmfString = buffer.decode().unwrap();
    /// assert_eq!("FCPublish", command_name);
    /// assert_eq!(3f64, transaction_id);
    /// assert_eq!(AmfString::default(), play_path)
    /// ```
    fn encode(&mut self, fc_publish: &FcPublish) {
        self.encode(&AmfString::from(fc_publish.get_command_name()));
        self.encode(&fc_publish.get_transaction_id());
        self.encode(&Null);
        self.encode(fc_publish.get_play_path());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_fc_publish() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("FCPublish"));
        buffer.encode(&Number::new(3f64));
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());
        let result: IOResult<FcPublish> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = FcPublish::new(3.into(), AmfString::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_fc_publish() {
        let mut buffer = ByteBuffer::default();
        let expected_transaction_id = 3f64;
        let expected_play_path = "";
        let expected = FcPublish::new(Number::new(expected_transaction_id), AmfString::from(expected_play_path));
        buffer.encode(&expected);
        let command_name: AmfString = buffer.decode().unwrap();
        assert_eq!("FCPublish", command_name);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_play_path: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_play_path, actual_play_path)
    }
}
