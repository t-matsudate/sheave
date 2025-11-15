use std::time::Duration;
use crate::messages::headers::MessageType;

/// The chunk information which is sent/received last.
#[derive(Debug, Clone, Copy)]
pub struct LastChunk {
    timestamp: Duration,
    message_length: u32,
    message_type: MessageType,
    message_id: u32
}

impl LastChunk {
    /// Constructs a LastChunk.
    ///
    /// Note the message ID is set 0 when message header isn't `New`.
    /// e.g. On receiving `StreamBegin`.
    pub fn new(timestamp: Duration, message_length: u32, message_type: MessageType, message_id: u32) -> Self {
        Self { timestamp, message_length, message_type, message_id }
    }

    /// Sets a timestamp.
    pub fn set_timestamp(&mut self, timestamp: Duration) {
        self.timestamp = timestamp;
    }

    /// Gets a timestamp.
    pub fn get_timestamp(&self) -> Duration {
        self.timestamp
    }

    /// Sets a message length.
    pub fn set_message_length(&mut self, message_length: u32) {
        self.message_length = message_length;
    }

    /// Gets a message length.
    pub fn get_message_length(&self) -> u32 {
        self.message_length
    }

    /// Sets a message type.
    pub fn set_message_type(&mut self, message_type: MessageType) {
        self.message_type = message_type;
    }

    /// Gets a message type.
    pub fn get_message_type(&self) -> MessageType {
        self.message_type
    }

    /// Sets a message ID.
    pub fn set_message_id(&mut self, message_id: u32) {
        self.message_id = message_id;
    }

    /// Gets a message ID.
    pub fn get_message_id(&self) -> u32 {
        self.message_id
    }
}
