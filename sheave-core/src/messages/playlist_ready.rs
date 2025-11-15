use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        Channel,
        ChunkData,
        Command,
        headers::MessageType
    },
};

/// The response message for SetPlaylist requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaylistReady;

impl ChunkData for PlaylistReady {
    const CHANNEL: Channel = Channel::Video;
    const MESSAGE_TYPE: MessageType = MessageType::Command;
}

impl Command for PlaylistReady {}

impl Decoder<PlaylistReady> for ByteBuffer {
    /// Decodes bytes into a playlist ready 
    fn decode(&mut self) -> IOResult<PlaylistReady> {
        Ok(PlaylistReady)
    }
}

impl Encoder<PlaylistReady> for ByteBuffer {
    /// Encodes a playlist ready command into bytes.
    /// However this encodes nothing because has no field.
    fn encode(&mut self, _: &PlaylistReady) {
        return
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_playlist_ready() {
        let mut buffer = ByteBuffer::default();
        let result: IOResult<PlaylistReady> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        assert_eq!(PlaylistReady, actual)
    }

    #[test]
    fn encode_playlist_ready() {
        let mut buffer = ByteBuffer::default();
        buffer.encode(&PlaylistReady);
    }
}
