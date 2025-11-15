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
        AmfString,
        Null
    }
};

/// This command is same as [`FcPublish`] except this requests to delete the topic path.
///
/// [`FcPublish`]: super::FcPublish
#[derive(Debug, Clone, PartialEq)]
pub struct FcUnpublish(AmfString);

impl FcUnpublish {
    /// Constructs a FcUnpublish command.
    pub fn new(topic_id: AmfString) -> Self {
        Self(topic_id)
    }

    /// Gets the topic ID.
    pub fn get_topic_id(&self) -> &AmfString {
        &self.0
    }
}

impl From<FcUnpublish> for AmfString {
    fn from(fc_unpublish: FcUnpublish) -> Self {
        fc_unpublish.0
    }
}

impl ChunkData for FcUnpublish {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for FcUnpublish {}

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
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
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
        Decoder::<Null>::decode(self)?;
        let topic_id: AmfString = self.decode()?;

        Ok(FcUnpublish(topic_id))
    }
}

impl Encoder<FcUnpublish> for ByteBuffer {
    /// Encodes a FcUnpublish command into bytes.
    fn encode(&mut self, fc_unpublish: &FcUnpublish) {
        self.encode(&Null);
        self.encode(fc_unpublish.get_topic_id());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_fc_unpublish() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());

        let result: IOResult<FcUnpublish> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = FcUnpublish::new(AmfString::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_fc_unpublish() {
        let mut buffer = ByteBuffer::default();
        let expected_topic_id = "";
        let expected = FcUnpublish::new(AmfString::from(expected_topic_path));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_topic_id: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_topic_id, actual_topic_id)
    }
}
