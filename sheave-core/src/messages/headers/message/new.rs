use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct New {
    pub timestamp: Duration,
    pub message_length: u32,
    pub message_type: u8,
    pub message_id: u32
}
