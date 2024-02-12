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
        ensure_event_type
    },
};

/// The event to tell that the stream is ready to a client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamBegin(u32);

impl StreamBegin {
    /// Constructs a StreamBegin event.
    pub fn new(message_id: u32) -> Self {
        Self(message_id)
    }

    /// Gets the message ID which this event has.
    pub fn get_message_id(&self) -> u32 {
        self.0
    }
}

impl From<StreamBegin> for u32 {
    fn from(stream_begin: StreamBegin) -> Self {
        stream_begin.0
    }
}

impl ChunkData for StreamBegin {
    const CHANNEL: Channel = Channel::Network;
    const MESSAGE_TYPE: MessageType = MessageType::UserControl;
}

impl UserControl for StreamBegin {
    const EVENT_TYPE: EventType = EventType::StreamBegin;
}

impl Decoder<StreamBegin> for ByteBuffer {
    /// Decodes bytes into a StreamBegin event.
    ///
    /// # Errors
    ///
    /// * [`InsufficientBufferLength`]
    ///
    /// When some field misses.
    ///
    /// * [`InconsistentEventType`]
    ///
    /// When the event type isn't `0` (Stream Begin).
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
    ///         StreamBegin
    ///     }
    /// };
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u16_be(EventType::StreamBegin as u16);
    /// buffer.put_u32_be(u32::default());
    /// assert!(Decoder::<StreamBegin>::decode(&mut buffer).is_ok());
    ///
    /// let mut buffer = ByteBuffer::default();
    /// buffer.put_u16_be(EventType::Other as u16);
    /// buffer.put_u32_be(u32::default());
    /// assert!(Decoder::<StreamBegin>::decode(&mut buffer).is_err())
    /// ```
    ///
    /// [`InsufficientBufferLength`]: crate::byte_buffer::InsufficientBufferLength
    /// [`InconsistentEventType`]: super::InconsistentEventType
    fn decode(&mut self) -> IOResult<StreamBegin> {
        self.get_u16_be().and_then(
            |event_type| ensure_event_type(EventType::StreamBegin, event_type)
        )?;

        let message_id = self.get_u32_be()?;
        Ok(StreamBegin(message_id))
    }
}

impl Encoder<StreamBegin> for ByteBuffer {
    /// Encodes a StreamBegin event into bytes.
    fn encode(&mut self, stream_begin: &StreamBegin) {
        self.put_u16_be(StreamBegin::EVENT_TYPE as u16);
        self.put_u32_be(stream_begin.get_message_id());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_stream_begin() {
        let mut buffer = ByteBuffer::default();
        buffer.put_u16_be(StreamBegin::EVENT_TYPE as u16);
        buffer.put_u32_be(u32::default());
        let result: IOResult<StreamBegin> = buffer.decode();
        assert!(result.is_ok());
        let actual = result.unwrap();
        let expected = StreamBegin::new(u32::default());
        assert_eq!(expected, actual)
    }

    #[test]
    fn encode_stream_begin() {
        let mut buffer = ByteBuffer::default();
        let expected_message_id = u32::default();
        let expected = StreamBegin::new(expected_message_id);
        buffer.encode(&expected);
        let event_type = buffer.get_u16_be().unwrap();
        assert_eq!(EventType::StreamBegin, EventType::from(event_type));
        let actual_message_id = buffer.get_u32_be().unwrap();
        assert_eq!(expected_message_id, actual_message_id)
    }
}
