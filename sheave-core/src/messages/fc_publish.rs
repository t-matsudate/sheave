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
        AmfString,
        Null
    }
};

/// The command to tell the playpath.
/// Typically, this becomes same as the releaseStream's one.
#[derive(Debug, Clone, PartialEq)]
pub struct FcPublish {
    transaction_id: Number,
    playpath: AmfString
}

impl FcPublish {
    /// Constructs a FcPublish command.
    pub fn new(transaction_id: Number, play_path: AmfString) -> Self {
        Self { transaction_id, play_path }
    }

    /// Gets the playpath.
    pub fn get_playpath(&self) -> &AmfString {
        &self.playpath
    }
}

impl From<FcPublish> for AmfString {
    fn from(fc_publish: FcPublish) -> Self {
        fc_publish.playpath
    }
}

impl ChunkData for FcPublish {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for FcPublish {
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
    /// buffer.encode(&Number::new(3f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// assert!(Decoder::<FcPublish>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<FcPublish>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<FcPublish> {
        let transaction_id: Number = self.decode()?;
        Decoder::<Null>::decode(self)?;
        let playpath: AmfString = self.decode()?;
        Ok(FcPublish { transaction_id, playpath })
    }
}

impl Encoder<FcPublish> for ByteBuffer {
    /// Encodes a FcPublish command into bytes.
    fn encode(&mut self, fc_publish: &FcPublish) {
        self.encode(&fc_publish.get_transaction_id());
        self.encode(&Null);
        self.encode(fc_publish.get_playpath());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_fc_publish() {
        let mut buffer = ByteBuffer::default();
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
        let expected_playpath = "";
        let expected = FcPublish::new(Number::new(expected_transaction_id), AmfString::from(expected_playpath));
        buffer.encode(&expected);
        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_playpath: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_playpath, actual_playpath)
    }
}
