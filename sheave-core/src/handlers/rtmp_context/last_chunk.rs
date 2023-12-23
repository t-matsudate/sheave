use std::{
    time::Duration
};
use crate::{
    messages::headers::{
        MessageHeader,
        MessageType,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct LastChunk {
    timestamp: Duration,
    message_length: u32,
    message_type: MessageType,
    message_id: u32
}

impl LastChunk {
    pub fn new(message_header: MessageHeader) -> Self {
        Self {
            timestamp: message_header.get_timestamp().unwrap(),
            message_length: message_header.get_message_length().unwrap(),
            message_type: message_header.get_message_type().unwrap(),
            message_id: message_header.get_message_id().unwrap()
        }
    }

    pub fn get_timestamp(&self) -> Duration {
        self.timestamp
    }

    pub fn get_message_length(&self) -> u32 {
        self.message_length
    }

    pub fn get_message_type(&self) -> MessageType {
        self.message_type
    }

    pub fn get_message_id(&self) -> u32 {
        self.message_id
    }

    pub fn update(&mut self, message_header: &MessageHeader, extended_timestamp: Option<Duration>) {
        if let Some(extended_timestamp) = extended_timestamp {
            self.timestamp += extended_timestamp;
        } else {
            self.timestamp += message_header.get_timestamp().unwrap();
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
