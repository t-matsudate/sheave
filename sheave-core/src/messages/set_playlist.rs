use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        Channel,
        ChunkData,
        Command,
        amf::v0::{
            EcmaArray,
            Null
        },
        headers::MessageType
    },
};

/// The command to tell the Playlist of streams.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetPlaylist(EcmaArray);

impl SetPlaylist {
    /// Constructs a SetPlaylist command.
    pub fn new(playpaths: EcmaArray) -> Self {
        Self(playpaths)
    }

    /// Gets the playlist.
    pub fn get_playpaths(&self) -> &EcmaArray {
        &self.0
    }
}

impl From<SetPlaylist> for EcmaArray {
    fn from(set_playlist: SetPlaylist) -> Self {
        set_playlist.0
    }
}

impl ChunkData for SetPlaylist {
    const CHANNEL: Channel = Channel::Video;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for SetPlaylist {}

impl Decoder<SetPlaylist> for ByteBuffer {
    /// Decodes bytes into a SetPlaylist command.
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
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         SetPlaylist,
    ///         amf::v0::{
    ///             Null,
    ///             EcmaArray
    ///         },
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&Null);
    /// buffer.encode(&EcmaArray::default());
    /// assert!(Decoder::<SetPlaylist>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<SetPlaylist>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    fn decode(&mut self) -> IOResult<SetPlaylist> {
        Decoder::<Null>::decode(self)?;
        let playpaths: EcmaArray = self.decode()?;
        Ok(SetPlaylist(playpaths))
    }
}

impl Encoder<SetPlaylist> for ByteBuffer {
    /// Encodes a SetPlaylist command into bytes.
    fn encode(&mut self, set_playlist: &SetPlaylist) {
        self.encode(&Null);
        self.encode(set_playlist.get_playpaths());
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ecma_array,
        messages::amf::v0::AmfString,
    };
    use super::*;

    #[test]
    fn decode_set_playlist() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&Null);
        buffer.encode(
            &ecma_array!(
                "0" => AmfString::default()
            )
        );
        let result: IOResult<SetPlaylist> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = SetPlaylist::new(
            ecma_array!(
                "0" => AmfString::default()
            )
        );
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_set_playlist() {
        let mut buffer = ByteBuffer::default();
        let expected_playpaths = ecma_array!(
            "0" => AmfString::default()
        );
        let expected = SetPlaylist::new(expected_playpaths.clone());
        buffer.encode(&expected);
        Decoder::<Null>::decode(&mut buffer).unwrap();
        let actual_playpaths: EcmaArray = buffer.decode().unwrap();
        assert_eq!(expected_playpaths, actual_playpaths)
    }
}
