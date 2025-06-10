use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder
};
use super::{
    Channel,
    ChunkData,
    headers::MessageType
};

/// The message to handle something data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetDataFrame(Vec<u8>);

impl SetDataFrame {
    /// Constructs a new SetDataFrame message.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<SetDataFrame> for Vec<u8> {
    fn from(set_data_frame: SetDataFrame) -> Self {
        set_data_frame.0
    }
}

impl ChunkData for SetDataFrame {
    const CHANNEL: Channel = Channel::Audio;
    const MESSAGE_TYPE: MessageType = MessageType::Data;
}

impl Decoder<SetDataFrame> for ByteBuffer {
    /// Decodes bytes into a SetDataFrame message.
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
    /// * [`InconsistentCommand`]
    ///
    /// When the command name isn't `"@setDataFrame"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::fill;
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::SetDataFrame
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// let mut bytes: [u8; 128] = [0; 128];
    /// fill(&mut bytes);
    /// buffer.put_bytes(&bytes);
    /// assert!(Decoder::<SetDataFrame>::decode(&mut buffer).is_ok());
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<SetDataFrame> {
        let remained = self.remained();
        let bytes = self.get_bytes(remained)?.to_vec();
        Ok(SetDataFrame(bytes))
    }
}

impl Encoder<SetDataFrame> for ByteBuffer {
    /// Encodes a SetDataFrame message into bytes.
    fn encode(&mut self, set_data_frame: &SetDataFrame) {
        self.put_bytes(&set_data_frame.0);
    }
}

#[cfg(test)]
mod tests {
    use rand::fill;
    use super::*;

    #[test]
    fn decode_set_data_frame() {
        let mut buffer = ByteBuffer::default();
        let mut bytes: [u8; 128] = [0; 128];
        fill(&mut bytes);
        buffer.put_bytes(&bytes);
        let result: IOResult<SetDataFrame> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = SetDataFrame::new(bytes.to_vec());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_set_data_frame() {
        let mut buffer = ByteBuffer::default();
        let mut expected_bytes: [u8; 128] = [0; 128];
        fill(&mut expected_bytes);
        let expected = SetDataFrame::new(expected_bytes.to_vec());
        buffer.encode(&expected);
        let actual_data: Vec<u8> = buffer.into();
        assert_eq!(expected_bytes.as_slice(), &actual_data)
    }
}
