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

/// This command is same as [`FcPublish`] except this requests to delete the playpath.
///
/// [`FcPublish`]: super::FcPublish
#[derive(Debug, Clone, PartialEq)]
pub struct FcUnpublish {
    transaction_id: Number,
    playpath: AmfString
}

impl FcUnpublish {
    /// Constructs a FcUnpublish command.
    pub fn new(transaction_id: Number, play_path: AmfString) -> Self {
        Self { transaction_id, playpath }
    }

    /// Gets the playpath.
    pub fn get_playpath(&self) -> &AmfString {
        &self.playpath
    }
}

impl From<FcUnpublish> for AmfString {
    fn from(fc_unpublish: FcUnpublish) -> Self {
        fc_unpublish.playpath
    }
}

impl ChunkData for FcUnpublish {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for FcUnpublish {
    fn get_transaction_id(&self) -> Number {
        self.transaction_id
    }
}

impl Decoder<FcUnpublish> for ByteBuffer {
    /// Decodes bytes into a FcUnpublish command.
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
    ///         FcUnpublish,
    ///         amf::v0::{
    ///             Number,
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Number::new(6f64));
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// assert!(Decoder::<FcUnpublish>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<FcUnpublish>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<FcUnpublish> {
        let transaction_id: Number = self.decode()?;
        Decoder::<Null>::decode(self)?;
        let playpath: AmfString = self.decode()?;

        Ok(FcUnpublish { transaction_id, playpath })
    }
}

impl Encoder<FcUnpublish> for ByteBuffer {
    /// Encodes a FcUnpublish command into bytes.
    fn encode(&mut self, fc_unpublish: &FcUnpublish) {
        self.encode(&fc_unpublish.get_transaction_id());
        self.encode(&Null);
        self.encode(fc_unpublish.get_playpath());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_fc_unpublish() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Number::new(6f64));
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());

        let result: IOResult<FcUnpublish> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = FcUnpublish::new(6.into(), AmfString::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_fc_unpublish() {
        let mut buffer = ByteBuffer::default();
        let expected_transaction_id = 6f64;
        let expected_playpath = "";
        let expected = FcUnpublish::new(Number::new(expected_transaction_id), AmfString::from(expected_play_path));
        buffer.encode(&expected);

        let actual_transaction_id: Number = buffer.decode().unwrap();
        assert_eq!(expected_transaction_id, actual_transaction_id);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_playpath: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_playpath, actual_playpath)
    }
}
