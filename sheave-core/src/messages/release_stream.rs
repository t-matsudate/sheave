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

/// The command to tell some path to the server.
///
/// Following format is required.
///
/// |Field|AMF Type|Value|
/// | :- | :- | :- |
/// |Command Name|[`String`]|`"releaseStream"`|
/// |Transaction ID|[`Number`]|A number which is next of the connect.|
/// ||[`Null`]|Nothing but an AMF's type marker is in.|
/// |ID|[`String`]|Some ID to audio/video data to make server opened.|
///
/// For example, ID is defined as some file name in FFmpeg and OBS.
///
/// [`Number`]: crate::messages::amf::v0::Number
/// [`String`]: crate::messages::amf::v0::AmfString
/// [`Null`]: crate::messages::amf::v0::Null
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseStream(AmfString);

impl ReleaseStream {
    /// Constructs a ReleaseStream command.
    pub fn new(topic_id: AmfString) -> Self {
        Self(topic_id)
    }

    /// Gets the topic ID.
    pub fn get_topic_id(&self) -> &AmfString {
        &self.0
    }
}

impl From<ReleaseStream> for AmfString {
    fn from(release_stream: ReleaseStream) -> Self {
        release_stream.0
    }
}

impl ChunkData for ReleaseStream {
    const CHANNEL: Channel = Channel::System;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for ReleaseStream {}

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
    ///             AmfString,
    ///             Null
    ///         }
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// buffer.encode(&AmfString::default());
    /// assert!(Decoder::<ReleaseStream>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<ReleaseStream>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<ReleaseStream> {
        Decoder::<Null>::decode(self)?;
        let topic_id: AmfString = self.decode()?;
        Ok(ReleaseStream(topic_id))
    }
}

impl Encoder<ReleaseStream> for ByteBuffer {
    /// Encodes a ReleaseStream command into bytes.
    fn encode(&mut self, release_stream: &ReleaseStream) {
        self.encode(&Null);
        self.encode(release_stream.get_topic_id());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_release_stream() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());
        let result: IOResult<ReleaseStream> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = ReleaseStream::new(AmfString::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_release_stream() {
        let mut buffer = ByteBuffer::default();
        let expected_topic_id = "";
        let expected = ReleaseStream::new(AmfString::from(expected_topic_id));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_topic_id: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_topic_id, actual_topic_id)
    }
}
