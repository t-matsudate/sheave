use std::io::Result as IOResult;
use crate::{
    ByteBuffer,
    Decoder,
    Encoder,
    messages::{
        Channel,
        ChunkData,
        EventType,
        UserControl,
        headers::MessageType,
    }
};

/// The event to tell that its stream will be buffered, to the server.
///
/// Following format is required:
///
/// |Event Data|Length (in bytes)|Description|
/// | :- | -: | :- |
/// |Message ID|4|The message ID which is same as contained in [`createStream`].|
/// |Buffer Length|4|**A time length** to charge a data into a stream (in milliseconds).|
///
/// [`createStream`]: crate::messages::CreateStream
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetBufferLength(u32, u32);

impl SetBufferLength {
    /// Constructs a SetBufferLength event.
    pub fn new(message_id: u32, buffering_time: u32) -> Self {
        Self(message_id, buffering_time)
    }

    /// Gets the message ID which this event has.
    pub fn get_message_id(&self) -> u32 {
        self.0
    }

    /// Gets the buffering time which this event has.
    pub fn get_buffering_time(&self) -> u32 {
        self.1
    }
}

impl From<SetBufferLength> for (u32, u32) {
    fn from(set_buffer_length: SetBufferLength) -> Self {
        (set_buffer_length.0, set_buffer_length.1)
    }
}

impl ChunkData for SetBufferLength {
    const CHANNEL: Channel = Channel::Network;
    const MESSAGE_TYPE: MessageType = MessageType::UserControl;
}

impl UserControl for SetBufferLength {
    const EVENT_TYPE: EventType = EventType::SetBufferLength;
}

impl Decoder<SetBufferLength> for ByteBuffer {
    /// Decodes bytes into a SetBufferLength event.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When some field misses.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::{
    ///     ByteBuffer,
    ///     Decoder,
    ///     Encoder,
    ///     messages::{
    ///         EventType,
    ///         SetBufferLength
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u32_be(u32::default());
    /// buffer.put_u32_be(u32::default());
    /// assert!(Decoder::<SetBufferLength>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// assert!(Decoder::<SetBufferLength>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    fn decode(&mut self) -> IOResult<SetBufferLength> {
        let message_id = self.get_u32_be()?;
        let buffering_time = self.get_u32_be()?;
        Ok(SetBufferLength(message_id, buffering_time))
    }
}

impl Encoder<SetBufferLength> for ByteBuffer {
    /// Encodes a SetBufferLength event into bytes.
    fn encode(&mut self, set_buffer_length: &SetBufferLength) {
        self.put_u32_be(set_buffer_length.get_message_id());
        self.put_u32_be(set_buffer_length.get_buffering_time());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_set_buffer_length() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u32_be(u32::default());
        buffer.put_u32_be(u32::default());
        let result: IOResult<SetBufferLength> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = SetBufferLength::new(u32::default(), u32::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_set_buffer_length() {
        let mut buffer = ByteBuffer::default();
        let expected_message_id = u32::default();
        let expected_buffering_time = u32::default();
        let expected = SetBufferLength::new(expected_message_id, expected_buffering_time);
        buffer.encode(&expected);
        let actual_message_id = buffer.get_u32_be().unwrap();
        assert_eq!(expected_message_id, actual_message_id);
        let actual_buffering_time = buffer.get_u32_be().unwrap();
        assert_eq!(expected_buffering_time, actual_buffering_time)
    }
}
