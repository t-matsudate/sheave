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

/// The command to tell the PlayPath. (e.g. something file name)
#[derive(Debug, Clone, PartialEq)]
pub struct GetStreamLength(AmfString);

impl GetStreamLength {
    /// Constructs a GetStreamLength command.
    pub fn new(playpath: AmfString) -> Self {
        Self(playpath)
    }

    /// Gets the Play Path.
    pub fn get_playpath(&self) -> &AmfString {
        &self.0
    }
}

impl From<GetStreamLength> for AmfString {
    fn from(get_stream_length: GetStreamLength) -> Self {
        get_stream_length.0
    }
}

impl ChunkData for GetStreamLength {
    const CHANNEL: Channel = Channel::Source;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for GetStreamLength {}

impl Decoder<GetStreamLength> for ByteBuffer {
    /// Decodes bytes into a GetStreamLength command.
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
    ///         GetStreamLength,
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
    /// assert!(Decoder::<GetStreamLength>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<GetStreamLength>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    fn decode(&mut self) -> IOResult<GetStreamLength> {
        Decoder::<Null>::decode(self)?;
        let playpath: AmfString = self.decode()?;
        Ok(GetStreamLength(playpath))
    }
}

impl Encoder<GetStreamLength> for ByteBuffer {
    /// Encodes a GetStreamLength command into bytes.
    fn encode(&mut self, get_stream_length: &GetStreamLength) {
        self.encode(&Null);
        self.encode(get_stream_length.get_playpath());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_get_stream_length() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(&AmfString::default());
        let result: IOResult<GetStreamLength> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = GetStreamLength::new(AmfString::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_get_stream_length() {
        let mut buffer = ByteBuffer::default();
        let expected_playpath = "";
        let expected = GetStreamLength::new(AmfString::from(expected_playpath));
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_playpath: AmfString = buffer.decode().unwrap();
        assert_eq!(expected_playpath, actual_playpath)
    }
}
