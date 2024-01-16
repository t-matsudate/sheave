use std::{
    time::Duration
};
use crate::{
    messages::headers::{
        MessageHeader,
        MessageType,
    },
};

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
    /// Note this panics when fields are incomplete because regards a message header passed as the 11 bytes type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::panic::catch_unwind;
    ///
    /// let panicked = catch_unwind(
    ///     || {
    ///         use std::time::Duration;
    ///         use sheave_core::{
    ///             messages::headers::{
    ///                 MessageType,
    ///                 MessageHeader
    ///             },
    ///             handlers::LastChunk
    ///         };
    ///
    ///         LastChunk::new(MessageHeader::SameSource((Duration::default(), u32::default(), MessageType::Command).into()))
    ///     }
    /// );
    /// assert!(panicked.is_err());
    /// ```
    pub fn new(message_header: MessageHeader) -> Self {
        Self {
            timestamp: message_header.get_timestamp().unwrap(),
            message_length: message_header.get_message_length().unwrap(),
            message_type: message_header.get_message_type().unwrap(),
            message_id: message_header.get_message_id().unwrap()
        }
    }

    /// Gets a timestamp.
    pub fn get_timestamp(&self) -> Duration {
        self.timestamp
    }

    /// Gets a message length.
    pub fn get_message_length(&self) -> u32 {
        self.message_length
    }

    /// Gets a message type.
    pub fn get_message_type(&self) -> MessageType {
        self.message_type
    }

    /// Gets a message ID.
    pub fn get_message_id(&self) -> u32 {
        self.message_id
    }

    /// Updates this chunk information into a new one passed.
    /// Note if any extended timestamp is passed, this increases its timestamp using it instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use sheave_core::{
    ///     handlers::LastChunk,
    ///     messages::headers::{
    ///         MessageHeader,
    ///         MessageType
    ///     }
    /// };
    ///
    /// let mut last_chunk = LastChunk::new(MessageHeader::New((Duration::default(), u32::default(), MessageType::Command, u32::default()).into()));
    ///
    /// last_chunk.update(&MessageHeader::New((Duration::from_millis(1), 1, MessageType::Command, 1).into()), None);
    /// assert_eq!(Duration::from_millis(1), last_chunk.get_timestamp());
    /// assert_eq!(1, last_chunk.get_message_length());
    /// assert_eq!(MessageType::Command, last_chunk.get_message_type());
    /// assert_eq!(1, last_chunk.get_message_id());
    ///
    /// last_chunk.update(&MessageHeader::New((Duration::from_millis(2), 2, MessageType::Other, 2).into()), Some(Duration::from_millis(0x01000000 as u64)));
    /// assert_eq!(Duration::from_millis(0x01000000 as u64), last_chunk.get_timestamp());
    /// assert_eq!(2, last_chunk.get_message_length());
    /// assert_eq!(MessageType::Other, last_chunk.get_message_type());
    /// assert_eq!(2, last_chunk.get_message_id())
    /// ```
    pub fn update(&mut self, message_header: &MessageHeader, extended_timestamp: Option<Duration>) {
        if let Some(extended_timestamp) = extended_timestamp {
            self.timestamp = extended_timestamp;
        } else {
            self.timestamp = message_header.get_timestamp().unwrap();
        }

        message_header.get_message_length().map(
            |message_length| self.message_length = message_length
        );
        message_header.get_message_type().map(
            |message_type| self.message_type = message_type
        );
        message_header.get_message_id().map(
            |message_id| self.message_id = message_id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn last_chunk_constructs_with_not_new() {
        LastChunk::new(MessageHeader::SameSource((Duration::default(), u32::default(), MessageType::Command).into()));
    }
}
