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

/// This command is same as [`FcPublish`] except this requests to delete the playpath.
///
/// [`FcPublish`]: super::FcPublish
#[derive(Debug, Clone, PartialEq)]
pub struct FcUnpublish(AmfString);

impl FcUnpublish {
    /// Constructs a FcUnpublish command.
    pub fn new(playpath: AmfString) -> Self {
        Self(playpath)
    }

    /// Gets the playpath.
    pub fn get_playpath(&self) -> &AmfString {
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
        let playpath: AmfString = self.decode()?;

        Ok(FcUnpublish(playpath))
    }
}

impl Encoder<FcUnpublish> for ByteBuffer {
    /// Encodes a FcUnpublish command into bytes.
    fn encode(&mut self, fc_unpublish: &FcUnpublish) {
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
        let expected_playpath = "";
        let expected = FcUnpublish::new(AmfString::from(expected_playpath));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_playpath: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_playpath, actual_playpath)
    }
}
