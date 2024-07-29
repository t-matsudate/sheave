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

/// The command to tell the Play Path (e.g. something file name).
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseStream(AmfString);

impl ReleaseStream {
    /// Constructs a ReleaseStream command.
    pub fn new(playpath: AmfString) -> Self {
        Self(playpath)
    }

    /// Gets the Play Path.
    pub fn get_playpath(&self) -> &AmfString {
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
        let playpath: AmfString = self.decode()?;
        Ok(ReleaseStream(playpath))
    }
}

impl Encoder<ReleaseStream> for ByteBuffer {
    /// Encodes a ReleaseStream command into bytes.
    fn encode(&mut self, release_stream: &ReleaseStream) {
        self.encode(&Null);
        self.encode(release_stream.get_playpath());
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
        let expected_playpath = "";
        let expected = ReleaseStream::new(AmfString::from(expected_playpath));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_playpath: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_playpath, actual_playpath)
    }
}
