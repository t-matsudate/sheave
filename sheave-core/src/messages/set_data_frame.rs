use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder
};
use super::{
    Channel,
    ChunkData,
    amf::v0::AmfString,
    ensure_command_name,
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
    /// use rand::{
    ///     Fill,
    ///     thread_rng
    /// };
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         SetDataFrame,
    ///         amf::v0::AmfString,
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// let mut bytes: [u8; 128] = [0; 128];
    /// bytes.try_fill(&mut thread_rng()).unwrap();
    /// buffer.encode(&AmfString::from("@setDataFrame"));
    /// buffer.put_bytes(&bytes);
    /// assert!(Decoder::<SetDataFrame>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.encode(&AmfString::from("something else"));
    /// assert!(Decoder::<SetDataFrame>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentMarker`]: crate::messages::amf::InconsistentMarker
    /// [`InvalidString`]: crate::messages::amf::InvalidString
    /// [`InconsistentCommand`]: super::InconsistentCommand
    fn decode(&mut self) -> IOResult<SetDataFrame> {
        Decoder::<AmfString>::decode(self).and_then(
            |command| ensure_command_name("@setDataFrame", command)
        )?;

        let remained = self.remained();
        let bytes = self.get_bytes(remained)?.to_vec();
        Ok(SetDataFrame(bytes))
    }
}

impl Encoder<SetDataFrame> for ByteBuffer {
    /// Encodes a SetDataFrame message into bytes.
    fn encode(&mut self, set_data_frame: &SetDataFrame) {
        self.encode(&AmfString::from("@setDataFrame"));
        self.put_bytes(&set_data_frame.0);
    }
}

#[cfg(test)]
mod tests {
    use rand::{
        Fill,
        thread_rng
    };
    use super::*;

    #[test]
    fn decode_set_data_frame() {
        let mut buffer = ByteBuffer::default();
        let mut bytes: [u8; 128] = [0; 128];
        bytes.try_fill(&mut thread_rng()).unwrap();
        buffer.encode(&AmfString::from("@setDataFrame"));
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
        expected_bytes.try_fill(&mut thread_rng()).unwrap();
        let expected = SetDataFrame::new(expected_bytes.to_vec());
        buffer.encode(&expected);
        let message_name: AmfString = buffer.decode().unwrap();
        assert_eq!("@setDataFrame", message_name);
        let actual_data: Vec<u8> = buffer.into();
        assert_eq!(expected_bytes.as_slice(), &actual_data)
    }
}
