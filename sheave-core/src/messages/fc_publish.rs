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

/// The command to tell the playpath.
/// Typically, this becomes same as the releaseStream's one.
#[derive(Debug, Clone, PartialEq)]
pub struct FcPublish(AmfString);

impl FcPublish {
    /// Constructs a FcPublish command.
    pub fn new(playpath: AmfString) -> Self {
        Self(playpath)
    }

    /// Gets the playpath.
    pub fn get_playpath(&self) -> &AmfString {
        &self.0
    }
}

impl From<FcPublish> for AmfString {
    fn from(fc_publish: FcPublish) -> Self {
        fc_publish.0
    }
}

impl ChunkData for FcPublish {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for FcPublish {}

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
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
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
        Decoder::<Null>::decode(self)?;
        let playpath: AmfString = self.decode()?;
        Ok(FcPublish(playpath))
    }
}

impl Encoder<FcPublish> for ByteBuffer {
    /// Encodes a FcPublish command into bytes.
    fn encode(&mut self, fc_publish: &FcPublish) {
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
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());
        let result: IOResult<FcPublish> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = FcPublish::new(AmfString::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_fc_publish() {
        let mut buffer = ByteBuffer::default();
        let expected_playpath = "";
        let expected = FcPublish::new(AmfString::from(expected_playpath));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_playpath: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_playpath, actual_playpath)
    }
}
