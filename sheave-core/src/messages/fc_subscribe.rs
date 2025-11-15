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

/// The command to tell the topic path.
#[derive(Debug, Clone, PartialEq)]
pub struct FcSubscribe(AmfString);

impl FcSubscribe {
    /// Constructs a FcSubscribe command.
    pub fn new(topic_id: AmfString) -> Self {
        Self(topic_id)
    }

    /// Gets the topic path.
    pub fn get_topic_id(&self) -> &AmfString {
        &self.0
    }
}

impl From<FcSubscribe> for AmfString {
    fn from(fc_subscribe: FcSubscribe) -> Self {
        fc_subscribe.0
    }
}

impl ChunkData for FcSubscribe {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for FcSubscribe {}

impl Decoder<FcSubscribe> for ByteBuffer {
    /// Decodes bytes into a FcSubscribe command.
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
    ///         FcSubscribe,
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
    /// assert!(Decoder::<FcSubscribe>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<FcSubscribe>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<FcSubscribe> {
        Decoder::<Null>::decode(self)?;
        let topic_id: AmfString = self.decode()?;
        Ok(FcSubscribe(topic_id))
    }
}

impl Encoder<FcSubscribe> for ByteBuffer {
    /// Encodes a FcSubscribe command into bytes.
    fn encode(&mut self, fc_subscribe: &FcSubscribe) {
        self.encode(&Null);
        self.encode(fc_subscribe.get_topic_id());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_fc_subscribe() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());
        let result: IOResult<FcSubscribe> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = FcSubscribe::new(AmfString::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_fc_subscribe() {
        let mut buffer = ByteBuffer::default();
        let expected_topic_id = "";
        let expected = FcSubscribe::new(AmfString::from(expected_topic_id));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_topic_id: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_topic_id, actual_topic_id)
    }
}
