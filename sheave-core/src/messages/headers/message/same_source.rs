use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct SameSource {
    pub timestamp: Duration,
    pub message_length: u32,
    pub message_type: u8
}
