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

/// The command to tell the Play Path (e.g. something file name).
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseStream {
    transaction_id: Number,
    play_path: AmfString
}

impl ReleaseStream {
    const COMMAND_NAME: &'static str = "releaseStream";

    /// Constructs a ReleaseStream command.
    pub fn new(transaction_id: Number, play_path: AmfString) -> Self {
        Self { transaction_id, play_path }
    }

    /// Gets the Play Path.
    pub fn get_play_path(&self) -> &AmfString {
        &self.play_path
    }
}

impl From<ReleaseStream> for AmfString {
    fn from(release_stream: ReleaseStream) -> Self {
        release_stream.play_path
    }
}

impl ChunkData for ReleaseStream {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for ReleaseStream {
    fn get_command_name(&self) -> &str {
        Self::COMMAND_NAME
    }

    fn get_transaction_id(&self) -> Number {
        self.transaction_id
    }
}

impl Decoder<ReleaseStream> for ByteBuffer {
    /// Decodes bytes into a ReleaseStream command.
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
    /// When the command name isn't `"releaseStream"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         ReleaseStream,
    ///         amf::v0::{
    ///             Number,
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("releaseStream"));
    /// buffer.encode(&Number::new(2f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// assert!(Decoder::<ReleaseStream>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// buffer.encode(&Number::new(2f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// assert!(Decoder::<ReleaseStream>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<ReleaseStream> {
        Decoder::<AmfString>::decode(self).and_then(
            |command| ensure_command_name("releaseStream", command)
        )?;

        let transaction_id: Number = self.decode()?;
        Decoder::<Null>::decode(self)?;
        let play_path: AmfString = self.decode()?;
        Ok(ReleaseStream { transaction_id, play_path })
    }
}

impl Encoder<ReleaseStream> for ByteBuffer {
    /// Encodes a ReleaseStream command into bytes.
    fn encode(&mut self, release_stream: &ReleaseStream) {
        self.encode(&AmfString::from(release_stream.get_command_name()));
        self.encode(&release_stream.get_transaction_id());
        self.encode(&Null);
        self.encode(release_stream.get_play_path());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_release_stream() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&AmfString::from("releaseStream"));
        buffer.encode(&Number::new(2f64));
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());
        let result: IOResult<ReleaseStream> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = ReleaseStream::new(2.into(), AmfString::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_release_stream() {
        let mut buffer = ByteBuffer::default();
        let expected_transaction_id = 2f64;
        let expected_play_path = "";
        let expected = ReleaseStream::new(Number::new(expected_transaction_id), AmfString::from(expected_play_path));
        buffer.encode(&expected);
        let command_name: AmfString = buffer.decode().unwrap();
        assert_eq!("releaseStream", command_name);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_play_path: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_play_path, actual_play_path)
    }
}
